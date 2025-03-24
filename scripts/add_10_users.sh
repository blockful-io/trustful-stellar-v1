#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Default values
NETWORK="testnet"
SOURCE_KEY=""
SCORER_ADDRESS=""
FUND_ACCOUNT=true

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

# Function to ensure the account is funded
ensure_account_funded() {
    local address=$1
    local keyname=$2
    
    # First, try to see if account already exists
    local query_result=$(stellar query account "$address" --network "$NETWORK" 2>&1)
    
    if [[ "$query_result" == *"Account not found"* ]]; then
        echo -e "${YELLOW}Account doesn't exist on $NETWORK. Funding it...${NC}"
        # Account doesn't exist, fund it
        stellar keys fund "$address" --network "$NETWORK"
        if [ $? -ne 0 ]; then
            echo -e "${RED}Error: Failed to fund new account $address on $NETWORK${NC}"
            exit 1
        fi
        echo -e "${GREEN}Successfully funded account on $NETWORK${NC}"
    else
        # Account exists, but let's make sure it has enough funds
        # We'll fund it again anyway to ensure it has enough XLM
        echo -e "${YELLOW}Account exists but might need more funds. Adding more funds...${NC}"
        stellar keys fund "$address" --network "$NETWORK" 2>/dev/null || true
        echo -e "${GREEN}Added funds to existing account on $NETWORK${NC}"
    fi
}

# Function to generate and fund a key if it doesn't exist
generate_and_fund_key_if_needed() {
    local key_name=$1
    
    # Check if key exists locally
    if stellar keys ls | grep -q "$key_name"; then
        local address=$(stellar keys address "$key_name")
        echo -e "${GREEN}Key $key_name exists locally with address: $address${NC}"
        
        # Ensure this account has funds
        ensure_account_funded "$address" "$key_name"
    else
        echo -e "${YELLOW}Generating new key for $key_name...${NC}"
        # Generate new key
        stellar keys generate --global "$key_name"
        if [ $? -ne 0 ]; then
            echo -e "${RED}Error: Failed to generate key for $key_name${NC}"
            exit 1
        fi
        
        local address=$(stellar keys address "$key_name")
        echo -e "${GREEN}Generated key $key_name with address: $address${NC}"
        
        # Fund the newly created key
        ensure_account_funded "$address" "$key_name"
    fi
}

# Function to add a user
add_user() {
    local user_key=$1
    echo -e "${YELLOW}Adding user $user_key to scorer contract...${NC}"
    ADD_USER_RESULT=$(stellar contract invoke \
        --id "$SCORER_ADDRESS" \
        --source "$user_key" \
        --network "$NETWORK" \
        -- \
        add_user \
        --user "$user_key")

    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to add user $user_key to scorer${NC}"
        echo "$ADD_USER_RESULT"
        exit 1
    fi

    echo -e "${GREEN}Successfully added user $user_key to scorer${NC}"
}

# Function to add a manager
add_manager() {
    local manager_key=$1
    echo -e "${YELLOW}Adding manager $manager_key to scorer contract...${NC}"
    ADD_MANAGER_RESULT=$(stellar contract invoke \
        --id "$SCORER_ADDRESS" \
        --source "$SOURCE_KEY" \
        --network "$NETWORK" \
        -- \
        add_manager \
        --new_manager "$manager_key" \
        --sender "$SOURCE_KEY")

    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to add manager $manager_key to scorer${NC}"
        echo "$ADD_MANAGER_RESULT"
        exit 1
    fi

    echo -e "${GREEN}Successfully added manager $manager_key to scorer${NC}"
}

# Add 10 users
for i in {1..10}; do
    USER_KEY="_user_$i"
    generate_and_fund_key_if_needed "$USER_KEY"
    add_user "$USER_KEY"
done

# Add 2 managers
for i in {1..2}; do
    MANAGER_KEY="_manager_$i"
    generate_and_fund_key_if_needed "$MANAGER_KEY"
    add_manager "$MANAGER_KEY"
done