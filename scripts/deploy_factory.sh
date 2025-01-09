#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Default values
NETWORK="testnet"
SOURCE_KEY=""

# Load deployer contract info
if [ ! -f .deploy/deployer.env ]; then
    echo -e "${RED}Error: Deployer contract info not found. Run deploy_deployer.sh first${NC}"
    exit 1
fi

source .deploy/deployer.env

# Validate deployer contract ID
if [ -z "$CONTRACT_ID" ]; then
    echo -e "${RED}Error: Deployer contract ID not found in .deploy/deployer.env${NC}"
    echo "Please make sure deploy_deployer.sh was run successfully"
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

# Build and prepare contracts
echo -e "${YELLOW}Building contracts...${NC}"
cargo build --target wasm32-unknown-unknown --release

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Build failed${NC}"
    exit 1
fi

# Get account address
ADMIN_ADDRESS=$(stellar keys address "$SOURCE_KEY")
echo -e "${GREEN}Using admin address: $ADMIN_ADDRESS${NC}"

# Install scorer WASM first
echo -e "${YELLOW}Installing scorer WASM...${NC}"
SCORER_RESULT=$(stellar contract install \
    --wasm wasm/scorer.wasm \
    --source "$SOURCE_KEY" \
    --network "$NETWORK" \
    --ignore-checks)

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to install scorer WASM${NC}"
    echo "$SCORER_RESULT"
    exit 1
fi

SCORER_HASH=$SCORER_RESULT
echo -e "${GREEN}scorer hash: $SCORER_HASH${NC}"

# Then install factory WASM
echo -e "${YELLOW}Installing factory WASM...${NC}"
FACTORY_RESULT=$(stellar contract install \
    --wasm wasm/scorer_factory.wasm \
    --source "$SOURCE_KEY" \
    --network "$NETWORK" \
    --ignore-checks)

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to install factory WASM${NC}"
    echo "$FACTORY_RESULT"
    exit 1
fi

FACTORY_HASH=$FACTORY_RESULT

# Finally deploy the factory using the deployer
echo -e "${YELLOW}Deploying factory contract...${NC}"
DEPLOY_RESULT=$(stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$SOURCE_KEY" \
    --network "$NETWORK" \
    -- \
    deploy \
    --deployer "$ADMIN_ADDRESS" \
    --wasm-hash "$FACTORY_HASH" \
    --salt "$(openssl rand -hex 32)" \
    --init-fn "initialize" \
    --init-args "[{\"address\":\"$ADMIN_ADDRESS\"},{\"bytes\":\"$SCORER_HASH\"}]")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Factory deployment failed${NC}"
    echo "$DEPLOY_RESULT"
    exit 1
fi

# Extract the factory address from the result array
FACTORY_ADDRESS=$(echo $DEPLOY_RESULT | jq -r '.[0]')

# Save factory deployment information
echo "NETWORK=$NETWORK" > .deploy/factory.env
echo "FACTORY_ADDRESS=$FACTORY_ADDRESS" >> .deploy/factory.env
echo "ADMIN_ADDRESS=$ADMIN_ADDRESS" >> .deploy/factory.env
echo "DEPLOYMENT_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")" >> .deploy/factory.env

echo -e "${GREEN}Factory deployment successful!${NC}"
echo "Factory Address: $FACTORY_ADDRESS"
echo "Deployment information saved to .deploy/factory.env"

# Add manager
echo -e "${YELLOW}Adding manager...${NC}"
ADD_MANAGER_RESULT=$(stellar contract invoke \
    --id "$FACTORY_ADDRESS" \
    --source "$SOURCE_KEY" \
    --network "$NETWORK" \
    -- \
    add_manager \
    --caller "$ADMIN_ADDRESS" \
    --manager "$ADMIN_ADDRESS")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to add manager${NC}"
    echo "$ADD_MANAGER_RESULT"
    exit 1
fi

echo -e "${GREEN}Successfully added manager${NC}"

# Remove manager
echo -e "${YELLOW}Removing manager...${NC}"
REMOVE_MANAGER_RESULT=$(stellar contract invoke \
    --id "$FACTORY_ADDRESS" \
    --source "$SOURCE_KEY" \
    --network "$NETWORK" \
    -- \
    remove_manager \
    --caller "$ADMIN_ADDRESS" \
    --manager "$ADMIN_ADDRESS")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to remove manager${NC}"
    echo "$REMOVE_MANAGER_RESULT"
    exit 1
fi

echo -e "${GREEN}Successfully removed manager${NC}" 


# Add manager
echo -e "${YELLOW}Adding manager...${NC}"
ADD_MANAGER_RESULT=$(stellar contract invoke \
    --id "$FACTORY_ADDRESS" \
    --source "$SOURCE_KEY" \
    --network "$NETWORK" \
    -- \
    add_manager \
    --caller "$ADMIN_ADDRESS" \
    --manager "$ADMIN_ADDRESS")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to add manager${NC}"
    echo "$ADD_MANAGER_RESULT"
    exit 1
fi

echo -e "${GREEN}Successfully added manager${NC}"