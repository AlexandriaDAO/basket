#!/bin/bash
# Kong Locker-specific deployment script
# Usage: ./deploy.sh [--network ic] [--fresh] [--backend-only|--frontend-only]

set -e

# Get the directory where this script is located (src/kong_locker)
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# Get the root directory (two levels up from src/kong_locker)
ROOT_DIR="$( cd "$SCRIPT_DIR/../.." && pwd )"

# Parse arguments
NETWORK="local"
FRESH_DEPLOY=false
DEPLOY_TARGET="all"

# Track deployment success
BACKEND_DEPLOYED=false
FRONTEND_DEPLOYED=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --fresh)
            FRESH_DEPLOY=true
            shift
            ;;
        --backend-only)
            DEPLOY_TARGET="backend"
            shift
            ;;
        --frontend-only)
            DEPLOY_TARGET="frontend"
            shift
            ;;
        --help)
            echo "Usage: ./deploy.sh [options]"
            echo ""
            echo "Options:"
            echo "  --network ic        Deploy to mainnet (default: local)"
            echo "  --fresh            Clean deployment (uninstalls code first)"
            echo "  --backend-only     Deploy only the backend"
            echo "  --frontend-only    Deploy only the frontend"
            echo "  --help             Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./deploy.sh                    # Deploy to local"
            echo "  ./deploy.sh --network ic       # Deploy to mainnet"
            echo "  ./deploy.sh --fresh            # Fresh local deployment"
            echo "  ./deploy.sh --network ic --backend-only  # Deploy only backend to mainnet"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Display deployment configuration
echo "================================================"
echo "Kong Locker Deployment"
echo "================================================"
echo "Network: $NETWORK"
echo "Target: $DEPLOY_TARGET"
echo "Fresh deploy: $FRESH_DEPLOY"
echo "Working from: $ROOT_DIR"

if [ "$NETWORK" == "ic" ]; then
    echo ""
    echo "Mainnet Canister IDs:"
    echo "  Backend:  eazgb-giaaa-aaaap-qqc2q-cai"
    echo "  Frontend: c6w56-taaaa-aaaai-atlma-cai"
fi

echo "================================================"
echo ""

# Change to root directory for all operations
cd "$ROOT_DIR"

# Check network connection
if [ "$NETWORK" == "local" ]; then
    # Check if dfx is running for local
    if ! dfx ping &>/dev/null; then
        echo "ERROR: dfx is not running. Please start it manually"
        echo "Run: dfx start --background --host 127.0.0.1:4943"
        exit 1
    fi
else
    # For mainnet, use daopad identity (no password required for Claude)
    echo "Switching to daopad identity for mainnet deployment..."
    dfx identity use daopad
    IDENTITY=$(dfx identity whoami)
    echo "Using identity: $IDENTITY"
    echo ""
    
    # Set environment variable to suppress the plaintext identity warning if using daopad
    export DFX_WARNING=-mainnet_plaintext_identity
fi

# Handle fresh deployment if requested
if [ "$FRESH_DEPLOY" = true ] && [ "$NETWORK" == "local" ]; then
    if [ "$DEPLOY_TARGET" == "all" ] || [ "$DEPLOY_TARGET" == "backend" ]; then
        echo "Fresh deployment requested - uninstalling backend canister..."
        dfx canister uninstall-code kong_locker 2>/dev/null || true
        echo "Backend canister uninstalled for fresh initialization"
    fi
fi

# Deploy backend
if [ "$DEPLOY_TARGET" == "all" ] || [ "$DEPLOY_TARGET" == "backend" ]; then
    echo ""
    echo "Building Kong Locker backend..."
    
    # Build Kong Locker backend
    if ! cargo build --target wasm32-unknown-unknown --release -p kong_locker --locked; then
        echo "❌ Kong Locker backend build failed!"
        exit 1
    fi
    
    # Extract candid interface for kong_locker
    echo "Extracting Candid interface for kong_locker..."
    if command -v candid-extractor &> /dev/null; then
        candid-extractor target/wasm32-unknown-unknown/release/kong_locker.wasm > src/kong_locker/kong_locker/kong_locker.did
        echo "✓ Candid interface extracted"
        echo "NOTE: lock_canister.did is NOT auto-generated to preserve module hash"
    else
        echo "❌ ERROR: candid-extractor not found!"
        echo "Please install with: cargo install candid-extractor"
        exit 1
    fi
    
    # Deploy Kong Locker backend with specified ID
    echo "Deploying kong_locker..."
    if [ "$NETWORK" == "ic" ]; then
        # Deploy to mainnet with specified ID
        echo "Deploying kong_locker to mainnet..."
        if dfx deploy --network ic kong_locker --specified-id eazgb-giaaa-aaaap-qqc2q-cai; then
            echo "✓ Kong Locker backend deployed successfully"
            BACKEND_DEPLOYED=true
        else
            echo "❌ Kong Locker backend deployment failed!"
            exit 1
        fi
    else
        # For local deployment, also use specified ID
        if dfx deploy kong_locker --specified-id eazgb-giaaa-aaaap-qqc2q-cai; then
            echo "✓ Kong Locker backend deployed successfully (local with specified ID)"
            BACKEND_DEPLOYED=true
        else
            echo "❌ Kong Locker backend deployment failed!"
            exit 1
        fi
    fi
fi

