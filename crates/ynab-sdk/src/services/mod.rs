mod accounts;
mod categories;
mod category_groups;
mod money_movements;
mod months;
mod payee_locations;
mod payees;
mod plans;
mod scheduled_transactions;
mod transactions;
mod user;
pub use accounts::AccountsService;
pub use categories::CategoriesService;
pub use category_groups::CategoryGroupsService;
pub use money_movements::{MoneyMovementGroupsService, MoneyMovementsService};
pub use months::MonthsService;
pub use payee_locations::PayeeLocationsService;
pub use payees::PayeesService;
pub use plans::PlansService;
pub use scheduled_transactions::ScheduledTransactionsService;
pub use transactions::TransactionsService;
pub use user::UserService;

use crate::Client;
impl Client {
    #[must_use]
    pub fn user(&self) -> UserService {
        UserService::new(self.clone())
    }
    #[must_use]
    pub fn plans(&self) -> PlansService {
        PlansService::new(self.clone())
    }
    #[must_use]
    pub fn accounts(&self) -> AccountsService {
        AccountsService::new(self.clone())
    }
    #[must_use]
    pub fn categories(&self) -> CategoriesService {
        CategoriesService::new(self.clone())
    }
    #[must_use]
    pub fn category_groups(&self) -> CategoryGroupsService {
        CategoryGroupsService::new(self.clone())
    }
    #[must_use]
    pub fn payees(&self) -> PayeesService {
        PayeesService::new(self.clone())
    }
    #[must_use]
    pub fn payee_locations(&self) -> PayeeLocationsService {
        PayeeLocationsService::new(self.clone())
    }
    #[must_use]
    pub fn months(&self) -> MonthsService {
        MonthsService::new(self.clone())
    }
    #[must_use]
    pub fn money_movements(&self) -> MoneyMovementsService {
        MoneyMovementsService::new(self.clone())
    }
    #[must_use]
    pub fn money_movement_groups(&self) -> MoneyMovementGroupsService {
        MoneyMovementGroupsService::new(self.clone())
    }
    #[must_use]
    pub fn transactions(&self) -> TransactionsService {
        TransactionsService::new(self.clone())
    }
    #[must_use]
    pub fn scheduled_transactions(&self) -> ScheduledTransactionsService {
        ScheduledTransactionsService::new(self.clone())
    }
}
