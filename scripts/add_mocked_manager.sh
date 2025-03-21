#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Default values
NETWORK="testnet"
SOURCE_KEY=""

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

# Validate source key
if [ -z "$SOURCE_KEY" ]; then
    echo -e "${RED}Error: Source key name is required${NC}"
    usage
    exit 1
fi

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

# Add specific manager
SPECIFIC_MANAGER_KEY="GD7IDV44QE7CN35M2QLSAISAYPSOSSZTV7LWMKBU5PKDS7NQKTFRZUTS"
add_manager "$SPECIFIC_MANAGER_KEY"