# Deploy frontend
if [ "$DEPLOY_TARGET" == "all" ] || [ "$DEPLOY_TARGET" == "frontend" ]; then
    echo ""
    echo "Building Kong Locker frontend..."
    cd src/kong_locker/kong_locker_frontend
    
    if ! npm install; then
        echo "❌ npm install failed for Kong Locker frontend!"
        cd "$ROOT_DIR"
        exit 1
    fi
    
    # Generate declarations for backend (if needed)
    echo "Generating backend declarations..."
    dfx generate kong_locker || echo "Warning: Failed to generate backend declarations"
    
    if ! npm run build; then
        echo "❌ Kong Locker frontend build failed!"
        cd "$ROOT_DIR"
        exit 1
    fi
    
    cd "$ROOT_DIR"
    
    echo "Deploying Kong Locker frontend..."
    if [ "$NETWORK" == "ic" ]; then
        echo "Deploying Kong Locker frontend to mainnet..."
        if dfx deploy --network ic kong_locker_frontend --specified-id c6w56-taaaa-aaaai-atlma-cai; then
            FRONTEND_ID=$(dfx canister --network ic id kong_locker_frontend 2>/dev/null || echo "c6w56-taaaa-aaaai-atlma-cai")
            echo "✓ Kong Locker frontend deployed successfully"
            echo "   Kong Locker URL: https://$FRONTEND_ID.icp0.io/"
            echo "   KongLocker.org: https://konglocker.org"
            FRONTEND_DEPLOYED=true
        else
            echo "❌ Kong Locker frontend deployment failed!"
            exit 1
        fi
    else
        if dfx deploy kong_locker_frontend --specified-id c6w56-taaaa-aaaai-atlma-cai; then
            FRONTEND_ID=$(dfx canister id kong_locker_frontend)
            echo "✓ Kong Locker frontend deployed successfully (local)"
            echo "   Local URL: http://$FRONTEND_ID.localhost:4943"
            FRONTEND_DEPLOYED=true
        else
            echo "❌ Kong Locker frontend deployment failed!"
            exit 1
        fi
    fi
fi

# Display deployment summary
echo ""
echo "================================================"
echo "Kong Locker Deployment Summary"
echo "================================================"

if [ "$NETWORK" == "ic" ]; then
    echo ""
    echo "Mainnet Status:"
    if [ "$BACKEND_DEPLOYED" == true ]; then
        BACKEND_ID=$(dfx canister --network ic id kong_locker 2>/dev/null || echo "eazgb-giaaa-aaaap-qqc2q-cai")
        echo "  ✓ Backend:  $BACKEND_ID"
    elif [ "$DEPLOY_TARGET" == "all" ] || [ "$DEPLOY_TARGET" == "backend" ]; then
        echo "  ❌ Backend:  Deployment failed"
    fi
    
    if [ "$FRONTEND_DEPLOYED" == true ]; then
        FRONTEND_ID=$(dfx canister --network ic id kong_locker_frontend 2>/dev/null || echo "c6w56-taaaa-aaaai-atlma-cai")
        echo "  ✓ Frontend: https://$FRONTEND_ID.icp0.io/"
        echo "             https://konglocker.org"
    elif [ "$DEPLOY_TARGET" == "all" ] || [ "$DEPLOY_TARGET" == "frontend" ]; then
        echo "  ❌ Frontend: Deployment failed"
    fi
    
    if [ "$BACKEND_DEPLOYED" == true ]; then
        echo ""
        echo "Testing Kong Locker backend:"
        echo "  dfx canister --network ic call kong_locker get_all_voting_powers"
        echo "  dfx canister --network ic call kong_locker get_lock_position '(principal \"YOUR_PRINCIPAL\")')"
    fi
else
    echo ""
    echo "Local Status:"
    if [ "$BACKEND_DEPLOYED" == true ]; then
        BACKEND_ID=$(dfx canister id kong_locker)
        echo "  ✓ Backend:  $BACKEND_ID"
        echo "    Candid UI: http://127.0.0.1:4943/?canisterId=$(dfx canister id __Candid_UI 2>/dev/null || echo 'vpyes-67777-77774-qaaeq-cai')&id=$BACKEND_ID"
    elif [ "$DEPLOY_TARGET" == "all" ] || [ "$DEPLOY_TARGET" == "backend" ]; then
        echo "  ❌ Backend: Deployment failed"
    fi
    
    if [ "$FRONTEND_DEPLOYED" == true ]; then
        FRONTEND_ID=$(dfx canister id kong_locker_frontend)
        echo "  ✓ Frontend: http://$FRONTEND_ID.localhost:4943"
    elif [ "$DEPLOY_TARGET" == "all" ] || [ "$DEPLOY_TARGET" == "frontend" ]; then
        echo "  ❌ Frontend: Deployment failed"
    fi
    
    if [ "$BACKEND_DEPLOYED" == true ]; then
        echo ""
        echo "Testing backend locally:"
        echo "  dfx canister call kong_locker get_all_voting_powers"
    fi
fi

# Show overall status
echo ""
if [ "$BACKEND_DEPLOYED" == false ] && [ "$FRONTEND_DEPLOYED" == false ]; then
    echo "⚠️  No components were successfully deployed. Please check the errors above."
elif [ "$BACKEND_DEPLOYED" == false ] || [ "$FRONTEND_DEPLOYED" == false ]; then
    echo "⚠️  Some components failed to deploy. Please check the warnings above."
else
    echo "✅ All requested Kong Locker components deployed successfully!"
fi

echo "================================================"