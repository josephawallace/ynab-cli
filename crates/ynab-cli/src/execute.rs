use std::io::Read;

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use ynab_sdk::{
    Client, CreateTransactions, ExistingTransaction, GoalFrequency, NewCategory, NewTransaction,
    Nullable, PlanId, PostPayee, SaveAccount, SaveAccountType, SaveCategory, SaveCategoryGroup,
    SaveMonthCategory, SavePayee, SaveScheduledTransaction, SaveTransactionWithIdOrImportId,
    ScheduledTransactionFrequency, ServerKnowledge, TransactionClearedStatus, TransactionFilters,
    TransactionFlagColor, TransactionType,
};

use crate::{
    commands::{
        AccountCommand, CategoryCommand, CategoryGroupCommand, CategoryMonthCommand, ClearedArg,
        Command, FlagArg, GoalFrequencyArg, MoneyMovementCommand, MoneyMovementGroupCommand,
        MonthCommand, PayeeCommand, PayeeLocationCommand, PlanCommand, SaveAccountTypeArg,
        ScheduledFrequencyArg, ScheduledTransactionCommand, ScheduledUpdate, ScheduledWrite,
        TransactionCommand, TransactionCreate, TransactionCreateFields, TransactionList,
        TransactionTypeArg, TransactionUpdate, UserCommand,
    },
    error::CliError,
};

pub async fn execute(
    command: Command,
    client: &Client,
    input: &mut dyn Read,
) -> Result<Value, CliError> {
    match command {
        Command::User(args) => match args.command {
            UserCommand::Show => value(client.user().get().await?.user),
        },
        Command::Plan(args) => execute_plan(args.command, client).await,
        Command::Account(args) => execute_account(args.command, client).await,
        Command::Category(args) => execute_category(args.command, client).await,
        Command::CategoryGroup(args) => execute_category_group(args.command, client).await,
        Command::Payee(args) => execute_payee(args.command, client).await,
        Command::Month(args) => execute_month(args.command, client).await,
        Command::MoneyMovement(args) => execute_money_movement(args.command, client).await,
        Command::MoneyMovementGroup(args) => {
            execute_money_movement_group(args.command, client).await
        }
        Command::Transaction(args) => execute_transaction(args.command, client, input).await,
        Command::ScheduledTransaction(args) => {
            execute_scheduled_transaction(args.command, client).await
        }
        Command::Auth(_) => Err(CliError::Invocation(
            "auth commands must be executed through the credential handler".into(),
        )),
    }
}

async fn execute_plan(command: PlanCommand, client: &Client) -> Result<Value, CliError> {
    match command {
        PlanCommand::List { include_accounts } => {
            value(client.plans().list(include_accounts).await?)
        }
        PlanCommand::Show {
            plan,
            last_knowledge,
        } => value(
            client
                .plans()
                .get(&read_plan(plan), last_knowledge.map(ServerKnowledge))
                .await?,
        ),
        PlanCommand::Settings { plan } => {
            value(client.plans().settings(&read_plan(plan)).await?.settings)
        }
    }
}

async fn execute_account(command: AccountCommand, client: &Client) -> Result<Value, CliError> {
    match command {
        AccountCommand::List {
            plan,
            last_knowledge,
        } => value(
            client
                .accounts()
                .list(&read_plan(plan), last_knowledge.map(ServerKnowledge))
                .await?,
        ),
        AccountCommand::Show { id, plan } => {
            value(client.accounts().get(&read_plan(plan), id).await?.account)
        }
        AccountCommand::Create {
            plan,
            name,
            account_type,
            balance,
        } => {
            let request = SaveAccount {
                name,
                account_type: save_account_type(account_type),
                balance: balance.0,
            };
            value(client.accounts().create(&plan, &request).await?.account)
        }
    }
}

