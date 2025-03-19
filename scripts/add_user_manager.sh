#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Default values
NETWORK="testnet"
SOURCE_KEY=""
TARGET_KEY=""

# Load scorer contract info
if [ ! -f .deploy/scorer.env ]; then
    echo -e "${RED}Error: Scorer contract info not found. Run create_scorer.sh first${NC}"
    exit 1
fi

source .deploy/scorer.env

# Validate scorer address
if [ -z "$SCORER_ADDRESS" ]; then
    echo -e "${RED}Error: Scorer address not found in .deploy/scorer.env${NC}"
    echo "Please make sure create_scorer.sh was run successfully"
    exit 1
fi

# Print usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -n, --network <testnet|mainnet>  Network to deploy to (default: testnet)"
    echo "  -s, --source <key_name>          Source account key name (required)"
    echo "  -t, --target <key_name>          Target account key name to add as user/manager (required)"
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
        -t|--target)
            TARGET_KEY="$2"
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

if [[ -z "$TARGET_KEY" ]]; then
    echo -e "${RED}Error: Target key name is required${NC}"
    usage
    exit 1
fi

if [[ "$NETWORK" != "testnet" && "$NETWORK" != "mainnet" ]]; then
    echo -e "${RED}Error: Network must be either 'testnet' or 'mainnet'${NC}"
    usage
    exit 1
fi

# Get account addresses
ADMIN_ADDRESS=$(stellar keys address "$SOURCE_KEY")
TARGET_ADDRESS=$(stellar keys address "$TARGET_KEY")
echo -e "${GREEN}Using admin address: $ADMIN_ADDRESS${NC}"
echo -e "${GREEN}Using target address: $TARGET_ADDRESS${NC}"

# Add manager to the scorer contract
echo -e "${YELLOW}Adding manager to scorer contract...${NC}"
ADD_MANAGER_RESULT=$(stellar contract invoke \
    --id "$SCORER_ADDRESS" \
    --source "$SOURCE_KEY" \
    --network "$NETWORK" \
    -- \
    add_manager \
    --sender "$ADMIN_ADDRESS" \
    --new_manager "$TARGET_ADDRESS")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to add manager to scorer${NC}"
    echo "$ADD_MANAGER_RESULT"
    exit 1
fi

echo -e "${GREEN}Successfully added manager to scorer${NC}"

# Add user to the scorer contract
echo -e "${YELLOW}Adding user to scorer contract...${NC}"
ADD_USER_RESULT=$(stellar contract invoke \
    --id "$SCORER_ADDRESS" \
    --source "$TARGET_KEY" \
    --network "$NETWORK" \
    -- \
    add_user \
    --user "$TARGET_ADDRESS")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to add user to scorer${NC}"
    echo "$ADD_USER_RESULT"
    exit 1
fi

echo -e "${GREEN}Successfully added user to scorer${NC}"

# Remove manager from scorer contract
echo -e "${YELLOW}Removing manager from scorer contract...${NC}"
REMOVE_MANAGER_RESULT=$(stellar contract invoke \
    --id "$SCORER_ADDRESS" \
    --source "$SOURCE_KEY" \
    --network "$NETWORK" \
    -- \
    remove_manager \
    --sender "$ADMIN_ADDRESS" \
    --manager_to_remove "$TARGET_ADDRESS")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to remove manager from scorer${NC}"
    echo "$REMOVE_MANAGER_RESULT"
    exit 1
fi

echo -e "${GREEN}Successfully removed manager from scorer${NC}"

# Remove user from scorer contract
echo -e "${YELLOW}Removing user from scorer contract...${NC}"
REMOVE_USER_RESULT=$(stellar contract invoke \
    --id "$SCORER_ADDRESS" \
    --source "$TARGET_KEY" \
    --network "$NETWORK" \
    -- \
    remove_user \
    --user "$TARGET_ADDRESS")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to remove user from scorer${NC}"
    echo "$REMOVE_USER_RESULT"
    exit 1
fi

echo -e "${GREEN}Successfully removed user from scorer${NC}"