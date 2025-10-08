#!/bin/bash

ACCOUNT="076fc336ae77b257baaf98489b9e6b06aa0290e0d8c6ea09e3cc6147"

dfx deploy icp_ledger_canister --argument "(variant { Init = record {
  minting_account = \"$ACCOUNT\";
  initial_values = vec {
    record {
      \"$ACCOUNT\";
      record {
        e8s = 100_000_000_000 : nat64;
      };
    };
  };
  send_whitelist = vec {};
  transfer_fee = opt record {
    e8s = 10_000 : nat64;
  };
  token_symbol = opt \"LICP\";
  token_name = opt \"Local ICP\";
} })"