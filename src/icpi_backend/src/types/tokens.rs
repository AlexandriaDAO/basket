use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::hash::Hash;

// Central ICPI token constant
pub const ICPI_CANISTER_ID: &str = "l6lep-niaaa-aaaap-qqeda-cai";
pub const CKUSDT_CANISTER_ID: &str = "cngnf-vqaaa-aaaar-qag4q-cai";

// Token identifiers - use symbols as strings (Kongswap pattern)
#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrackedToken {
    ALEX,
    ZERO,
    KONG,
    BOB,
    ckUSDT,
}

impl TrackedToken {
    // Single source of truth for all tracked tokens (excluding ckUSDT which is the reserve asset)
    pub const ALL: [TrackedToken; 4] = [
        TrackedToken::ALEX,
        TrackedToken::ZERO,
        TrackedToken::KONG,
        TrackedToken::BOB,
    ];

    pub fn all() -> &'static [TrackedToken] {
        &Self::ALL
    }

    pub fn all_vec() -> Vec<TrackedToken> {
        Self::ALL.to_vec()
    }

    pub fn to_symbol(&self) -> &str {
        match self {
            TrackedToken::ALEX => "ALEX",
            TrackedToken::ZERO => "ZERO",
            TrackedToken::KONG => "KONG",
            TrackedToken::BOB => "BOB",
            TrackedToken::ckUSDT => "ckUSDT",
        }
    }

    pub fn from_symbol(symbol: &str) -> Result<Self, String> {
        match symbol {
            "ALEX" => Ok(TrackedToken::ALEX),
            "ZERO" => Ok(TrackedToken::ZERO),
            "KONG" => Ok(TrackedToken::KONG),
            "BOB" => Ok(TrackedToken::BOB),
            "ckUSDT" => Ok(TrackedToken::ckUSDT),
            _ => Err(format!("Unknown tracked token symbol: {}", symbol)),
        }
    }

    pub fn get_canister_id(&self) -> Result<Principal, String> {
        match self {
            TrackedToken::ALEX => Principal::from_text("ysy5f-2qaaa-aaaap-qkmmq-cai")
                .map_err(|e| format!("Invalid ALEX principal: {}", e)),
            TrackedToken::ZERO => Principal::from_text("b3d2q-ayaaa-aaaap-qqcfq-cai")
                .map_err(|e| format!("Invalid ZERO principal: {}", e)),
            TrackedToken::KONG => Principal::from_text("o7oak-iyaaa-aaaaq-aadzq-cai")
                .map_err(|e| format!("Invalid KONG principal: {}", e)),
            TrackedToken::BOB => Principal::from_text("7pail-xaaaa-aaaas-aabmq-cai")
                .map_err(|e| format!("Invalid BOB principal: {}", e)),
            TrackedToken::ckUSDT => Principal::from_text(CKUSDT_CANISTER_ID)
                .map_err(|e| format!("Invalid ckUSDT principal: {}", e)),
        }
    }

    pub fn get_decimals(&self) -> u8 {
        match self {
            TrackedToken::ALEX => 8,
            TrackedToken::ZERO => 8,
            TrackedToken::KONG => 8,
            TrackedToken::BOB => 8,
            TrackedToken::ckUSDT => 6,
        }
    }
}

// Token metadata for frontend queries
#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct TokenMetadata {
    pub symbol: String,
    pub canister_id: Principal,
    pub decimals: u8,
}