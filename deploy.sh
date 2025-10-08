#!/bin/bash

# Default values
NETWORK="local"
DEPLOY_FRONTEND=true
DEPLOY_BACKEND=true
DEPLOY_TOKEN=false
BUILD_ONLY=false
REINSTALL=false

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --ic)
            NETWORK="ic"
            shift
            ;;
        --local)
            NETWORK="local"
            shift
            ;;
        --frontend-only)
            DEPLOY_FRONTEND=true
            DEPLOY_BACKEND=false
            DEPLOY_TOKEN=false
            shift
            ;;
        --backend-only)
            DEPLOY_FRONTEND=false
            DEPLOY_BACKEND=true
            DEPLOY_TOKEN=false
            shift
            ;;
        --token-only)
            DEPLOY_FRONTEND=false
            DEPLOY_BACKEND=false
            DEPLOY_TOKEN=true
            shift
            ;;
        --with-token)
            DEPLOY_TOKEN=true
            shift
            ;;
        --build-only)
            BUILD_ONLY=true
            shift
            ;;
        --reinstall)
            REINSTALL=true
            shift
            ;;
        --help)
            echo "Usage: ./deploy.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --network <name>   Specify the network (local or ic) [default: local]"
            echo "  --ic               Deploy to mainnet (shorthand for --network ic)"
            echo "  --local            Deploy to local (shorthand for --network local)"
            echo "  --frontend-only    Deploy only the frontend canister"
            echo "  --backend-only     Deploy only the backend canister"
            echo "  --token-only       Deploy only the ICPI token canister"
            echo "  --with-token       Include ICPI token in deployment"
            echo "  --build-only       Only build, do not deploy"
            echo "  --reinstall        Reinstall canisters (delete and recreate)"
            echo "  --help             Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./deploy.sh                        # Deploy frontend and backend to local"
            echo "  ./deploy.sh --with-token           # Deploy all including token to local"
            echo "  ./deploy.sh --ic --with-token      # Deploy all including token to mainnet"
            echo "  ./deploy.sh --token-only           # Deploy only token to local"
            echo "  ./deploy.sh --ic --backend-only    # Deploy only backend to mainnet"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Function to print colored messages
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Start deployment
print_info "Starting deployment process..."
print_info "Network: $NETWORK"
print_info "Deploy Frontend: $DEPLOY_FRONTEND"
print_info "Deploy Backend: $DEPLOY_BACKEND"
print_info "Deploy Token: $DEPLOY_TOKEN"

# Check if dfx is installed
if ! command -v dfx &> /dev/null; then
    print_error "dfx is not installed. Please install it first."
    exit 1
fi

# If deploying to local, ensure local replica is running
if [ "$NETWORK" = "local" ]; then
    if ! dfx ping &> /dev/null; then
        print_warn "Local replica is not running. Starting it now..."
        dfx start --clean --background
        sleep 5
    else
        print_info "Local replica is already running"
    fi
fi

# Build frontend if needed
if [ "$DEPLOY_FRONTEND" = true ]; then
    print_info "Building frontend..."
    cd src/icpi_frontend || exit 1
    npm install
    npm run build
    cd ../.. || exit 1
fi

# Build backend if needed
if [ "$DEPLOY_BACKEND" = true ]; then
    print_info "Building backend..."
    cargo build --release --target wasm32-unknown-unknown --manifest-path=src/icpi_backend/Cargo.toml
fi

# Exit if build-only
if [ "$BUILD_ONLY" = true ]; then
    print_info "Build complete (--build-only flag was set)"
    exit 0
fi

# Deploy canisters
DEPLOY_ARGS=""

if [ "$NETWORK" = "ic" ]; then
    DEPLOY_ARGS="--network ic"
else
    DEPLOY_ARGS=""
fi

# Handle reinstall mode
if [ "$REINSTALL" = true ]; then
    print_warn "Reinstall mode: removing existing canisters..."

    if [ "$DEPLOY_BACKEND" = true ]; then
        dfx canister stop icpi_backend $DEPLOY_ARGS 2>/dev/null || true
        dfx canister delete icpi_backend $DEPLOY_ARGS 2>/dev/null || true
    fi

    if [ "$DEPLOY_FRONTEND" = true ]; then
        dfx canister stop icpi_frontend $DEPLOY_ARGS 2>/dev/null || true
        dfx canister delete icpi_frontend $DEPLOY_ARGS 2>/dev/null || true
    fi

    if [ "$DEPLOY_TOKEN" = true ]; then
        dfx canister stop ICPI $DEPLOY_ARGS 2>/dev/null || true
        dfx canister delete ICPI $DEPLOY_ARGS 2>/dev/null || true
    fi
fi

# Create canisters if needed
if [ "$DEPLOY_BACKEND" = true ]; then
    print_info "Creating backend canister..."
    dfx canister create icpi_backend $DEPLOY_ARGS --specified-id ev6xm-haaaa-aaaap-qqcza-cai 2>/dev/null || true
fi

if [ "$DEPLOY_FRONTEND" = true ]; then
    print_info "Creating frontend canister..."
    dfx canister create icpi_frontend $DEPLOY_ARGS --specified-id qhlmp-5aaaa-aaaam-qd4jq-cai 2>/dev/null || true
fi

if [ "$DEPLOY_TOKEN" = true ]; then
    print_info "Creating ICPI token canister..."
    dfx canister create ICPI $DEPLOY_ARGS --specified-id l6lep-niaaa-aaaap-qqeda-cai 2>/dev/null || true