async fn execute_category(command: CategoryCommand, client: &Client) -> Result<Value, CliError> {
    match command {
        CategoryCommand::List {
            plan,
            last_knowledge,
        } => value(
            client
                .categories()
                .list(&read_plan(plan), last_knowledge.map(ServerKnowledge))
                .await?,
        ),
        CategoryCommand::Show { id, plan } => value(
            client
                .categories()
                .get(&read_plan(plan), id)
                .await?
                .category,
        ),
        CategoryCommand::Create(args) => {
            let request = NewCategory {
                name: Some(args.name),
                note: nullable(args.note),
                category_group_id: args.category_group,
                goal_target: nullable(args.goal_target.map(|amount| amount.0)),
                goal_target_date: nullable(args.goal_target_date.map(|date| date.0)),
                goal_needs_whole_amount: nullable(args.goal_needs_whole_amount),
                goal_frequency: args.goal_frequency.map(goal_frequency),
            };
            value(
                client
                    .categories()
                    .create(&args.plan, &request)
                    .await?
                    .category,
            )
        }
        CategoryCommand::Update(args) => {
            let request = SaveCategory {
                name: nullable(args.name),
                note: nullable(args.note),
                category_group_id: args.category_group,
                goal_target: nullable(args.goal_target.map(|amount| amount.0)),
                goal_target_date: nullable(args.goal_target_date.map(|date| date.0)),
                goal_needs_whole_amount: nullable(args.goal_needs_whole_amount),
                goal_frequency: args.goal_frequency.map(goal_frequency),
            };
            value(
                client
                    .categories()
                    .update(&args.plan, args.id, &request)
                    .await?
                    .category,
            )
        }
        CategoryCommand::Month(args) => match args.command {
            CategoryMonthCommand::Show { month, id, plan } => value(
                client
                    .categories()
                    .get_month(&read_plan(plan), month, id)
                    .await?
                    .category,
            ),
            CategoryMonthCommand::Assign {
                month,
                id,
                plan,
                amount,
            } => value(
                client
                    .categories()
                    .update_month(&plan, month, id, &SaveMonthCategory { budgeted: amount.0 })
                    .await?
                    .category,
            ),
        },
    }
}

async fn execute_category_group(
    command: CategoryGroupCommand,
    client: &Client,
) -> Result<Value, CliError> {
    match command {
        CategoryGroupCommand::Create { plan, name } => value(
            client
                .category_groups()
                .create(&plan, &SaveCategoryGroup { name })
                .await?
                .category_group,
        ),
        CategoryGroupCommand::Update { id, plan, name } => value(
            client
                .category_groups()
                .update(&plan, id, &SaveCategoryGroup { name })
                .await?
                .category_group,
        ),
    }
}

async fn execute_payee(command: PayeeCommand, client: &Client) -> Result<Value, CliError> {
    match command {
        PayeeCommand::List {
            plan,
            last_knowledge,
        } => value(
            client
                .payees()
                .list(&read_plan(plan), last_knowledge.map(ServerKnowledge))
                .await?,
        ),
        PayeeCommand::Show { id, plan } => {
            value(client.payees().get(&read_plan(plan), id).await?.payee)
        }
        PayeeCommand::Create { plan, name } => value(
            client
                .payees()
                .create(&plan, &PostPayee { name })
                .await?
                .payee,
        ),
        PayeeCommand::Update { id, plan, name } => value(
            client
                .payees()
                .update(&plan, id, &SavePayee { name: Some(name) })
                .await?
                .payee,
        ),
        PayeeCommand::Location(args) => match args.command {
            PayeeLocationCommand::List { plan, payee } => match payee {
                Some(payee) => value(
                    client
                        .payee_locations()
                        .list_for_payee(&read_plan(plan), payee)
                        .await?,
                ),
                None => value(client.payee_locations().list(&read_plan(plan)).await?),
            },
            PayeeLocationCommand::Show { id, plan } => value(
                client
                    .payee_locations()
                    .get(&read_plan(plan), id)
                    .await?
                    .payee_location,
            ),
        },
    }
}

async fn execute_month(command: MonthCommand, client: &Client) -> Result<Value, CliError> {
    match command {
        MonthCommand::List {
            plan,
            last_knowledge,
        } => value(
            client
                .months()
                .list(&read_plan(plan), last_knowledge.map(ServerKnowledge))
                .await?,
        ),
        MonthCommand::Show { month, plan } => {
            value(client.months().get(&read_plan(plan), month).await?.month)
        }
    }
}

