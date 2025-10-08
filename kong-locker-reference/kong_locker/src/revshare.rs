use candid::{Principal, Nat};
use icrc_ledger_types::icrc1::{account::Account, transfer::TransferArg};
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;

// Constants
const MIN_ICP_BALANCE: u64 = 500_000_000;  // 5 ICP threshold (3 ICP to keep + 2 ICP buffer)
const ICP_RESERVE: u64 = 300_000_000;      // 3 ICP permanent reserve (enough for 3 canister creations)  
const CHECK_INTERVAL: u64 = 14400;         // Check every 4 hours
const ALEX_REVSHARE: &str = "e454q-riaaa-aaaap-qqcyq-cai";  // Centralized revenue sharing service
const ICP_LEDGER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

// Initialize revenue sharing timer
pub fn init_revshare_timer() {
    set_timer_interval(
        Duration::from_secs(CHECK_INTERVAL), 
        || {
            ic_cdk::spawn(async {
                let _ = send_to_revshare().await;
            });
        }
    );
    ic_cdk::print("Revenue sharing timer initialized - sending to alex_revshare every 4 hours");
}

// Check balance and send excess to alex_revshare
async fn send_to_revshare() -> Result<String, String> {
    ic_cdk::println!("REVSHARE: Checking balance for revenue sharing...");
    
    // Check ICP balance
    let balance = get_icp_balance().await?;
    
    ic_cdk::println!("REVSHARE: Balance check - {} e8s", balance);
    
    // Only proceed if we have more than 5 ICP
    if balance < MIN_ICP_BALANCE {
        ic_cdk::println!("REVSHARE: Balance {} below threshold {}", balance, MIN_ICP_BALANCE);
        return Ok(format!("Balance {} below threshold", balance));
    }
    
    // Calculate transfer amount (keep 3 ICP + fee for next transfer)
    let transfer_amount = balance.saturating_sub(ICP_RESERVE + 10_000);
    
    ic_cdk::println!("REVSHARE: Sending {} e8s to alex_revshare", transfer_amount);
    
    // Transfer to alex_revshare
    transfer_icp_to_revshare(transfer_amount).await?;
    
    Ok(format!("Successfully sent {} ICP to alex_revshare", transfer_amount / 100_000_000))
}

// Get current ICP balance
async fn get_icp_balance() -> Result<u64, String> {
    let ledger = Principal::from_text(ICP_LEDGER).unwrap();
    let canister_id = ic_cdk::api::id();
    
    let account = Account {
        owner: canister_id,
        subaccount: None,
    };
    
    let result: Result<(Nat,), _> = ic_cdk::call(
        ledger,
        "icrc1_balance_of",
        (account,)
    ).await;
    
    match result {
        Ok((balance,)) => {
            let balance_str = balance.to_string();
            Ok(balance_str.parse::<u64>().unwrap_or(0))
        }
        Err(e) => Err(format!("Failed to get balance: {:?}", e))
    }
}

// Transfer ICP to alex_revshare canister
async fn transfer_icp_to_revshare(amount: u64) -> Result<(), String> {
    let icp_ledger = Principal::from_text(ICP_LEDGER).unwrap();
    let revshare_canister = Principal::from_text(ALEX_REVSHARE).unwrap();
    
    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: revshare_canister,
            subaccount: None,
        },
        fee: None,
        created_at_time: None,
        memo: None,
        amount: Nat::from(amount),
    };
    
    let result: Result<(Result<Nat, icrc_ledger_types::icrc1::transfer::TransferError>,), _> = ic_cdk::call(
        icp_ledger,
        "icrc1_transfer",
        (transfer_args,)
    ).await;
    
    match result {
        Ok((Ok(block_index),)) => {
            ic_cdk::println!("REVSHARE: Successfully transferred {} ICP to alex_revshare, block: {}", 
                amount / 100_000_000, block_index);
            Ok(())
        }
        Ok((Err(e),)) => Err(format!("Transfer failed: {:?}", e)),
        Err(e) => Err(format!("Transfer call failed: {:?}", e)),
    }
}