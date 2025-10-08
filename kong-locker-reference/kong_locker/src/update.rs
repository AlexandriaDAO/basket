use ic_cdk::update;
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, update_settings, canister_status,
    CanisterSettings, CreateCanisterArgument, InstallCodeArgument, 
    UpdateSettingsArgument, CanisterInstallMode, CanisterIdRecord,
};
use candid::{Principal, Nat, Encode};

use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use icrc_ledger_types::icrc1::transfer::{TransferArg as TransferArgs, TransferError};
use crate::storage::{StorablePrincipal, USER_LOCK_CANISTERS, LOCK_CANISTER_WASM};

/// Create and immediately blackhole a lock canister (with 2 ICP payment required)
#[update]
pub async fn create_lock_canister() -> Result<Principal, String> {
    let user = ic_cdk::caller();
    
    // Check if user already has one
    if USER_LOCK_CANISTERS.with(|c| c.borrow().contains_key(&StorablePrincipal(user))) {
        return Err("You already have a lock canister".to_string());
    }
    
    // STEP 1: Take 2 ICP payment from user (atomic - succeeds or fails completely)
    let icp_ledger = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    let transfer_result: Result<(Result<Nat, TransferFromError>,), _> = ic_cdk::call(
        icp_ledger,
        "icrc2_transfer_from",
        (TransferFromArgs {
            spender_subaccount: None,
            from: Account { owner: user, subaccount: None },
            to: Account { owner: ic_cdk::id(), subaccount: None },
            amount: Nat::from(200_000_000u64), // 2 ICP
            fee: Some(Nat::from(10_000u64)),
            memo: None,
            created_at_time: None,
        },)
    ).await;

    match transfer_result {
        Ok((Ok(_block_index),)) => {
            // Payment successful, continue
        },
        Ok((Err(e),)) => {
            return Err(format!("Payment failed: {:?}. Please approve 2 ICP first", e));
        },
        Err(e) => {
            return Err(format!("Transfer call failed: {:?}", e));
        }
    }
    
    // STEP 2: Create canister
    let wasm = LOCK_CANISTER_WASM.to_vec();
    let create_args = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![ic_cdk::id()]), // Factory as temporary controller
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            log_visibility: None,
            wasm_memory_limit: None,
        }),
    };
    
    let canister_id_record = create_canister(create_args, 1_500_000_000_000u128)
        .await
        .map_err(|e| format!("Failed to create canister: {:?}", e))?;
    let canister_id = canister_id_record.0.canister_id;
    
    // STEP 3: Install code with creator principal as init argument
    let install_args = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: canister_id.clone(),
        wasm_module: wasm,
        arg: Encode!(&user).unwrap(),
    };
    
    install_code(install_args)
        .await
        .map_err(|e| format!("Failed to install code: {:?}", e))?;
    
    // ⚡ CRITICAL CHANGE: Store mapping IMMEDIATELY after successful installation
    // This ensures we never lose track of the user's canister
    USER_LOCK_CANISTERS.with(|c| {
        c.borrow_mut().insert(StorablePrincipal(user), StorablePrincipal(canister_id));
    });
    ic_cdk::print(format!("Stored mapping for user {} -> canister {}", user, canister_id));
    
    // STEP 4: Send 1 ICP (best effort - don't fail if this fails)
    let transfer_args = TransferArgs {
        from_subaccount: None,
        to: Account { owner: canister_id, subaccount: None },
        amount: Nat::from(100_000_000u64), // 1 ICP
        fee: Some(Nat::from(10_000u64)),
        memo: None,
        created_at_time: None,
    };
    
    match ic_cdk::call::<_, (Result<Nat, TransferError>,)>(
        icp_ledger,
        "icrc1_transfer",
        (transfer_args,)
    ).await {
        Ok((Ok(_),)) => {
            ic_cdk::print("Successfully funded lock canister");
        },
        _ => {
            ic_cdk::print("Warning: Failed to fund lock canister - user can retry");
        }
    }
    
    // STEP 5: Trigger KongSwap registration (best effort)
    match ic_cdk::call::<_, (Result<String, String>,)>(
        canister_id,
        "register_if_funded",
        ()
    ).await {
        Ok((Ok(_),)) => {
            ic_cdk::print("Successfully triggered registration");
        },
        _ => {
            ic_cdk::print("Warning: Registration failed - user can retry");
        }
    }
    
    // STEP 6: Blackhole (best effort - can be retried if fails)
    let blackhole_args = UpdateSettingsArgument {
        canister_id: canister_id.clone(),
        settings: CanisterSettings {
            controllers: Some(vec![]), // Empty = blackholed
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            log_visibility: None,
            wasm_memory_limit: None,
        },
    };
    
    match update_settings(blackhole_args).await {
        Ok(()) => {
            ic_cdk::print("Successfully blackholed canister");
        },
        Err(e) => {
            ic_cdk::print(format!("Warning: Blackholing failed - user can retry: {:?}", e));
        }
    }
    
    Ok(canister_id)
}


