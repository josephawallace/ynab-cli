//! Public scalar, request, and response models for the YNAB v1 API.

use std::{collections::BTreeMap, fmt, str::FromStr};

use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PlanId {
    Id(Uuid),
    LastUsed,
    Default,
}

impl fmt::Display for PlanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::LastUsed => f.write_str("last-used"),
            Self::Default => f.write_str("default"),
        }
    }
}

impl FromStr for PlanId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "last-used" => Ok(Self::LastUsed),
            "default" => Ok(Self::Default),
            _ => Ok(Self::Id(Uuid::parse_str(value)?)),
        }
    }
}

impl Serialize for PlanId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PlanId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ServerKnowledge(pub u64);

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Milliunits(pub i64);

impl Milliunits {
    #[must_use]
    pub fn from_decimal(decimal: Decimal) -> Option<Self> {
        match decimal.checked_mul(Decimal::new(1000, 0)) {
            Some(value) if value.fract().is_zero() => value.to_i64().map(Self),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_decimal(self) -> Decimal {
        Decimal::new(self.0, 3)
    }
}

#[derive(Debug, Error, Clone, Copy, Eq, PartialEq)]
#[error("month must be the first day of its UTC month")]
pub struct PlanMonthError;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlanMonth(Date);

impl PlanMonth {
    pub fn new(date: Date) -> Result<Self, PlanMonthError> {
        if date.day() == 1 {
            Ok(Self(date))
        } else {
            Err(PlanMonthError)
        }
    }

    #[must_use]
    pub const fn date(self) -> Date {
        self.0
    }
}

impl fmt::Display for PlanMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PlanMonth {
    type Err = PlanMonthError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Date::parse(value, &time::format_description::well_known::Iso8601::DATE)
            .map_err(|_| PlanMonthError)
            .and_then(Self::new)
    }
}

impl Serialize for PlanMonth {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PlanMonth {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Nullable<T> {
    #[default]
    Omitted,
    Null,
    Value(T),
}

impl<T> Nullable<T> {
    #[must_use]
    pub const fn is_omitted(&self) -> bool {
        matches!(self, Self::Omitted)
    }

