#!/bin/bash

echo "Testing Kongswap swap method signatures..."
echo ""

# Test swap with minimal args (should fail but show expected type)
echo "1. Testing swap with minimal args to see error message:"
dfx canister --network ic call 2ipq2-uqaaa-aaaar-qailq-cai swap '(record {
  pay_token = "ckUSDT";
  pay_amount = 1000000;
  receive_token = "ALEX"
})'

echo ""
echo "2. Testing swap with complete args:"
dfx canister --network ic call 2ipq2-uqaaa-aaaar-qailq-cai swap '(record {
  pay_token = "ckUSDT";
  pay_amount = 1000000;
  pay_tx_id = null;
  receive_token = "ALEX";
  receive_amount = null;
  receive_address = null;
  max_slippage = opt 2.0;
  referred_by = null
})'
