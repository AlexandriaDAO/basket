// src/kong_locker/lock_canister/src/lib.rs
use ic_cdk::{update, query, init};
use candid::{CandidType, Deserialize, Principal, Nat};
use serde::Serialize;
use std::cell::RefCell;

thread_local! {
    static CREATOR: RefCell<Principal> = RefCell::new(Principal::anonymous());
}

// Define transfer types directly to avoid version conflicts
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct IcpTransferArgs {
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub from_subaccount: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

// ===== KongSwap Types (matching actual implementation) =====
#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub enum TxId {
    BlockIndex(Nat),           // For ICP and SNS tokens
    TransactionHash(String),    // For ICRC tokens
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub struct SwapArgs {
    pub pay_token: String,              // Token to pay (e.g., "ICP")
    pub pay_amount: Nat,                // Amount in base units
    pub pay_tx_id: Option<TxId>,       // Transfer tx ID
    pub receive_token: String,          // Token to receive (e.g., "ALEX")
    pub receive_amount: Option<Nat>,    // Min amount (slippage protection)
    pub receive_address: Option<String>, // Optional different address
    pub max_slippage: Option<f64>,      // Max slippage % (e.g., 100.0)
    pub referred_by: Option<String>,    // Referral code
}

// Full SwapReply structure from KongSwap
#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub struct SwapReply {
    pub tx_id: u64,                 // Internal KongSwap transaction ID
    pub request_id: u64,            // Request tracking ID
    pub status: String,             // "Success" or error message
    pub pay_chain: String,          // Chain of pay token
    #[serde(default = "empty_string")]
    pub pay_address: String,        // Address of pay token
    pub pay_symbol: String,         // Pay token symbol
    pub pay_amount: Nat,            // Amount paid
    pub receive_chain: String,      // Chain of receive token
    #[serde(default = "empty_string")]
    pub receive_address: String,    // Address of receive token
    pub receive_symbol: String,     // Receive token symbol
    pub receive_amount: Nat,        // Amount received
    pub mid_price: f64,             // Mid price
    pub price: f64,                 // Execution price
    pub slippage: f64,              // Actual slippage
    pub txs: Vec<SwapTxReply>,      // Transaction details
    pub transfer_ids: Vec<TransferIdReply>, // Transfer IDs
    pub claim_ids: Vec<u64>,        // Claim IDs
    pub ts: u64,                    // Timestamp
}

// Helper function for serde default
fn empty_string() -> String {
    String::new()
}

// Minimal types for nested fields (we don't use these but need them for deserialization)
#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub struct SwapTxReply {
    pub pool_id: u32,
    pub pay_amount: Nat,
    pub receive_amount: Nat,
    pub price: f64,
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub struct TransferIdReply {
    pub transfer_id: u64,
    pub transfer: TransferReply,
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub struct TransferReply {
    pub token_id: u32,
    pub amount: Nat,
}

/// Register with KongSwap if ICP is available
/// Anyone can call this - ICP requirement prevents spam
#[update]
async fn register_if_funded() -> Result<String, String> {
    // CRITICAL: Check balance FIRST to prevent cycle drain from spam calls
    // This check is cheap and fails fast without using cycles
    let self_principal = ic_cdk::id();
    let icp_ledger = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    
    // Get balance - this is the ONLY inter-canister call if not funded
    let balance_result: Result<(Nat,), _> = ic_cdk::call(
        icp_ledger,
        "icrc1_balance_of",
        (Account { 
            owner: self_principal, 
            subaccount: None 
        },)
    ).await;
    
    let balance = match balance_result {
        Ok((bal,)) => bal,
        Err(_) => return Err("Failed to check balance".to_string()),
    };
    
    // Need at least 0.99 ICP (99_000_000 e8s) for KongSwap registration
    if balance < Nat::from(99_000_000u64) {
        return Err("Insufficient ICP. Need at least 0.99 ICP".to_string());
    }
    
    // Transfer ALL ICP to KongSwap (minus fee) to avoid idle funds
    let kong_backend = Principal::from_text("2ipq2-uqaaa-aaaar-qailq-cai").unwrap();
    let transfer_amount = balance - Nat::from(10_000u64); // Keep fee back
    
    let transfer_result: Result<(Result<Nat, String>,), _> = ic_cdk::call(
        icp_ledger,
        "icrc1_transfer",
        (IcpTransferArgs {
            to: Account { owner: kong_backend, subaccount: None },
            amount: transfer_amount.clone(), // Send ALL available ICP
            fee: Some(Nat::from(10_000u64)),
            memo: None,
            from_subaccount: None,
            created_at_time: None,
        },)
    ).await;
    
    let block_index = match transfer_result {
        Ok((Ok(block),)) => block,
        _ => return Err("Transfer to KongSwap failed".to_string()),
    };
    
    // Swap ALL transferred ICP for ALEX to register
    // KongSwap returns Result<SwapReply, String> directly
    let swap_result: Result<(Result<SwapReply, String>,), _> = ic_cdk::call(
        kong_backend,
        "swap",
        (SwapArgs {
            pay_token: "ICP".to_string(),
            pay_amount: transfer_amount,
            pay_tx_id: Some(TxId::BlockIndex(block_index)),
            receive_token: "ALEX".to_string(),
            receive_amount: None,
            receive_address: None,
            max_slippage: Some(100.0),
            referred_by: None,
        },)
    ).await;
    
    match swap_result {
        Ok((Ok(reply),)) if reply.status == "Success" => {
            Ok("Successfully registered with KongSwap".to_string())
        },
        Ok((Ok(_),)) => Ok("Registration attempted - swap executed".to_string()),
        Ok((Err(e),)) => Ok(format!("Registration attempted - {}", e)),
        Err(_) => Ok("Registration attempted - may already be registered".to_string()),
    }
}

/// Initialize the canister with the creator's principal
#[init]
fn init(creator: Principal) {
    CREATOR.with(|c| *c.borrow_mut() = creator);
}

/// Get the canister's principal (for sending LP tokens to)
#[query]
fn get_principal() -> Principal {
    ic_cdk::id()
}

/// Get the principal of the user who created this lock canister
#[query]
fn get_creator() -> Principal {
    CREATOR.with(|c| *c.borrow())
}

/// Get the version of this lock canister code
#[query]
fn version() -> String {
    "V1".to_string()
}

// Candid interface for dashboard UI
#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    include_str!("../lock_canister.did").to_string()
}

// THAT'S IT! No transfer functions, no owner tracking, no complexity
// This canister can receive LP tokens but can NEVER send them

ic_cdk::export_candid!();