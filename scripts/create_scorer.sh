#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Default values
NETWORK="testnet"
SOURCE_KEY=""

# Load factory contract info
if [ ! -f .deploy/factory.env ]; then
    echo -e "${RED}Error: Factory contract info not found. Run deploy_factory.sh first${NC}"
    exit 1
fi

source .deploy/factory.env

# Validate factory address
if [ -z "$FACTORY_ADDRESS" ]; then
    echo -e "${RED}Error: Factory address not found in .deploy/factory.env${NC}"
    echo "Please make sure deploy_factory.sh was run successfully"
    exit 1
fi

# Print usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -n, --network <testnet|mainnet>  Network to deploy to (default: testnet)"
    echo "  -s, --source <key_name>          Source account key name (required)"
    echo "  -h, --help                       Show this help message"
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

# Check if stellar CLI is installed
if ! command -v stellar &> /dev/null; then
    echo -e "${RED}Error: stellar CLI is not installed${NC}"
    echo "Please install it using: cargo install stellar-cli"
    exit 1
fi

# Get account address
ADMIN_ADDRESS=$(stellar keys address "$SOURCE_KEY")
echo -e "${GREEN}Using admin address: $ADMIN_ADDRESS${NC}"

# Generate a random salt for unique contract address
SALT=$(openssl rand -hex 32)

# Create initial badge map - Using proper struct format
BADGE_MAP="[[{\"u32\":1},{\"vec\":[{\"string\":\"Initial Badge\"},{\"address\":\"$ADMIN_ADDRESS\"},{\"u32\":100}]}]]"

# Create the init args
INIT_ARGS="[{\"address\":\"$ADMIN_ADDRESS\"},{\"map\":$BADGE_MAP}]"

# Create the scorer contract
echo -e "${YELLOW}Creating scorer contract...${NC}"
CREATE_RESULT=$(stellar contract invoke \
    --id "$FACTORY_ADDRESS" \
    --source "$SOURCE_KEY" \
    --network "$NETWORK" \
    -- \
    create_scorer \
    --deployer "$ADMIN_ADDRESS" \
    --salt "$SALT" \
    --init-fn "initialize" \
    --init-args "$INIT_ARGS")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to create scorer contract${NC}"
    echo "$CREATE_RESULT"
    exit 1
fi

SCORER_ADDRESS=$CREATE_RESULT

# Save scorer deployment information
echo "NETWORK=$NETWORK" > .deploy/scorer.env
echo "SCORER_ADDRESS=$SCORER_ADDRESS" >> .deploy/scorer.env
echo "ADMIN_ADDRESS=$ADMIN_ADDRESS" >> .deploy/scorer.env
echo "DEPLOYMENT_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")" >> .deploy/scorer.env

echo -e "${GREEN}Scorer contract creation successful!${NC}"
echo "Scorer Address: $SCORER_ADDRESS"
echo "Deployment information saved to .deploy/scorer.env" 