async fn execute_money_movement(
    command: MoneyMovementCommand,
    client: &Client,
) -> Result<Value, CliError> {
    match command {
        MoneyMovementCommand::List { plan, month } => match month {
            Some(month) => value(
                client
                    .money_movements()
                    .list_for_month(&read_plan(plan), month)
                    .await?,
            ),
            None => value(client.money_movements().list(&read_plan(plan)).await?),
        },
    }
}

async fn execute_money_movement_group(
    command: MoneyMovementGroupCommand,
    client: &Client,
) -> Result<Value, CliError> {
    match command {
        MoneyMovementGroupCommand::List { plan, month } => match month {
            Some(month) => value(
                client
                    .money_movement_groups()
                    .list_for_month(&read_plan(plan), month)
                    .await?,
            ),
            None => value(
                client
                    .money_movement_groups()
                    .list(&read_plan(plan))
                    .await?,
            ),
        },
    }
}

async fn execute_transaction(
    command: TransactionCommand,
    client: &Client,
    input: &mut dyn Read,
) -> Result<Value, CliError> {
    match command {
        TransactionCommand::List(args) => execute_transaction_list(args, client).await,
        TransactionCommand::Show { id, plan } => value(
            client
                .transactions()
                .get(&read_plan(plan), &id)
                .await?
                .transaction,
        ),
        TransactionCommand::Create(args) => {
            let plan = args.plan.clone();
            let request = transaction_create(args, input)?;
            let response = client.transactions().create(&plan, &request).await?;
            if let Some(transaction) = response.transaction {
                value(transaction)
            } else if let Some(transactions) = response.transactions {
                value(transactions)
            } else {
                value(response)
            }
        }
        TransactionCommand::Update(args) => {
            let plan = args.plan.clone();
            let id = args.id.clone();
            let request = transaction_update(args, input)?;
            value(
                client
                    .transactions()
                    .update(&plan, &id, &request)
                    .await?
                    .transaction,
            )
        }
        TransactionCommand::BatchUpdate {
            plan,
            input: path,
            yes,
        } => {
            require_yes(yes, "transaction batch-update")?;
            let request: Vec<SaveTransactionWithIdOrImportId> = read_json(&path, input)?;
            value(client.transactions().update_bulk(&plan, &request).await?)
        }
        TransactionCommand::Import { plan, yes } => {
            require_yes(yes, "transaction import")?;
            value(client.transactions().import(&plan).await?)
        }
        TransactionCommand::Delete { id, plan, yes } => {
            require_yes(yes, "transaction delete")?;
            value(client.transactions().delete(&plan, &id).await?.transaction)
        }
    }
}

async fn execute_transaction_list(
    args: TransactionList,
    client: &Client,
) -> Result<Value, CliError> {
    let plan = read_plan(args.plan);
    let filters = TransactionFilters {
        since_date: args.since.map(|date| date.0),
        until_date: args.until.map(|date| date.0),
        kind: args.kind.map(transaction_type),
        last_knowledge: args.last_knowledge.map(ServerKnowledge),
    };
    match (args.account, args.category, args.payee, args.month) {
        (Some(id), None, None, None) => value(
            client
                .transactions()
                .list_for_account(&plan, id, &filters)
                .await?,
        ),
        (None, Some(id), None, None) => value(
            client
                .transactions()
                .list_for_category(&plan, id, &filters)
                .await?,
        ),
        (None, None, Some(id), None) => value(
            client
                .transactions()
                .list_for_payee(&plan, id, &filters)
                .await?,
        ),
        (None, None, None, Some(month)) => value(
            client
                .transactions()
                .list_for_month(&plan, month, &filters)
                .await?,
        ),
        (None, None, None, None) => value(client.transactions().list(&plan, &filters).await?),
        _ => Err(CliError::Invocation(
            "only one transaction scope may be supplied".into(),
        )),
    }
}

