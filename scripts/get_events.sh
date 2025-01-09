#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Contract ID to query
CONTRACT_ID="CD64GYZHU57XUKUH2FUOOAWM4TWSQ4NCDE2NGJMOQEV3N5VSYPEQWWYK"

# Soroban RPC endpoint for testnet
RPC_URL="https://soroban-testnet.stellar.org"

# Updated valid start ledger from error message
START_LEDGER=498000

echo -e "${YELLOW}Fetching events for contract: $CONTRACT_ID${NC}"
echo -e "${YELLOW}Using start ledger: $START_LEDGER${NC}"

# Get events using curl
EVENTS=$(curl -s -X POST "$RPC_URL" \
    -H "Content-Type: application/json" \
    -d "{
        \"jsonrpc\": \"2.0\",
        \"id\": 1,
        \"method\": \"getEvents\",
        \"params\": {
            \"startLedger\": $START_LEDGER,
            \"filters\": [{
                \"type\": \"contract\",
                \"contractIds\": [\"$CONTRACT_ID\"]
            }],
            \"pagination\": {
                \"limit\": 1000
            }
        }
    }")

# Check for error in response
if echo "$EVENTS" | jq -e '.error' > /dev/null; then
    echo -e "${RED}Error: Failed to fetch events${NC}"
    echo "$EVENTS" | jq '.'
    exit 1
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is not installed${NC}"
    echo "Please install jq to format the JSON output"
    echo "Raw events:"
    echo "$EVENTS"
    exit 1
fi

# Pretty print the events
echo "$EVENTS" | jq '.'

# Only show success message if we actually got events
if [ "$(echo "$EVENTS" | jq '.result.events | length')" -gt 0 ]; then
    echo -e "${GREEN}Found $(echo "$EVENTS" | jq '.result.events | length') events!${NC}"
else
    echo -e "${YELLOW}No events found for this contract${NC}"
fi 