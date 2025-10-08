use candid::{CandidType, Deserialize, Nat, Principal};
use serde::Serialize;

// TxId type from kong_backend.did line 148-151
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum TxId {
    BlockIndex(Nat),
    TransactionId(String),
}

// SwapArgs from kong_backend.did lines 488-497
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SwapArgs {
    pub pay_token: String,           // Token symbol like "ICP", "ckUSDT"
    pub pay_amount: Nat,
    pub pay_tx_id: Option<TxId>,    // None for ICRC2 flow, Some for ICRC1
    pub receive_token: String,      // Token symbol
    pub receive_amount: Option<Nat>,
    pub receive_address: Option<String>,
    pub max_slippage: Option<f64>,
    pub referred_by: Option<String>,
}

// SwapTxReply from kong_backend.did lines 498-512
// CRITICAL: Must include ALL fields including 'ts'!
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SwapTxReply {
    pub pool_symbol: String,
    pub pay_chain: String,
    pub pay_address: String,
    pub pay_symbol: String,
    pub pay_amount: Nat,
    pub receive_chain: String,
    pub receive_address: String,
    pub receive_symbol: String,
    pub receive_amount: Nat,
    pub price: f64,
    pub lp_fee: Nat,
    pub gas_fee: Nat,
    pub ts: u64,  // IMPORTANT: This field was missing in previous attempt!
}

// ICTransferReply from kong_backend.did lines 153-160
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ICTransferReply {
    pub chain: String,
    pub symbol: String,
    pub is_send: bool,
    pub amount: Nat,
    pub canister_id: String,
    pub block_index: Nat,
}

// TransferReply from kong_backend.did lines 161-163
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum TransferReply {
    IC(ICTransferReply),
}

// TransferIdReply from kong_backend.did lines 164-167
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct TransferIdReply {
    pub transfer_id: u64,
    pub transfer: TransferReply,
}

// SwapReply from kong_backend.did lines 513-532
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SwapReply {
    pub tx_id: u64,
    pub request_id: u64,
    pub status: String,
    pub pay_chain: String,
    pub pay_address: String,
    pub pay_symbol: String,
    pub pay_amount: Nat,
    pub receive_chain: String,
    pub receive_address: String,
    pub receive_symbol: String,
    pub receive_amount: Nat,
    pub mid_price: f64,
    pub price: f64,
    pub slippage: f64,
    pub txs: Vec<SwapTxReply>,
    pub transfer_ids: Vec<TransferIdReply>,
    pub claim_ids: Vec<u64>,
    pub ts: u64,
}

// SwapAmounts types for price queries
#[derive(CandidType, Deserialize, Debug)]
pub struct SwapAmountsReply {
    pub pay_symbol: String,
    pub receive_symbol: String,
    pub pay_amount: Nat,
    pub receive_amount: Nat,
    pub mid_price: f64,
    pub price: f64,
    pub slippage: f64,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum SwapAmountsResult {
    Ok(SwapAmountsReply),
    Err(String),
}

// Match Kongswap's LPBalancesReply structure EXACTLY
#[derive(CandidType, Deserialize, Debug)]
pub struct LPBalancesReply {
    pub name: String,
    pub symbol: String,
    pub lp_token_id: u64,
    pub balance: f64,
    pub usd_balance: f64,
    pub chain_0: String,
    pub symbol_0: String,
    pub address_0: String,
    pub amount_0: f64,
    pub usd_amount_0: f64,
    pub chain_1: String,
    pub symbol_1: String,
    pub address_1: String,
    pub amount_1: f64,
    pub usd_amount_1: f64,
    pub ts: u64,
}

// Kongswap user_balances response types
#[derive(CandidType, Deserialize, Debug)]
pub enum UserBalancesReply {
    LP(LPBalancesReply),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum UserBalancesResult {
    Ok(Vec<UserBalancesReply>),
    Err(String),
}