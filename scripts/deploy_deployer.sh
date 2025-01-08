#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
NETWORK="testnet"
SOURCE_KEY=""
FUND_ACCOUNT=false

# Print usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -n, --network <testnet|mainnet>  Network to deploy to (default: testnet)"
    echo "  -s, --source <key_name>          Source account key name"
    echo "  -f, --fund                       Fund the account using friendbot (testnet only)"
    echo "  -h, --help                       Show this help message"
    echo
    echo "Example:"
    echo "  $0 --network testnet --source alice --fund"
    echo "  $0 -n mainnet -s my_key"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--network)
            NETWORK="$2"
            shift 2
            ;;
        -s|--source)
            SOURCE_KEY="$2"
            shift 2
            ;;
        -f|--fund)
            FUND_ACCOUNT=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            usage
            exit 1
            ;;
    esac
done

# Validate inputs
if [[ -z "$SOURCE_KEY" ]]; then
    echo -e "${RED}Error: Source key name is required${NC}"
    usage
    exit 1
fi

if [[ "$NETWORK" != "testnet" && "$NETWORK" != "mainnet" ]]; then
    echo -e "${RED}Error: Network must be either 'testnet' or 'mainnet'${NC}"
    usage
    exit 1
fi

if [[ "$NETWORK" == "mainnet" && "$FUND_ACCOUNT" == true ]]; then
    echo -e "${RED}Error: Cannot fund account on mainnet${NC}"
    exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if stellar CLI is installed
if ! command_exists stellar; then
    echo -e "${RED}Error: stellar CLI is not installed${NC}"
    echo "Please install it using: cargo install stellar-cli"
    exit 1
fi

# Build the contracts
echo -e "${YELLOW}Building contracts...${NC}"
cargo build --target wasm32-unknown-unknown --release

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Build failed${NC}"
    exit 1
fi

# Create directory for deployment artifacts
mkdir -p .deploy

# Generate and fund account if requested
if [[ "$FUND_ACCOUNT" == true && "$NETWORK" == "testnet" ]]; then
    echo -e "${YELLOW}Generating and funding account for $SOURCE_KEY...${NC}"
    stellar keys generate --global "$SOURCE_KEY" --network testnet --fund
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to generate and fund account${NC}"
        exit 1
    fi
fi

# Get account address
ACCOUNT_ADDRESS=$(stellar keys address "$SOURCE_KEY")
echo -e "${GREEN}Using account: $ACCOUNT_ADDRESS${NC}"

# Deploy the contract
echo -e "${YELLOW}Deploying deployer contract to $NETWORK...${NC}"
RESULT=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/deployer.wasm \
    --source "$SOURCE_KEY" \
    --network "$NETWORK")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Deployment failed${NC}"
    echo "$RESULT"
    exit 1
fi

# Extract contract ID from result
CONTRACT_ID=$RESULT

# Save deployment information
echo "NETWORK=$NETWORK" > .deploy/deployer.env
echo "CONTRACT_ID=$CONTRACT_ID" >> .deploy/deployer.env
echo "DEPLOYER_ADDRESS=$ACCOUNT_ADDRESS" >> .deploy/deployer.env
echo "DEPLOYMENT_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")" >> .deploy/deployer.env

echo -e "${GREEN}Deployment successful!${NC}"
echo "Contract ID: $CONTRACT_ID"
echo "Deployment information saved to .deploy/deployer.env"