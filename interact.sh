#!/bin/bash

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== ICPI Canister Interaction Tool ===${NC}\n"

show_menu() {
    echo -e "${BLUE}Choose a canister to interact with:${NC}"
    echo "1) Backend Canister (icpi_backend)"
    echo "2) ICPI Token (ICPI)"
    echo "3) Show all canister info"
    echo "4) Exit"
    echo
}

backend_menu() {
    echo -e "\n${YELLOW}Backend Canister Functions:${NC}"
    echo "1) greet(name) - Send a greeting"
    echo "2) Back to main menu"
    echo
    read -p "Select function: " choice

    case $choice in
        1)
            read -p "Enter name to greet: " name
            echo -e "${GREEN}Response:${NC}"
            dfx canister call icpi_backend greet "(\"$name\")"
            ;;
        2)
            return
            ;;
        *)
            echo "Invalid option"
            ;;
    esac
    echo
    read -p "Press Enter to continue..."
}

token_menu() {
    echo -e "\n${YELLOW}ICPI Token Functions:${NC}"
    echo "1) icrc1_name() - Get token name"
    echo "2) icrc1_symbol() - Get token symbol"
    echo "3) icrc1_decimals() - Get token decimals"
    echo "4) icrc1_total_supply() - Get total supply"
    echo "5) icrc1_fee() - Get transfer fee"
    echo "6) icrc1_balance_of(account) - Check balance"
    echo "7) Back to main menu"
    echo
    read -p "Select function: " choice

    case $choice in
        1)
            echo -e "${GREEN}Token Name:${NC}"
            dfx canister call ICPI icrc1_name '()'
            ;;
        2)
            echo -e "${GREEN}Token Symbol:${NC}"
            dfx canister call ICPI icrc1_symbol '()'
            ;;
        3)
            echo -e "${GREEN}Token Decimals:${NC}"
            dfx canister call ICPI icrc1_decimals '()'
            ;;
        4)
            echo -e "${GREEN}Total Supply:${NC}"
            dfx canister call ICPI icrc1_total_supply '()'
            ;;
        5)
            echo -e "${GREEN}Transfer Fee:${NC}"
            dfx canister call ICPI icrc1_fee '()'
            ;;
        6)
            read -p "Enter principal (or press Enter for your own): " principal
            if [ -z "$principal" ]; then
                principal=$(dfx identity get-principal)
            fi
            echo -e "${GREEN}Balance:${NC}"
            dfx canister call ICPI icrc1_balance_of "(record { owner = principal \"$principal\"; subaccount = null })"
            ;;
        7)
            return
            ;;
        *)
            echo "Invalid option"
            ;;
    esac
    echo
    read -p "Press Enter to continue..."
}

show_info() {
    echo -e "\n${GREEN}=== Deployed Canisters ===${NC}\n"

    echo -e "${BLUE}Frontend Application:${NC}"
    echo "  URL: http://qhlmp-5aaaa-aaaam-qd4jq-cai.localhost:4943/"
    echo "  Canister ID: qhlmp-5aaaa-aaaam-qd4jq-cai"
    echo

    echo -e "${BLUE}Backend Canister:${NC}"
    echo "  Canister ID: ehyav-lqaaa-aaaap-qqc2a-cai"
    echo "  Status: $(dfx canister status icpi_backend 2>/dev/null | grep "Status:" || echo "Unknown")"
    echo

    echo -e "${BLUE}ICPI Token:${NC}"
    echo "  Canister ID: es7ry-kyaaa-aaaap-qqczq-cai"
    echo "  Status: $(dfx canister status ICPI 2>/dev/null | grep "Status:" || echo "Unknown")"
    echo

    echo -e "${YELLOW}Current Identity:${NC} $(dfx identity whoami)"
    echo -e "${YELLOW}Principal:${NC} $(dfx identity get-principal)"
    echo
    read -p "Press Enter to continue..."
}

# Main loop
while true; do
    clear
    show_menu
    read -p "Enter your choice: " choice

    case $choice in
        1)
            backend_menu
            ;;
        2)
            token_menu
            ;;
        3)
            show_info
            ;;
        4)
            echo -e "${GREEN}Goodbye!${NC}"
            exit 0
            ;;
        *)
            echo "Invalid option"
            read -p "Press Enter to continue..."
            ;;
    esac
done