use std::{fmt, str::FromStr};

use clap::{ArgAction, ArgGroup, Args, Parser, Subcommand, ValueEnum};
use rust_decimal::Decimal;
use time::Date;
use uuid::Uuid;
use ynab_sdk::{Milliunits, PlanId, PlanMonth};

#[derive(Parser)]
#[command(name = "ynab-cli", version, about = "JSON-first client for YNAB")]
pub struct Cli {
    #[arg(long, value_enum, default_value_t = Output::Json, global = true)]
    pub output: Output,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Output {
    Json,
    Table,
}

#[derive(Subcommand)]
pub enum Command {
    User(UserArgs),
    Plan(PlanArgs),
    Account(AccountArgs),
    Category(CategoryArgs),
    CategoryGroup(CategoryGroupArgs),
    Payee(PayeeArgs),
    Month(MonthArgs),
    MoneyMovement(MoneyMovementArgs),
    MoneyMovementGroup(MoneyMovementGroupArgs),
    Transaction(TransactionArgs),
    ScheduledTransaction(ScheduledTransactionArgs),
    Auth(AuthArgs),
}

#[derive(Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCommand,
}

#[derive(Subcommand)]
pub enum UserCommand {
    Show,
}

#[derive(Args)]
pub struct PlanArgs {
    #[command(subcommand)]
    pub command: PlanCommand,
}

#[derive(Subcommand)]
pub enum PlanCommand {
    List {
        #[arg(long)]
        include_accounts: bool,
    },
    Show {
        #[arg(long)]
        plan: Option<PlanId>,
        #[arg(long)]
        last_knowledge: Option<u64>,
    },
    Settings {
        #[arg(long)]
        plan: Option<PlanId>,
    },
}

#[derive(Args)]
pub struct AccountArgs {
    #[command(subcommand)]
    pub command: AccountCommand,
}

