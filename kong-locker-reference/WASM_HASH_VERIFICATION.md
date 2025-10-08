# Lock Canister WASM Hash Verification

## Deployed Lock Canisters
All lock canisters created by Kong Locker on mainnet have the following module hash:
```
0x8a4e009fbb6fa0aa5b92cc3fa623cc9d2fea1a0c7ca679a7d589fe1fa96b37f4
```

Example canisters:
- fyqtc-iiaaa-aaaap-qqc6q-cai
- ma6tr-2qaaa-aaaap-qqdma-cai
- net4d-dqaaa-aaaap-qqdka-cai

## Verification Instructions

To verify that the GitHub source matches the deployed canisters:

1. **Clone the repository**
   ```bash
   git clone https://github.com/AlexandriaDAO/daopad.git
   cd daopad/src/kong_locker
   ```

2. **Build lock_canister**
   ```bash
   cargo build --target wasm32-unknown-unknown --release -p lock_canister --locked
   ```

3. **Verify the hash**
   ```bash
   sha256sum ../../target/wasm32-unknown-unknown/release/lock_canister.wasm
   ```

   Expected output:
   ```
   8a4e009fbb6fa0aa5b92cc3fa623cc9d2fea1a0c7ca679a7d589fe1fa96b37f4
   ```

4. **Verify deployed canister module hash**
   ```bash
   dfx canister --network ic info fyqtc-iiaaa-aaaap-qqc6q-cai
   ```

   Should show:
   ```
   Module hash: 0x8a4e009fbb6fa0aa5b92cc3fa623cc9d2fea1a0c7ca679a7d589fe1fa96b37f4
   ```

## Important Notes

### Candid Interface Preservation
The `lock_canister.did` file is NOT auto-generated to preserve the exact module hash. The deployed canisters use the Candid interface from commit e023645 (Sept 8, 2025), which does not include the `version` function in the interface even though the function exists in the code.

### Why This Matters
- The module hash includes the embedded Candid interface
- Even a single line change in the .did file changes the module hash
- The deployed canisters have the `version` function in code but not in the Candid interface

### DO NOT Run candid-extractor
Running `candid-extractor` on lock_canister will add the `version` function to the .did file and change the module hash. To maintain verification:
- Do NOT run: `candid-extractor target/wasm32-unknown-unknown/release/lock_canister.wasm > lock_canister.did`
- The lock_canister.did must remain unchanged to match deployed canisters

## Build Requirements
For exact reproducibility:
- Rust toolchain: See rust-toolchain.toml
- Dependencies: Locked via Cargo.lock
- Build flags: `--release --locked`
- Optimization: opt-level = "z", lto = true (in Cargo.toml)

## Security Guarantees
- All lock canisters are **blackholed** (no controllers)
- LP tokens are **permanently locked** (no unlock mechanism)
- Module hash verification proves the code matches GitHub source