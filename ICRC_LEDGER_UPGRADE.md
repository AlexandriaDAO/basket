# ICRC Ledger Upgrade Documentation

## icrc1-ledger.wasm.gz

This file contains the ICRC-1/ICRC-2 ledger wasm used to upgrade the ICPI token ledger to enable ICRC-2 features.

### File Information

- **File**: `icrc1-ledger.wasm.gz`
- **Size**: 569 KB (compressed)
- **Source**: https://download.dfinity.systems/ic/d87954601e4b22972899e9957e800406a0a6b929/canisters/ic-icrc1-ledger.wasm.gz
- **IC Release**: d87954601e4b22972899e9957e800406a0a6b929
- **Purpose**: Enable ICRC-2 features (approve/transfer_from) on ICPI ledger

### Upgrade Details

**Date**: 2025-10-08
**Canister**: l6lep-niaaa-aaaap-qqeda-cai (ICPI Token Ledger)
**Upgrade Type**: Feature flag enablement

**Command Used**:
```bash
dfx canister --network ic install l6lep-niaaa-aaaap-qqeda-cai \
  --mode upgrade \
  --wasm icrc1-ledger.wasm.gz \
  --argument '(variant { Upgrade = opt record { feature_flags = opt record { icrc2 = true } } })'
```

**Result**: Successfully upgraded. ICRC-2 functions now available.

### Verification

```bash
# Test ICRC-2 approve function
$ dfx canister --network ic call l6lep-niaaa-aaaap-qqeda-cai icrc2_approve \
  '(record { spender = record { owner = principal "ev6xm-haaaa-aaaap-qqcza-cai" }; amount = 100000000 })'
(variant { Ok = 10 : nat })  ✅
```

### Why This File is Committed

This wasm file is preserved in the repository for:
1. **Audit Trail**: Documents the exact version used to enable ICRC-2
2. **Reproducibility**: Ensures the upgrade can be verified and replicated
3. **Security**: Provides transparency for what code is running on the ledger
4. **Historical Reference**: Records a critical infrastructure upgrade

### Security Notes

- This is the official DFINITY ICRC-1 ledger implementation
- Downloaded from official IC release artifacts
- Enables ICRC-2 which is required for the burn flow in ICPI backend
- Before enabling ICRC-2, burn operations failed with "ICRC-2 features are not enabled on the ledger"

### Related PRs

- **PR #5**: Attempted workaround (removed ICRC-2 usage) - CLOSED
- **PR #6**: Proper fix after enabling ICRC-2 - uses this wasm

## Future Upgrades

If the ICPI ledger needs future upgrades:
1. Download the latest ICRC ledger wasm from IC releases
2. Test on a local or testnet deployment first
3. Perform upgrade with appropriate argument (Upgrade variant)
4. Update this documentation with new hash and version
5. Commit the new wasm file with clear documentation

## Checksums

**SHA256**: `9ab4baff988fb961d5fc216a887514d55f98bc959b5931325a63087ee882d66f`

To verify file integrity:
```bash
sha256sum icrc1-ledger.wasm.gz
# Should output: 9ab4baff988fb961d5fc216a887514d55f98bc959b5931325a63087ee882d66f
```

**Note**: This hash matches the module hash of the deployed ICPI canister after upgrade, confirming authenticity.