async fn execute_scheduled_transaction(
    command: ScheduledTransactionCommand,
    client: &Client,
) -> Result<Value, CliError> {
    match command {
        ScheduledTransactionCommand::List {
            plan,
            last_knowledge,
        } => value(
            client
                .scheduled_transactions()
                .list(&read_plan(plan), last_knowledge.map(ServerKnowledge))
                .await?,
        ),
        ScheduledTransactionCommand::Show { id, plan } => value(
            client
                .scheduled_transactions()
                .get(&read_plan(plan), id)
                .await?
                .scheduled_transaction,
        ),
        ScheduledTransactionCommand::Create(fields) => {
            let plan = fields.plan.clone();
            let request = scheduled_request(fields);
            value(
                client
                    .scheduled_transactions()
                    .create(&plan, &request)
                    .await?
                    .scheduled_transaction,
            )
        }
        ScheduledTransactionCommand::Update(ScheduledUpdate { id, fields }) => {
            let plan = fields.plan.clone();
            let request = scheduled_request(fields);
            value(
                client
                    .scheduled_transactions()
                    .update(&plan, id, &request)
                    .await?
                    .scheduled_transaction,
            )
        }
        ScheduledTransactionCommand::Delete { id, plan, yes } => {
            require_yes(yes, "scheduled-transaction delete")?;
            value(
                client
                    .scheduled_transactions()
                    .delete(&plan, id)
                    .await?
                    .scheduled_transaction,
            )
        }
    }
}

fn transaction_create(
    args: TransactionCreate,
    input: &mut dyn Read,
) -> Result<CreateTransactions, CliError> {
    if let Some(path) = args.input {
        return read_json(&path, input);
    }
    Ok(CreateTransactions::One(NewTransaction {
        transaction: create_fields(args.fields),
        import_id: Nullable::Omitted,
    }))
}

fn create_fields(fields: TransactionCreateFields) -> ExistingTransaction {
    ExistingTransaction {
        account_id: fields.account,
        date: fields.date.map(|date| date.0),
        amount: fields.amount.map(|amount| amount.0),
        payee_id: nullable(fields.payee_id),
        payee_name: nullable(fields.payee_name),
        category_id: nullable(fields.category),
        memo: nullable(fields.memo),
        cleared: fields.cleared.map(cleared_status),
        approved: fields.approved,
        flag_color: nullable(fields.flag.map(flag_color)),
        subtransactions: None,
    }
}

fn transaction_update(
    args: TransactionUpdate,
    input: &mut dyn Read,
) -> Result<ExistingTransaction, CliError> {
    if let Some(path) = args.input {
        return read_json(&path, input);
    }
    Ok(ExistingTransaction {
        account_id: args.account,
        date: args.date.map(|date| date.0),
        amount: args.amount.map(|amount| amount.0),
        payee_id: nullable(args.payee_id),
        payee_name: nullable(args.payee_name),
        category_id: nullable(args.category),
        memo: nullable(args.memo),
        cleared: args.cleared.map(cleared_status),
        approved: args.approved,
        flag_color: nullable(args.flag.map(flag_color)),
        subtransactions: None,
    })
}

fn scheduled_request(fields: ScheduledWrite) -> SaveScheduledTransaction {
    SaveScheduledTransaction {
        account_id: fields.account,
        date: fields.date.0,
        amount: fields.amount.map(|amount| amount.0),
        payee_id: nullable(fields.payee_id),
        payee_name: nullable(fields.payee_name),
        category_id: nullable(fields.category),
        memo: nullable(fields.memo),
        flag_color: nullable(fields.flag.map(flag_color)),
        frequency: fields.frequency.map(scheduled_frequency),
    }
}

fn nullable<T>(value: Option<T>) -> Nullable<T> {
    value.map_or(Nullable::Omitted, Nullable::Value)
}

fn read_json<T: DeserializeOwned>(path: &str, input: &mut dyn Read) -> Result<T, CliError> {
    let mut bytes = Vec::new();
    if path == "-" {
        input.read_to_end(&mut bytes)?;
    } else {
        std::fs::File::open(path)?.read_to_end(&mut bytes)?;
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| CliError::Input(format!("invalid input JSON: {error}")))
}

fn require_yes(value: bool, operation: &str) -> Result<(), CliError> {
    if value {
        Ok(())
    } else {
        Err(CliError::Invocation(format!("{operation} requires --yes")))
    }
}

fn read_plan(plan: Option<PlanId>) -> PlanId {
    plan.unwrap_or(PlanId::LastUsed)
}

