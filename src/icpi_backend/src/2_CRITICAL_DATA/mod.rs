//! Critical Data - Portfolio calculations and validation
//! Source of truth for all financial data

pub mod portfolio_value;
pub mod supply_tracker;
pub mod token_queries;
pub mod validation;

// Re-export commonly used functions
pub use portfolio_value::{calculate_portfolio_value_atomic, get_portfolio_state_uncached};
pub use supply_tracker::{get_icpi_supply_uncached, get_validated_supply};
pub use token_queries::{get_all_balances_uncached, get_token_balance_uncached};
pub use validation::{validate_price, validate_supply};

