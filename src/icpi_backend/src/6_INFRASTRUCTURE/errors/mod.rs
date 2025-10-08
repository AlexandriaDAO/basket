//! Error types for the ICPI backend

use candid::{CandidType, Deserialize};
use serde::Serialize;

// Result type alias for the entire application
pub type Result<T> = std::result::Result<T, IcpiError>;

// Main error type
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum IcpiError {
    // Minting errors
    Mint(MintError),

    // Burning errors
    Burn(BurnError),

    // Rebalancing errors
    Rebalance(RebalanceError),

    // Trading/swap errors
    Trading(TradingError),

    // Kongswap integration errors
    Kongswap(KongswapError),

    // Validation errors
    Validation(ValidationError),

    // Calculation errors
    Calculation(CalculationError),

    // System errors
    System(SystemError),

    // Query errors
    Query(QueryError),

    // Generic error
    Other(String),
}

// Mint-specific errors
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum MintError {
    InvalidMintId { id: String },
    AmountBelowMinimum { amount: String, minimum: String },
    AmountAboveMaximum { amount: String, maximum: String },
    FeeCollectionFailed { user: String, reason: String },
    DepositCollectionFailed { user: String, amount: String, reason: String },
    RefundFailed { user: String, amount: String, reason: String },
    InsufficientTVL { tvl: String, required: String },
    LedgerInteractionFailed { operation: String, details: String },
    Unauthorized { principal: String, mint_id: String },
    ProportionalCalculationError { reason: String },
}

// Burn-specific errors
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum BurnError {
    AmountBelowMinimum { amount: String, minimum: String },
    InsufficientApproval { required: String, approved: String },
    InsufficientBalance { required: String, available: String },
    NoSupply,
    NoRedemptionsPossible { reason: String },
    TokenTransferFailed { token: String, amount: String, reason: String },
}

// Rebalancing errors
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum RebalanceError {
    TimerNotActive,
    TooSoonToRebalance { last_time: u64, next_time: u64 },
    AllocationCalculationError { reason: String },
    SwapFailed { token: String, amount: String, reason: String },
    InsufficientBalance { token: String, available: String, required: String },
}

// Trading/swap errors
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum TradingError {
    InvalidQuote { reason: String },
    SlippageTooHigh { expected: String, actual: String, max_allowed: String },
    ApprovalFailed { token: String, amount: String, reason: String },
}

// Kongswap integration errors
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum KongswapError {
    BackendUnreachable { reason: String },
    LiquidityPoolNotFound { token_a: String, token_b: String },
    SwapAmountCalculationFailed { reason: String },
}

// Validation errors
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum ValidationError {
    InvalidPrincipal { principal: String },
    InvalidAmount { amount: String, reason: String },
    SupplyOutOfBounds { supply: String, max: String },
    PriceOutOfBounds { price: String, min: String, max: String },
    RapidChangeDetected { field: String, old_value: String, new_value: String, max_change: String },
    DataInconsistency { reason: String },
}

// Calculation errors
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum CalculationError {
    Overflow { operation: String },
    DivisionByZero { operation: String },
    ConversionError { from: String, to: String, reason: String },
    PrecisionLoss { operation: String, original: String, result: String },
}

// System errors
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum SystemError {
    Unauthorized { principal: String, required_role: String },
    StateCorrupted { reason: String },
    InterCanisterCallFailed { canister: String, method: String, reason: String },
    OperationInProgress { operation: String, user: String },
}

// Query errors
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum QueryError {
    CanisterUnreachable { canister: String, reason: String },
    InvalidResponse { canister: String, method: String, reason: String },
    Timeout { canister: String, method: String },
}

// Display implementations
impl std::fmt::Display for IcpiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IcpiError::Mint(e) => write!(f, "Mint error: {:?}", e),
            IcpiError::Burn(e) => write!(f, "Burn error: {:?}", e),
            IcpiError::Rebalance(e) => write!(f, "Rebalance error: {:?}", e),
            IcpiError::Trading(e) => write!(f, "Trading error: {:?}", e),
            IcpiError::Kongswap(e) => write!(f, "Kongswap error: {:?}", e),
            IcpiError::Validation(e) => write!(f, "Validation error: {:?}", e),
            IcpiError::Calculation(e) => write!(f, "Calculation error: {:?}", e),
            IcpiError::System(e) => write!(f, "System error: {:?}", e),
            IcpiError::Query(e) => write!(f, "Query error: {:?}", e),
            IcpiError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<String> for IcpiError {
    fn from(msg: String) -> Self {
        IcpiError::Other(msg)
    }
}

impl From<&str> for IcpiError {
    fn from(msg: &str) -> Self {
        IcpiError::Other(msg.to_string())
    }
}

impl From<candid::Error> for IcpiError {
    fn from(e: candid::Error) -> Self {
        IcpiError::Other(format!("Candid error: {}", e))
    }
}
