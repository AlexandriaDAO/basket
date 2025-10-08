//! System-wide constants
//! Single source of truth for all configuration values

// ===== Canister IDs =====
pub const ICPI_CANISTER_ID: &str = "l6lep-niaaa-aaaap-qqeda-cai"; // ICPI token ledger
pub const ICPI_LEDGER_ID: &str = "l6lep-niaaa-aaaap-qqeda-cai";
pub const ICPI_BACKEND_ID: &str = "ev6xm-haaaa-aaaap-qqcza-cai";
pub const CKUSDT_CANISTER_ID: &str = "cngnf-vqaaa-aaaar-qag4q-cai";
pub const KONGSWAP_BACKEND_ID: &str = "2ipq2-uqaaa-aaaar-qailq-cai";
pub const KONG_LOCKER_ID: &str = "eazgb-giaaa-aaaap-qqc2q-cai";

// Token canister IDs
pub const ALEX_CANISTER_ID: &str = "ysy5f-2qaaa-aaaap-qkmmq-cai";
pub const ZERO_CANISTER_ID: &str = "b3d2q-ayaaa-aaaap-qqcfq-cai";
pub const KONG_CANISTER_ID: &str = "o7oak-iyaaa-aaaaq-aadzq-cai";
pub const BOB_CANISTER_ID: &str = "7pail-xaaaa-aaaas-aabmq-cai";

// ===== Token Decimals =====
pub const ICPI_DECIMALS: u32 = 8;
pub const CKUSDT_DECIMALS: u32 = 6;
pub const ALEX_DECIMALS: u32 = 8;
pub const ZERO_DECIMALS: u32 = 8;
pub const KONG_DECIMALS: u32 = 8;
pub const BOB_DECIMALS: u32 = 8;

// ===== Minting Constants =====
pub const MIN_MINT_AMOUNT: u64 = 100_000; // 0.1 ckUSDT (e6)
pub const MAX_MINT_AMOUNT: u64 = 100_000_000_000; // 100k ckUSDT
pub const MINT_TIMEOUT_NANOS: u64 = 180_000_000_000; // 3 minutes
pub const MINT_FEE_AMOUNT: u64 = 100_000; // 0.1 ckUSDT
pub const MINT_FEE_E6: u64 = 100_000; // 0.1 ckUSDT (alias for compatibility)
pub const FEE_RECIPIENT: &str = "e454q-riaaa-aaaap-qqcyq-cai";

// ===== Burning Constants =====
pub const MIN_BURN_AMOUNT: u64 = 11_000; // 0.00011 ICPI (e8)
pub const BURN_FEE_BUFFER: u64 = 10_000; // Transfer fee buffer

// ===== Rebalancing Constants =====
pub const REBALANCE_INTERVAL_SECONDS: u64 = 3600; // 1 hour
pub const MIN_DEVIATION_PERCENT: f64 = 1.0; // 1% minimum deviation to trigger
pub const TRADE_INTENSITY: f64 = 0.1; // Trade 10% of deviation per hour
pub const MAX_SLIPPAGE_PERCENT: f64 = 2.0; // 2% max slippage
pub const MIN_TRADE_SIZE_USD: f64 = 10.0; // $10 minimum trade

// ===== Validation Thresholds =====
pub const MAX_SUPPLY_CHANGE_RATIO: f64 = 1.1; // 10% max supply change
pub const MAX_PRICE_CHANGE_RATIO: f64 = 2.0; // 100% max price change
pub const MIN_REASONABLE_PRICE: f64 = 0.0001; // $0.0001 minimum
pub const MAX_REASONABLE_PRICE: f64 = 1_000_000.0; // $1M maximum

// ===== Cache Durations (seconds) =====
pub const CACHE_DURATION_SHORT: u64 = 30;
pub const CACHE_DURATION_MEDIUM: u64 = 300;
pub const CACHE_DURATION_LONG: u64 = 3600;

// ===== Target Allocations (percentages) =====
pub const TARGET_ALEX_PERCENT: f64 = 25.0;
pub const TARGET_ZERO_PERCENT: f64 = 25.0;
pub const TARGET_KONG_PERCENT: f64 = 25.0;
pub const TARGET_BOB_PERCENT: f64 = 25.0;
