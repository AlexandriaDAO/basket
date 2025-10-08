//! Pure mathematical functions module
//! No I/O, no async - deterministic calculations only

pub mod pure_math;

pub use pure_math::{
    multiply_and_divide,
    convert_decimals,
    calculate_mint_amount,
    calculate_redemptions,
    calculate_trade_size,
};