fi

# Deploy canisters
CANISTERS_TO_DEPLOY=""

if [ "$DEPLOY_BACKEND" = true ]; then
    CANISTERS_TO_DEPLOY="icpi_backend"
fi

if [ "$DEPLOY_FRONTEND" = true ]; then
    if [ -z "$CANISTERS_TO_DEPLOY" ]; then
        CANISTERS_TO_DEPLOY="icpi_frontend"
    else
        CANISTERS_TO_DEPLOY="$CANISTERS_TO_DEPLOY icpi_frontend"
    fi
fi

# Deploy token separately with initialization arguments
if [ "$DEPLOY_TOKEN" = true ]; then
    print_info "Deploying ICPI token with initialization arguments..."

    # Convert SVG logo to base64
    TOKEN_LOGO=$(base64 -w 0 token-logo.svg 2>/dev/null || base64 token-logo.svg)

    # Deploy token with initialization arguments for ICRC1 ledger format
    dfx deploy ICPI $DEPLOY_ARGS --argument "(variant { Init =
    record {
         minting_account = record { owner = principal \"ev6xm-haaaa-aaaap-qqcza-cai\"; subaccount = null };
         fee_collector_account = null;
         transfer_fee = 10_000;
         decimals = opt 8;
         max_memo_length = opt 32;
         token_symbol = \"ICPI\";
         token_name = \"Internet Computer Portfolio Index\";
         metadata = vec {
            record { \"icrc1:symbol\"; variant { Text = \"ICPI\" } };
            record { \"icrc1:name\"; variant { Text = \"Internet Computer Portfolio Index\" } };
            record { \"icrc1:description\"; variant { Text = \"A redeemable basket of Liquid ICP Assets\" } };
            record { \"icrc1:decimals\"; variant { Nat = 8 } };
            record { \"icrc1:fee\"; variant { Nat = 10_000 } };
            record { \"icrc1:logo\"; variant { Text = \"data:image/svg+xml;base64,${TOKEN_LOGO}\" } };
         };
         initial_balances = vec {};
         feature_flags = opt record {
            icrc2 = true;
         };
         archive_options = record {
             num_blocks_to_archive = 1000;
             trigger_threshold = 2000;
             max_message_size_bytes = null;
             controller_id = principal \"ev6xm-haaaa-aaaap-qqcza-cai\";
             more_controller_ids = null;
             cycles_for_archive_creation = null;
             node_max_memory_size_bytes = null;
             max_transactions_per_response = null;
         };
     }
    })"

    if [ $? -eq 0 ]; then
        print_info "ICPI token deployed successfully!"
        if [ "$NETWORK" = "ic" ]; then
            print_info "ICPI Token canister ID: l6lep-niaaa-aaaap-qqeda-cai"
        else
            print_info "ICPI Token canister ID: l6lep-niaaa-aaaap-qqeda-cai (local)"
        fi
    else
        print_error "Failed to deploy ICPI token!"
        exit 1
    fi
fi

if [ -n "$CANISTERS_TO_DEPLOY" ]; then
    print_info "Deploying canisters: $CANISTERS_TO_DEPLOY"
    # Deploy each canister separately to avoid argument errors
    for CANISTER in $CANISTERS_TO_DEPLOY; do
        dfx deploy $CANISTER $DEPLOY_ARGS
    done

    if [ $? -eq 0 ]; then
        print_info "Deployment successful!"
        echo ""
        print_info "=== Canister URLs ==="

        # Get the Candid UI canister ID for local network
        if [ "$NETWORK" = "local" ]; then
            CANDID_UI=$(dfx canister id __Candid_UI 2>/dev/null || echo "uqqxf-5h777-77774-qaaaa-cai")
        fi

        # Print canister URLs
        if [ "$DEPLOY_FRONTEND" = true ]; then
            echo ""
            print_info "Frontend Application:"
            if [ "$NETWORK" = "ic" ]; then
                echo "  🌐 https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io"
            else
                echo "  🌐 http://qhlmp-5aaaa-aaaam-qd4jq-cai.localhost:4943/"
                echo "  🌐 http://127.0.0.1:4943/?canisterId=qhlmp-5aaaa-aaaam-qd4jq-cai"
            fi
        fi

        if [ "$DEPLOY_BACKEND" = true ]; then
            echo ""
            print_info "Backend Canister (Candid UI):"
            if [ "$NETWORK" = "ic" ]; then
                echo "  🔧 https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=ev6xm-haaaa-aaaap-qqcza-cai"
            else
                echo "  🔧 http://127.0.0.1:4943/?canisterId=$CANDID_UI&id=ev6xm-haaaa-aaaap-qqcza-cai"
            fi
            echo "  📝 Canister ID: ev6xm-haaaa-aaaap-qqcza-cai"
        fi

        if [ "$DEPLOY_TOKEN" = true ]; then
            echo ""
            print_info "ICPI Token Canister (Candid UI):"
            if [ "$NETWORK" = "ic" ]; then
                echo "  💰 https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=l6lep-niaaa-aaaap-qqeda-cai"
            else
                echo "  💰 http://127.0.0.1:4943/?canisterId=$CANDID_UI&id=l6lep-niaaa-aaaap-qqeda-cai"
            fi
            echo "  📝 Canister ID: l6lep-niaaa-aaaap-qqeda-cai"
        fi

        echo ""
    else
        print_error "Deployment failed!"
        exit 1
    fi
fi

print_info "Done!"