#[derive(Subcommand)]
pub enum AccountCommand {
    List {
        #[arg(long)]
        plan: Option<PlanId>,
        #[arg(long)]
        last_knowledge: Option<u64>,
    },
    Show {
        id: Uuid,
        #[arg(long)]
        plan: Option<PlanId>,
    },
    Create {
        #[arg(long)]
        plan: PlanId,
        #[arg(long)]
        name: String,
        #[arg(long = "type", value_enum)]
        account_type: SaveAccountTypeArg,
        #[arg(long, allow_hyphen_values = true)]
        balance: CliAmount,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SaveAccountTypeArg {
    Checking,
    Savings,
    Cash,
    CreditCard,
    OtherAsset,
    OtherLiability,
}

#[derive(Args)]
pub struct CategoryArgs {
    #[command(subcommand)]
    pub command: CategoryCommand,
}

#[derive(Subcommand)]
pub enum CategoryCommand {
    List {
        #[arg(long)]
        plan: Option<PlanId>,
        #[arg(long)]
        last_knowledge: Option<u64>,
    },
    Show {
        id: Uuid,
        #[arg(long)]
        plan: Option<PlanId>,
    },
    Create(CategoryCreate),
    Update(CategoryUpdate),
    Month(CategoryMonthArgs),
}

#[derive(Args)]
pub struct CategoryCreate {
    #[arg(long)]
    pub plan: PlanId,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub category_group: Uuid,
    #[arg(long)]
    pub note: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    pub goal_target: Option<CliAmount>,
    #[arg(long)]
    pub goal_target_date: Option<CliDate>,
    #[arg(long, action = ArgAction::Set)]
    pub goal_needs_whole_amount: Option<bool>,
    #[arg(long, value_enum)]
    pub goal_frequency: Option<GoalFrequencyArg>,
}

#[derive(Args)]
#[command(group(ArgGroup::new("update").required(true).multiple(true).args([
    "name", "note", "category_group", "goal_target", "goal_target_date",
    "goal_needs_whole_amount", "goal_frequency"
])))]
pub struct CategoryUpdate {
    pub id: Uuid,
    #[arg(long)]
    pub plan: PlanId,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub note: Option<String>,
    #[arg(long)]
    pub category_group: Option<Uuid>,
    #[arg(long, allow_hyphen_values = true)]
    pub goal_target: Option<CliAmount>,
    #[arg(long)]
    pub goal_target_date: Option<CliDate>,
    #[arg(long, action = ArgAction::Set)]
    pub goal_needs_whole_amount: Option<bool>,
    #[arg(long, value_enum)]
    pub goal_frequency: Option<GoalFrequencyArg>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum GoalFrequencyArg {
    Monthly,
    Weekly,
    Yearly,
}

#[derive(Args)]
pub struct CategoryMonthArgs {
    #[command(subcommand)]
    pub command: CategoryMonthCommand,
}

#[derive(Subcommand)]
pub enum CategoryMonthCommand {
    Show {
        month: PlanMonth,
        id: Uuid,
        #[arg(long)]
        plan: Option<PlanId>,
    },
    Assign {
        month: PlanMonth,
        id: Uuid,
        #[arg(long)]
        plan: PlanId,
        #[arg(long, allow_hyphen_values = true)]
        amount: CliAmount,
    },
}

#[derive(Args)]
pub struct CategoryGroupArgs {
    #[command(subcommand)]
    pub command: CategoryGroupCommand,
}

#[derive(Subcommand)]
pub enum CategoryGroupCommand {
    Create {
        #[arg(long)]
        plan: PlanId,
        #[arg(long)]
        name: String,
    },
    Update {
        id: Uuid,
        #[arg(long)]
        plan: PlanId,
        #[arg(long)]
        name: String,
    },
}

#[derive(Args)]
pub struct PayeeArgs {
    #[command(subcommand)]
    pub command: PayeeCommand,
}

#[derive(Subcommand)]
pub enum PayeeCommand {
    List {
        #[arg(long)]
        plan: Option<PlanId>,
        #[arg(long)]
        last_knowledge: Option<u64>,
    },
    Show {
        id: Uuid,
        #[arg(long)]
        plan: Option<PlanId>,
    },
    Create {
        #[arg(long)]
        plan: PlanId,
        #[arg(long)]
        name: String,
    },
    Update {
        id: Uuid,
        #[arg(long)]
        plan: PlanId,
        #[arg(long)]
        name: String,
    },
    Location(PayeeLocationArgs),
}

#[derive(Args)]
pub struct PayeeLocationArgs {
    #[command(subcommand)]
    pub command: PayeeLocationCommand,
}

#[derive(Subcommand)]
pub enum PayeeLocationCommand {
    List {
        #[arg(long)]
        plan: Option<PlanId>,
        #[arg(long)]
        payee: Option<Uuid>,
    },
    Show {
        id: Uuid,
        #[arg(long)]
        plan: Option<PlanId>,
    },
}

#[derive(Args)]
pub struct MonthArgs {
    #[command(subcommand)]
    pub command: MonthCommand,
}

#[derive(Subcommand)]
pub enum MonthCommand {
    List {
        #[arg(long)]
        plan: Option<PlanId>,
        #[arg(long)]
        last_knowledge: Option<u64>,
    },
    Show {
        month: PlanMonth,
        #[arg(long)]
        plan: Option<PlanId>,
    },
}

#[derive(Args)]
pub struct MoneyMovementArgs {
    #[command(subcommand)]
    pub command: MoneyMovementCommand,
}

#[derive(Subcommand)]
pub enum MoneyMovementCommand {
    List {
        #[arg(long)]
        plan: Option<PlanId>,
        #[arg(long)]
        month: Option<PlanMonth>,
    },
}

#[derive(Args)]
pub struct MoneyMovementGroupArgs {
    #[command(subcommand)]
    pub command: MoneyMovementGroupCommand,
}

#[derive(Subcommand)]
pub enum MoneyMovementGroupCommand {
    List {
        #[arg(long)]
        plan: Option<PlanId>,
        #[arg(long)]
        month: Option<PlanMonth>,
    },
}

#[derive(Args)]
pub struct TransactionArgs {
    #[command(subcommand)]
    pub command: TransactionCommand,
}

#[derive(Subcommand)]
pub enum TransactionCommand {
    List(TransactionList),
    Show {
        id: String,
        #[arg(long)]
        plan: Option<PlanId>,
    },
    Create(TransactionCreate),
    Update(TransactionUpdate),
    BatchUpdate {
        #[arg(long)]
        plan: PlanId,
        #[arg(long)]
        input: String,
        #[arg(long, required = true)]
        yes: bool,
    },
    Import {
        #[arg(long)]
        plan: PlanId,
        #[arg(long, required = true)]
        yes: bool,
    },
    Delete {
        id: String,
        #[arg(long)]
        plan: PlanId,
        #[arg(long, required = true)]
        yes: bool,
    },
}

#[derive(Args)]
#[command(group(ArgGroup::new("scope").multiple(false).args(["account", "category", "payee", "month"])))]
pub struct TransactionList {
    #[arg(long)]
    pub plan: Option<PlanId>,
    #[arg(long)]
    pub account: Option<Uuid>,
    #[arg(long)]
    pub category: Option<Uuid>,
    #[arg(long)]
    pub payee: Option<Uuid>,
    #[arg(long)]
    pub month: Option<PlanMonth>,
    #[arg(long)]
    pub since: Option<CliDate>,
    #[arg(long)]
    pub until: Option<CliDate>,
    #[arg(long = "type", value_enum)]
    pub kind: Option<TransactionTypeArg>,
    #[arg(long)]
    pub last_knowledge: Option<u64>,
}

const TRANSACTION_FIELDS: [&str; 10] = [
    "account",
    "date",
    "amount",
    "payee_id",
    "payee_name",
    "category",
    "memo",
    "cleared",
    "approved",
    "flag",
];

#[derive(Args)]
pub struct TransactionCreate {
    #[arg(long)]
    pub plan: PlanId,
    #[arg(long, conflicts_with_all = TRANSACTION_FIELDS)]
    pub input: Option<String>,
    #[command(flatten)]
    pub fields: TransactionCreateFields,
}

#[derive(Args)]
pub struct TransactionCreateFields {
    #[arg(long, required_unless_present = "input")]
    pub account: Option<Uuid>,
    #[arg(long, required_unless_present = "input")]
    pub date: Option<CliDate>,
    #[arg(long, required_unless_present = "input", allow_hyphen_values = true)]
    pub amount: Option<CliAmount>,
    #[arg(long, conflicts_with = "payee_name")]
    pub payee_id: Option<Uuid>,
    #[arg(long)]
    pub payee_name: Option<String>,
    #[arg(long)]
    pub category: Option<Uuid>,
    #[arg(long)]
    pub memo: Option<String>,
    #[arg(long, value_enum)]
    pub cleared: Option<ClearedArg>,
    #[arg(long, action = ArgAction::Set)]
    pub approved: Option<bool>,
    #[arg(long, value_enum)]
    pub flag: Option<FlagArg>,
}

#[derive(Args)]
#[command(group(ArgGroup::new("source").required(true).multiple(true).args([
    "input", "account", "date", "amount", "payee_id", "payee_name",
    "category", "memo", "cleared", "approved", "flag"
])))]
pub struct TransactionUpdate {
    pub id: String,
    #[arg(long)]
    pub plan: PlanId,
    #[arg(long, conflicts_with_all = TRANSACTION_FIELDS)]
    pub input: Option<String>,
    #[arg(long)]
    pub account: Option<Uuid>,
    #[arg(long)]
    pub date: Option<CliDate>,
    #[arg(long, allow_hyphen_values = true)]
    pub amount: Option<CliAmount>,
    #[arg(long, conflicts_with = "payee_name")]
    pub payee_id: Option<Uuid>,
    #[arg(long)]
    pub payee_name: Option<String>,
    #[arg(long)]
    pub category: Option<Uuid>,
    #[arg(long)]
    pub memo: Option<String>,
    #[arg(long, value_enum)]
    pub cleared: Option<ClearedArg>,
    #[arg(long, action = ArgAction::Set)]
    pub approved: Option<bool>,
    #[arg(long, value_enum)]
    pub flag: Option<FlagArg>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ClearedArg {
    Cleared,
    Uncleared,
    Reconciled,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FlagArg {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TransactionTypeArg {
    Uncategorized,
    Unapproved,
}

#[derive(Args)]
pub struct ScheduledTransactionArgs {
    #[command(subcommand)]
    pub command: ScheduledTransactionCommand,
}

#[derive(Subcommand)]
pub enum ScheduledTransactionCommand {
    List {
        #[arg(long)]
        plan: Option<PlanId>,
        #[arg(long)]
        last_knowledge: Option<u64>,
    },
    Show {
        id: Uuid,
        #[arg(long)]
        plan: Option<PlanId>,
    },
    Create(ScheduledWrite),
    Update(ScheduledUpdate),
    Delete {
        id: Uuid,
        #[arg(long)]
        plan: PlanId,
        #[arg(long, required = true)]
        yes: bool,
    },
}

#[derive(Args)]
pub struct ScheduledWrite {
    #[arg(long)]
    pub plan: PlanId,
    #[arg(long)]
    pub account: Uuid,
    #[arg(long)]
    pub date: CliDate,
    #[arg(long, allow_hyphen_values = true)]
    pub amount: Option<CliAmount>,
    #[arg(long, conflicts_with = "payee_name")]
    pub payee_id: Option<Uuid>,
    #[arg(long)]
    pub payee_name: Option<String>,
    #[arg(long)]
    pub category: Option<Uuid>,
    #[arg(long)]
    pub memo: Option<String>,
    #[arg(long, value_enum)]
    pub flag: Option<FlagArg>,
    #[arg(long, value_enum)]
    pub frequency: Option<ScheduledFrequencyArg>,
}

#[derive(Args)]
pub struct ScheduledUpdate {
    pub id: Uuid,
    #[command(flatten)]
    pub fields: ScheduledWrite,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ScheduledFrequencyArg {
    Never,
    Daily,
    Weekly,
    EveryOtherWeek,
    TwiceAMonth,
    Every4Weeks,
    Monthly,
    EveryOtherMonth,
    Every3Months,
    Every4Months,
    TwiceAYear,
    Yearly,
    EveryOtherYear,
}

#[derive(Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Subcommand)]
pub enum AuthCommand {
    Store,
    Clear,
    Status,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliAmount(pub Milliunits);

impl FromStr for CliAmount {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let decimal =
            Decimal::from_str(value).map_err(|_| "amount must be a decimal value".to_owned())?;
        Milliunits::from_decimal(decimal).map(Self).ok_or_else(|| {
            "amount must have at most three fractional digits and fit in i64 milliunits".to_owned()
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliDate(pub Date);

impl FromStr for CliDate {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Date::parse(value, &time::format_description::well_known::Iso8601::DATE)
            .map(Self)
            .map_err(|error| error.to_string())
    }
}

impl fmt::Display for CliDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