    #[must_use]
    pub const fn has_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }
}

impl<T: Serialize> Serialize for Nullable<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Omitted | Self::Null => serializer.serialize_none(),
            Self::Value(value) => value.serialize(serializer),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Nullable<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(|value| match value {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

macro_rules! open_string {
    ($name:ident, $($constant:ident = $value:literal),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            $(pub const $constant: &'static str = $value;)+

            #[must_use]
            pub fn is_known(&self) -> bool {
                matches!(self.0.as_str(), $($value)|+)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self { Self(value) }
        }

        impl FromStr for $name {
            type Err = std::convert::Infallible;
            fn from_str(value: &str) -> Result<Self, Self::Err> { Ok(Self(value.to_owned())) }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
        }
    };
}

open_string!(
    AccountType,
    CHECKING = "checking",
    SAVINGS = "savings",
    CASH = "cash",
    CREDIT_CARD = "creditCard",
    LINE_OF_CREDIT = "lineOfCredit",
    OTHER_ASSET = "otherAsset",
    OTHER_LIABILITY = "otherLiability",
    MORTGAGE = "mortgage",
    AUTO_LOAN = "autoLoan",
    STUDENT_LOAN = "studentLoan",
    PERSONAL_LOAN = "personalLoan",
    MEDICAL_DEBT = "medicalDebt",
    OTHER_DEBT = "otherDebt",
);
open_string!(
    SaveAccountType,
    CHECKING = "checking",
    SAVINGS = "savings",
    CASH = "cash",
    CREDIT_CARD = "creditCard",
    OTHER_ASSET = "otherAsset",
    OTHER_LIABILITY = "otherLiability",
);
open_string!(
    TransactionClearedStatus,
    CLEARED = "cleared",
    UNCLEARED = "uncleared",
    RECONCILED = "reconciled",
);
open_string!(
    TransactionFlagColor,
    RED = "red",
    ORANGE = "orange",
    YELLOW = "yellow",
    GREEN = "green",
    BLUE = "blue",
    PURPLE = "purple",
);
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TransactionFlagName(pub String);
open_string!(
    ScheduledTransactionFrequency,
    NEVER = "never",
    DAILY = "daily",
    WEEKLY = "weekly",
    EVERY_OTHER_WEEK = "everyOtherWeek",
    TWICE_A_MONTH = "twiceAMonth",
    EVERY_FOUR_WEEKS = "every4Weeks",
    MONTHLY = "monthly",
    EVERY_OTHER_MONTH = "everyOtherMonth",
    EVERY_THREE_MONTHS = "every3Months",
    EVERY_FOUR_MONTHS = "every4Months",
    TWICE_A_YEAR = "twiceAYear",
    YEARLY = "yearly",
    EVERY_OTHER_YEAR = "everyOtherYear",
);
open_string!(
    GoalType,
    TARGET_BALANCE = "TB",
    TARGET_BY_DATE = "TBD",
    MONTHLY_FUNDING = "MF",
    NEED = "NEED",
    DEBT = "DEBT"
);
open_string!(
    GoalFrequency,
    MONTHLY = "monthly",
    WEEKLY = "weekly",
    YEARLY = "yearly"
);
open_string!(
    DebtTransactionType,
    PAYMENT = "payment",
    REFUND = "refund",
    FEE = "fee",
    INTEREST = "interest",
    ESCROW = "escrow",
    BALANCE_ADJUSTMENT = "balanceAdjustment",
    CREDIT = "credit",
    CHARGE = "charge"
);
open_string!(
    HybridTransactionType,
    TRANSACTION = "transaction",
    SUBTRANSACTION = "subtransaction"
);
open_string!(
    TransactionType,
    UNCATEGORIZED = "uncategorized",
    UNAPPROVED = "unapproved"
);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    pub data: T,
}

pub(crate) type DataEnvelope<T> = SuccessResponse<T>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub id: String,
    pub name: String,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPayload {
    pub user: User,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DateFormat {
    pub format: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurrencyFormat {
    pub iso_code: String,
    pub example_format: String,
    pub decimal_digits: u8,
    pub decimal_separator: String,
    pub symbol_first: bool,
    pub group_separator: String,
    pub currency_symbol: String,
    pub display_symbol: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanSummary {
    pub id: Uuid,
    pub name: String,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_modified_on: Option<OffsetDateTime>,
    pub first_month: Option<Date>,
    pub last_month: Option<Date>,
    pub date_format: Option<DateFormat>,
    pub currency_format: Option<CurrencyFormat>,
    pub accounts: Option<Vec<Account>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlansPayload {
    pub plans: Vec<PlanSummary>,
    pub default_plan: Option<PlanSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanSettings {
    pub date_format: DateFormat,
    pub currency_format: CurrencyFormat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanSettingsPayload {
    pub settings: PlanSettings,
}

pub type LoanAccountPeriodicValue = BTreeMap<String, Milliunits>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountBase {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    pub balance: Milliunits,
    pub cleared_balance: Milliunits,
    pub uncleared_balance: Milliunits,
    pub transfer_payee_id: Option<Uuid>,
    pub direct_import_linked: Option<bool>,
    pub direct_import_in_error: Option<bool>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_reconciled_at: Option<OffsetDateTime>,
    pub debt_original_balance: Option<Milliunits>,
    pub debt_interest_rates: Option<LoanAccountPeriodicValue>,
    pub debt_minimum_payments: Option<LoanAccountPeriodicValue>,
    pub debt_escrow_amounts: Option<LoanAccountPeriodicValue>,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(flatten)]
    pub base: AccountBase,
    pub balance_formatted: Option<String>,
    pub balance_currency: Option<Decimal>,
    pub cleared_balance_formatted: Option<String>,
    pub cleared_balance_currency: Option<Decimal>,
    pub uncleared_balance_formatted: Option<String>,
    pub uncleared_balance_currency: Option<Decimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveAccount {
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: SaveAccountType,
    pub balance: Milliunits,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountPayload {
    pub account: Account,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountsPayload {
    pub accounts: Vec<Account>,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryGroup {
    pub id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub internal: bool,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryBase {
    pub id: Uuid,
    pub category_group_id: Uuid,
    pub category_group_name: Option<String>,
    pub name: String,
    pub hidden: bool,
    pub internal: bool,
    pub original_category_group_id: Option<Uuid>,
    pub note: Option<String>,
    pub budgeted: Milliunits,
    pub activity: Milliunits,
    pub balance: Milliunits,
    pub goal_type: Option<GoalType>,
    pub goal_needs_whole_amount: Option<bool>,
    pub goal_day: Option<u32>,
    pub goal_cadence: Option<u32>,
    pub goal_cadence_frequency: Option<u32>,
    pub goal_creation_month: Option<PlanMonth>,
    pub goal_target: Option<Milliunits>,
    pub goal_target_month: Option<PlanMonth>,
    pub goal_target_date: Option<Date>,
    pub goal_percentage_complete: Option<u32>,
    pub goal_months_to_budget: Option<u32>,
    pub goal_under_funded: Option<Milliunits>,
    pub goal_overall_funded: Option<Milliunits>,
    pub goal_overall_left: Option<Milliunits>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub goal_snoozed_at: Option<OffsetDateTime>,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Category {
    #[serde(flatten)]
    pub base: CategoryBase,
    pub balance_formatted: Option<String>,
    pub balance_currency: Option<Decimal>,
    pub activity_formatted: Option<String>,
    pub activity_currency: Option<Decimal>,
    pub budgeted_formatted: Option<String>,
    pub budgeted_currency: Option<Decimal>,
    pub goal_target_formatted: Option<String>,
    pub goal_target_currency: Option<Decimal>,
    pub goal_under_funded_formatted: Option<String>,
    pub goal_under_funded_currency: Option<Decimal>,
    pub goal_overall_funded_formatted: Option<String>,
    pub goal_overall_funded_currency: Option<Decimal>,
    pub goal_overall_left_formatted: Option<String>,
    pub goal_overall_left_currency: Option<Decimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryGroupWithCategories {
    #[serde(flatten)]
    pub group: CategoryGroup,
    pub categories: Vec<Category>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoriesPayload {
    pub category_groups: Vec<CategoryGroupWithCategories>,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryPayload {
    pub category: Category,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveCategoryPayload {
    pub category: Category,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveCategoryGroupPayload {
    pub category_group: CategoryGroup,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveCategoryGroup {
    pub name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SaveCategory {
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub name: Nullable<String>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub note: Nullable<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_group_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub goal_target: Nullable<Milliunits>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub goal_target_date: Nullable<Date>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub goal_needs_whole_amount: Nullable<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_frequency: Option<GoalFrequency>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewCategory {
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub note: Nullable<String>,
    pub category_group_id: Uuid,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub goal_target: Nullable<Milliunits>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub goal_target_date: Nullable<Date>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub goal_needs_whole_amount: Nullable<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_frequency: Option<GoalFrequency>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveMonthCategory {
    pub budgeted: Milliunits,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payee {
    pub id: Uuid,
    pub name: String,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostPayee {
    pub name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SavePayee {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PayeesPayload {
    pub payees: Vec<Payee>,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PayeePayload {
    pub payee: Payee,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavePayeePayload {
    pub payee: Payee,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PayeeLocation {
    pub id: Uuid,
    pub payee_id: Uuid,
    pub latitude: String,
    pub longitude: String,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PayeeLocationsPayload {
    pub payee_locations: Vec<PayeeLocation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PayeeLocationPayload {
    pub payee_location: PayeeLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveSubTransaction {
    pub amount: Milliunits,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub payee_id: Nullable<Uuid>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub payee_name: Nullable<String>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub category_id: Nullable<Uuid>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub memo: Nullable<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExistingTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Date>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Milliunits>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub payee_id: Nullable<Uuid>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub payee_name: Nullable<String>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub category_id: Nullable<Uuid>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub memo: Nullable<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared: Option<TransactionClearedStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub flag_color: Nullable<TransactionFlagColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

impl ExistingTransaction {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.payee_id.has_value() && self.payee_name.has_value() {
            Err(ValidationError::PayeeConflict)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NewTransaction {
    #[serde(flatten)]
    pub transaction: ExistingTransaction,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub import_id: Nullable<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SaveTransactionWithIdOrImportId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_id: Option<String>,
    #[serde(flatten)]
    pub transaction: ExistingTransaction,
}

impl SaveTransactionWithIdOrImportId {
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.transaction.validate()?;
        if self.id.is_some() == self.import_id.is_some() {
            Err(ValidationError::BulkIdentity)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub enum CreateTransactions {
    One(NewTransaction),
    Many(Vec<NewTransaction>),
}

impl CreateTransactions {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Self::One(transaction) => transaction.transaction.validate(),
            Self::Many(transactions) if transactions.is_empty() => {
                Err(ValidationError::EmptyTransactions)
            }
            Self::Many(transactions) => transactions
                .iter()
                .try_for_each(|transaction| transaction.transaction.validate()),
        }
    }
}

impl Serialize for CreateTransactions {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Self::One(transaction) => map.serialize_entry("transaction", transaction)?,
            Self::Many(transactions) => map.serialize_entry("transactions", transactions)?,
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for CreateTransactions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Input {
            transaction: Option<NewTransaction>,
            transactions: Option<Vec<NewTransaction>>,
        }
        let input = Input::deserialize(deserializer)?;
        match (input.transaction, input.transactions) {
            (Some(transaction), None) => Ok(Self::One(transaction)),
            (None, Some(transactions)) if !transactions.is_empty() => Ok(Self::Many(transactions)),
            _ => Err(serde::de::Error::custom(
                ValidationError::TransactionVariant,
            )),
        }
    }
}

#[derive(Debug, Error, Clone, Copy, Eq, PartialEq)]
pub enum ValidationError {
    #[error("payee_id and payee_name cannot both be supplied")]
    PayeeConflict,
    #[error("transaction collection cannot be empty")]
    EmptyTransactions,
    #[error("a transaction wrapper needs exactly one nonempty transaction variant")]
    TransactionVariant,
    #[error("a bulk item needs exactly one of id or import_id")]
    BulkIdentity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionSummaryBase {
    pub id: Uuid,
    pub date: Date,
    pub amount: Milliunits,
    pub memo: Option<String>,
    pub cleared: TransactionClearedStatus,
    pub approved: bool,
    pub flag_color: Option<TransactionFlagColor>,
    pub flag_name: Option<TransactionFlagName>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub matched_transaction_id: Option<Uuid>,
    pub import_id: Option<String>,
    pub import_payee_name: Option<String>,
    pub import_payee_name_original: Option<String>,
    pub debt_transaction_type: Option<DebtTransactionType>,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionSummary {
    #[serde(flatten)]
    pub base: TransactionSummaryBase,
    pub amount_formatted: Option<String>,
    pub amount_currency: Option<Decimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubTransactionBase {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub amount: Milliunits,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubTransaction {
    #[serde(flatten)]
    pub base: SubTransactionBase,
    pub amount_formatted: Option<String>,
    pub amount_currency: Option<Decimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionDetail {
    #[serde(flatten)]
    pub summary: TransactionSummary,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<SubTransaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridTransaction {
    #[serde(flatten)]
    pub summary: TransactionSummary,
    #[serde(rename = "type")]
    pub transaction_type: HybridTransactionType,
    pub parent_transaction_id: Option<Uuid>,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct TransactionFilters {
    pub since_date: Option<Date>,
    pub until_date: Option<Date>,
    pub kind: Option<TransactionType>,
    pub last_knowledge: Option<ServerKnowledge>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionsPayload {
    pub transactions: Vec<TransactionDetail>,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridTransactionsPayload {
    pub transactions: Vec<HybridTransaction>,
    pub server_knowledge: Option<ServerKnowledge>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPayload {
    pub transaction: TransactionDetail,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveTransactionsPayload {
    pub transaction_ids: Vec<Uuid>,
    pub transaction: Option<TransactionDetail>,
    pub transactions: Option<Vec<TransactionDetail>>,
    pub duplicate_import_ids: Option<Vec<String>>,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportPayload {
    pub transaction_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BulkTransactions {
    pub transaction_ids: Vec<Uuid>,
    pub duplicate_import_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BulkPayload {
    pub bulk: BulkTransactions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveScheduledTransaction {
    pub account_id: Uuid,
    pub date: Date,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Milliunits>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub payee_id: Nullable<Uuid>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub payee_name: Nullable<String>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub category_id: Nullable<Uuid>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub memo: Nullable<String>,
    #[serde(default, skip_serializing_if = "Nullable::is_omitted")]
    pub flag_color: Nullable<TransactionFlagColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<ScheduledTransactionFrequency>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTransactionSummaryBase {
    pub id: Uuid,
    pub date_first: Date,
    pub date_next: Date,
    pub frequency: ScheduledTransactionFrequency,
    pub amount: Milliunits,
    pub memo: Option<String>,
    pub flag_color: Option<TransactionFlagColor>,
    pub flag_name: Option<TransactionFlagName>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTransactionSummary {
    #[serde(flatten)]
    pub base: ScheduledTransactionSummaryBase,
    pub amount_formatted: Option<String>,
    pub amount_currency: Option<Decimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledSubTransactionBase {
    pub id: Uuid,
    pub scheduled_transaction_id: Uuid,
    pub amount: Milliunits,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledSubTransaction {
    #[serde(flatten)]
    pub base: ScheduledSubTransactionBase,
    pub amount_formatted: Option<String>,
    pub amount_currency: Option<Decimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTransactionDetail {
    #[serde(flatten)]
    pub summary: ScheduledTransactionSummary,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<ScheduledSubTransaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTransactionsPayload {
    pub scheduled_transactions: Vec<ScheduledTransactionDetail>,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTransactionPayload {
    pub scheduled_transaction: ScheduledTransactionDetail,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonthSummaryBase {
    pub month: PlanMonth,
    pub note: Option<String>,
    pub income: Milliunits,
    pub budgeted: Milliunits,
    pub activity: Milliunits,
    pub to_be_budgeted: Milliunits,
    pub age_of_money: Option<u32>,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonthSummary {
    #[serde(flatten)]
    pub base: MonthSummaryBase,
    pub income_formatted: Option<String>,
    pub income_currency: Option<Decimal>,
    pub budgeted_formatted: Option<String>,
    pub budgeted_currency: Option<Decimal>,
    pub activity_formatted: Option<String>,
    pub activity_currency: Option<Decimal>,
    pub to_be_budgeted_formatted: Option<String>,
    pub to_be_budgeted_currency: Option<Decimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonthDetailBase {
    #[serde(flatten)]
    pub summary: MonthSummaryBase,
    pub categories: Vec<CategoryBase>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonthDetail {
    #[serde(flatten)]
    pub summary: MonthSummary,
    pub categories: Vec<Category>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonthsPayload {
    pub months: Vec<MonthSummary>,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonthPayload {
    pub month: MonthDetail,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoneyMovementBase {
    pub id: Uuid,
    pub month: Option<PlanMonth>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub moved_at: Option<OffsetDateTime>,
    pub note: Option<String>,
    pub money_movement_group_id: Option<Uuid>,
    pub performed_by_user_id: Option<Uuid>,
    pub from_category_id: Option<Uuid>,
    pub to_category_id: Option<Uuid>,
    pub amount: Milliunits,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoneyMovement {
    #[serde(flatten)]
    pub base: MoneyMovementBase,
    pub amount_formatted: Option<String>,
    pub amount_currency: Option<Decimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoneyMovementGroup {
    pub id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub group_created_at: OffsetDateTime,
    pub month: PlanMonth,
    pub note: Option<String>,
    pub performed_by_user_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoneyMovementsPayload {
    pub money_movements: Vec<MoneyMovement>,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoneyMovementGroupsPayload {
    pub money_movement_groups: Vec<MoneyMovementGroup>,
    pub server_knowledge: ServerKnowledge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanDetail {
    pub id: Uuid,
    pub name: String,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_modified_on: Option<OffsetDateTime>,
    pub first_month: Option<Date>,
    pub last_month: Option<Date>,
    pub date_format: Option<DateFormat>,
    pub currency_format: Option<CurrencyFormat>,
    pub accounts: Option<Vec<AccountBase>>,
    pub payees: Option<Vec<Payee>>,
    pub payee_locations: Option<Vec<PayeeLocation>>,
    pub category_groups: Option<Vec<CategoryGroup>>,
    pub categories: Option<Vec<CategoryBase>>,
    pub months: Option<Vec<MonthDetailBase>>,
    pub transactions: Option<Vec<TransactionSummaryBase>>,
    pub subtransactions: Option<Vec<SubTransactionBase>>,
    pub scheduled_transactions: Option<Vec<ScheduledTransactionSummaryBase>>,
    pub scheduled_subtransactions: Option<Vec<ScheduledSubTransactionBase>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanDetailPayload {
    pub plan: PlanDetail,
    pub server_knowledge: ServerKnowledge,
}

pub type ExistingCategory = SaveCategory;
pub type SaveTransactionWithOptionalFields = ExistingTransaction;

#[derive(Clone, Debug, Serialize)]
pub struct PostAccountWrapper<'a> {
    pub account: &'a SaveAccount,
}

#[derive(Clone, Debug, Serialize)]
pub struct PostCategoryGroupWrapper<'a> {
    pub category_group: &'a SaveCategoryGroup,
}

pub type PatchCategoryGroupWrapper<'a> = PostCategoryGroupWrapper<'a>;

#[derive(Clone, Debug, Serialize)]
pub struct PostCategoryWrapper<'a> {
    pub category: &'a NewCategory,
}

#[derive(Clone, Debug, Serialize)]
pub struct PatchCategoryWrapper<'a> {
    pub category: &'a ExistingCategory,
}

#[derive(Clone, Debug, Serialize)]
pub struct PatchMonthCategoryWrapper<'a> {
    pub category: &'a SaveMonthCategory,
}

#[derive(Clone, Debug, Serialize)]
pub struct PostPayeeWrapper<'a> {
    pub payee: &'a PostPayee,
}

#[derive(Clone, Debug, Serialize)]
pub struct PatchPayeeWrapper<'a> {
    pub payee: &'a SavePayee,
}

pub type PostTransactionsWrapper = CreateTransactions;

#[derive(Clone, Debug, Serialize)]
pub struct PatchTransactionsWrapper<'a> {
    pub transactions: &'a [SaveTransactionWithIdOrImportId],
}

#[derive(Clone, Debug, Serialize)]
pub struct PutTransactionWrapper<'a> {
    pub transaction: &'a ExistingTransaction,
}

#[derive(Clone, Debug, Serialize)]
pub struct PostScheduledTransactionWrapper<'a> {
    pub scheduled_transaction: &'a SaveScheduledTransaction,
}

pub type PutScheduledTransactionWrapper<'a> = PostScheduledTransactionWrapper<'a>;

pub type AccountResponse = SuccessResponse<AccountPayload>;
pub type AccountsResponse = SuccessResponse<AccountsPayload>;
pub type BulkResponse = SuccessResponse<BulkPayload>;
pub type CategoriesResponse = SuccessResponse<CategoriesPayload>;
pub type CategoryResponse = SuccessResponse<CategoryPayload>;
pub type HybridTransactionsResponse = SuccessResponse<HybridTransactionsPayload>;
pub type MoneyMovementGroupsResponse = SuccessResponse<MoneyMovementGroupsPayload>;
pub type MoneyMovementsResponse = SuccessResponse<MoneyMovementsPayload>;
pub type MonthDetailResponse = SuccessResponse<MonthPayload>;
pub type MonthSummariesResponse = SuccessResponse<MonthsPayload>;
pub type PayeeLocationResponse = SuccessResponse<PayeeLocationPayload>;
pub type PayeeLocationsResponse = SuccessResponse<PayeeLocationsPayload>;
pub type PayeeResponse = SuccessResponse<PayeePayload>;
pub type PayeesResponse = SuccessResponse<PayeesPayload>;
pub type PlanDetailResponse = SuccessResponse<PlanDetailPayload>;
pub type PlanSettingsResponse = SuccessResponse<PlanSettingsPayload>;
pub type PlanSummaryResponse = SuccessResponse<PlansPayload>;
pub type SaveCategoryGroupResponse = SuccessResponse<SaveCategoryGroupPayload>;
pub type SaveCategoryResponse = SuccessResponse<SaveCategoryPayload>;
pub type SavePayeeResponse = SuccessResponse<SavePayeePayload>;
pub type SaveTransactionsResponse = SuccessResponse<SaveTransactionsPayload>;
pub type ScheduledTransactionResponse = SuccessResponse<ScheduledTransactionPayload>;
pub type ScheduledTransactionsResponse = SuccessResponse<ScheduledTransactionsPayload>;
pub type TransactionResponse = SuccessResponse<TransactionPayload>;
pub type TransactionsImportResponse = SuccessResponse<ImportPayload>;
pub type TransactionsResponse = SuccessResponse<TransactionsPayload>;
pub type UserResponse = SuccessResponse<UserPayload>;
