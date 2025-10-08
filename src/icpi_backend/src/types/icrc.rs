use candid::{CandidType, Deserialize, Nat, Principal};
use serde::Serialize;

// ICRC1 Account type
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<[u8; 32]>,
}

// ICRC1 Transfer arguments
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct TransferArgs {
    pub from_subaccount: Option<[u8; 32]>,
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

// ICRC1 Transfer result
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum TransferResult {
    Ok(Nat),
    Err(TransferError),
}

// ICRC1 Transfer errors
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

// ICRC2 Approve arguments
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ApproveArgs {
    pub from_subaccount: Option<[u8; 32]>,
    pub spender: Account,
    pub amount: Nat,
    pub expected_allowance: Option<Nat>,
    pub expires_at: Option<u64>,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

// ICRC2 Approve result
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum ApproveResult {
    Ok(Nat),
    Err(ApproveError),
}

// ICRC2 Approve errors
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum ApproveError {
    BadFee { expected_fee: Nat },
    InsufficientFunds { balance: Nat },
    AllowanceChanged { current_allowance: Nat },
    Expired { ledger_time: u64 },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

// ICRC2 TransferFrom arguments
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct TransferFromArgs {
    pub from: Account,
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

// ICRC2 TransferFrom errors
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum TransferFromError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}