fn value<T: Serialize>(value: T) -> Result<Value, CliError> {
    serde_json::to_value(value).map_err(|error| CliError::Local(error.to_string()))
}

fn save_account_type(value: SaveAccountTypeArg) -> SaveAccountType {
    let value = match value {
        SaveAccountTypeArg::Checking => SaveAccountType::CHECKING,
        SaveAccountTypeArg::Savings => SaveAccountType::SAVINGS,
        SaveAccountTypeArg::Cash => SaveAccountType::CASH,
        SaveAccountTypeArg::CreditCard => SaveAccountType::CREDIT_CARD,
        SaveAccountTypeArg::OtherAsset => SaveAccountType::OTHER_ASSET,
        SaveAccountTypeArg::OtherLiability => SaveAccountType::OTHER_LIABILITY,
    };
    SaveAccountType(value.into())
}

fn goal_frequency(value: GoalFrequencyArg) -> GoalFrequency {
    let value = match value {
        GoalFrequencyArg::Monthly => GoalFrequency::MONTHLY,
        GoalFrequencyArg::Weekly => GoalFrequency::WEEKLY,
        GoalFrequencyArg::Yearly => GoalFrequency::YEARLY,
    };
    GoalFrequency(value.into())
}

fn cleared_status(value: ClearedArg) -> TransactionClearedStatus {
    let value = match value {
        ClearedArg::Cleared => TransactionClearedStatus::CLEARED,
        ClearedArg::Uncleared => TransactionClearedStatus::UNCLEARED,
        ClearedArg::Reconciled => TransactionClearedStatus::RECONCILED,
    };
    TransactionClearedStatus(value.into())
}

fn flag_color(value: FlagArg) -> TransactionFlagColor {
    let value = match value {
        FlagArg::Red => TransactionFlagColor::RED,
        FlagArg::Orange => TransactionFlagColor::ORANGE,
        FlagArg::Yellow => TransactionFlagColor::YELLOW,
        FlagArg::Green => TransactionFlagColor::GREEN,
        FlagArg::Blue => TransactionFlagColor::BLUE,
        FlagArg::Purple => TransactionFlagColor::PURPLE,
    };
    TransactionFlagColor(value.into())
}

fn transaction_type(value: TransactionTypeArg) -> TransactionType {
    let value = match value {
        TransactionTypeArg::Uncategorized => TransactionType::UNCATEGORIZED,
        TransactionTypeArg::Unapproved => TransactionType::UNAPPROVED,
    };
    TransactionType(value.into())
}

fn scheduled_frequency(value: ScheduledFrequencyArg) -> ScheduledTransactionFrequency {
    let value = match value {
        ScheduledFrequencyArg::Never => ScheduledTransactionFrequency::NEVER,
        ScheduledFrequencyArg::Daily => ScheduledTransactionFrequency::DAILY,
        ScheduledFrequencyArg::Weekly => ScheduledTransactionFrequency::WEEKLY,
        ScheduledFrequencyArg::EveryOtherWeek => ScheduledTransactionFrequency::EVERY_OTHER_WEEK,
        ScheduledFrequencyArg::TwiceAMonth => ScheduledTransactionFrequency::TWICE_A_MONTH,
        ScheduledFrequencyArg::Every4Weeks => ScheduledTransactionFrequency::EVERY_FOUR_WEEKS,
        ScheduledFrequencyArg::Monthly => ScheduledTransactionFrequency::MONTHLY,
        ScheduledFrequencyArg::EveryOtherMonth => ScheduledTransactionFrequency::EVERY_OTHER_MONTH,
        ScheduledFrequencyArg::Every3Months => ScheduledTransactionFrequency::EVERY_THREE_MONTHS,
        ScheduledFrequencyArg::Every4Months => ScheduledTransactionFrequency::EVERY_FOUR_MONTHS,
        ScheduledFrequencyArg::TwiceAYear => ScheduledTransactionFrequency::TWICE_A_YEAR,
        ScheduledFrequencyArg::Yearly => ScheduledTransactionFrequency::YEARLY,
        ScheduledFrequencyArg::EveryOtherYear => ScheduledTransactionFrequency::EVERY_OTHER_YEAR,
    };
    ScheduledTransactionFrequency(value.into())
}