/// Complete setup for a partially created canister
#[update]
pub async fn complete_my_canister_setup() -> Result<String, String> {
    let user = ic_cdk::caller();
    
    // Get user's canister
    let canister_id = USER_LOCK_CANISTERS.with(|c| 
        c.borrow().get(&StorablePrincipal(user)).map(|sp| sp.0)
    ).ok_or("No lock canister found. Use create_lock_canister() first.")?;
    
    let mut actions_taken = Vec::new();
    let icp_ledger = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    
    // Check 1: Does it have code installed?
    match ic_cdk::call::<_, (Principal,)>(canister_id, "get_principal", ()).await {
        Ok(_) => {
            // Has code, proceed to next checks
        },
        Err(_) => {
            // No code, install it with creator principal as init argument
            let wasm = LOCK_CANISTER_WASM.to_vec();
            let install_args = InstallCodeArgument {
                mode: CanisterInstallMode::Install,
                canister_id: canister_id.clone(),
                wasm_module: wasm,
                arg: Encode!(&user).unwrap(),
            };
            
            install_code(install_args)
                .await
                .map_err(|e| format!("Failed to install code: {:?}", e))?;
            
            actions_taken.push("installed code");
        }
    }
    
    // Check 2: Does it have ICP for registration?
    let balance_result: Result<(Nat,), _> = ic_cdk::call(
        icp_ledger,
        "icrc1_balance_of",
        (Account { owner: canister_id, subaccount: None },)
    ).await;
    
    let needs_funding = match balance_result {
        Ok((balance,)) => balance < Nat::from(100_010_000u64), // Need 1 ICP + fees
        Err(_) => true,
    };
    
    if needs_funding {
        // Send 1 ICP
        let transfer_args = TransferArgs {
            from_subaccount: None,
            to: Account { owner: canister_id, subaccount: None },
            amount: Nat::from(100_000_000u64),
            fee: Some(Nat::from(10_000u64)),
            memo: None,
            created_at_time: None,
        };
        
        match ic_cdk::call::<_, (Result<Nat, TransferError>,)>(
            icp_ledger,
            "icrc1_transfer",
            (transfer_args,)
        ).await {
            Ok((Ok(_),)) => {
                actions_taken.push("funded with 1 ICP");
            },
            _ => {
                return Err("Failed to fund canister. Please send 1 ICP manually.".to_string());
            }
        }
    }
    
    // Check 3: Is it registered with KongSwap?
    // We only care about whether the user exists, not the actual balance data
    let kong_backend = Principal::from_text("2ipq2-uqaaa-aaaar-qailq-cai").unwrap();
    let kong_check: Result<(Result<Vec<candid::Empty>, String>,), _> = ic_cdk::call(
        kong_backend,
        "user_balances",
        (canister_id.to_text(),)
    ).await;
    
    let needs_registration = match kong_check {
        Ok((Err(msg),)) if msg.contains("User not found") => true,
        Ok((Ok(_),)) => false,
        _ => true, // Assume needs registration if unclear
    };
    
    if needs_registration {
        // Trigger registration
        match ic_cdk::call::<_, (Result<String, String>,)>(
            canister_id,
            "register_if_funded",
            ()
        ).await {
            Ok((Ok(_),)) => {
                actions_taken.push("registered with KongSwap");
            },
            Ok((Err(e),)) => {
                if !e.contains("already be registered") {
                    return Err(format!("Registration failed: {}", e));
                }
            },
            Err(e) => {
                return Err(format!("Failed to call registration: {:?}", e));
            }
        }
    }
    
    // Check 4: Is it blackholed?
    let status_result = canister_status(CanisterIdRecord { canister_id }).await;
    
    let is_blackholed = match status_result {
        Ok(status) => status.0.settings.controllers.is_empty(),
        Err(_) => false, // Assume not blackholed if can't check
    };
    
    if !is_blackholed {
        // Blackhole it
        let blackhole_args = UpdateSettingsArgument {
            canister_id: canister_id.clone(),
            settings: CanisterSettings {
                controllers: Some(vec![]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
                log_visibility: None,
                wasm_memory_limit: None,
            },
        };
        
        update_settings(blackhole_args)
            .await
            .map_err(|e| format!("Failed to blackhole: {:?}", e))?;
        
        actions_taken.push("blackholed canister");
    }
    
    if actions_taken.is_empty() {
        Ok("Canister already fully configured".to_string())
    } else {
        Ok(format!("Completed setup: {}", actions_taken.join(", ")))
    }
}

