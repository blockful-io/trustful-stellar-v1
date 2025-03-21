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

BADGE_MAP="["
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0001\"},{\"address\":\"GDQMNGUDOSMCCN6MD52DPXX4ACECXVODFK2NQQGFXYLGXJFZ2LEEIY35\"}]},{\"u32\":3}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0101\"},{\"address\":\"GDCBRDWFCCS7MY7BJREOLLEXYTWUFDGIEZZOXPD7EALXHDFCQY3QL7AE\"}]},{\"u32\":10}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0102\"},{\"address\":\"GB76OH7Z2N4BWUZREZTJ36WUDMAFCDYISM26MCNTJWZQ63D3NROYKBOZ\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0103\"},{\"address\":\"GDOPCKCQEXJFVXLWHHGVIVXYT5FKKYWRPPLPKK32RPNCKXURKZUVPJTG\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0104\"},{\"address\":\"GCEUS7FJMZWV5MGYQRUF6SR3T3IQVPASWXMT6REV5CB76N4RGOWST4JU\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0105\"},{\"address\":\"GABPJ5EWLQBVTVMHJPMF7DZ5OOZXR3JRTA3EBZEMJII6C57UEQDP4HNO\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0201\"},{\"address\":\"GBOKD6LRD3TRNHCEOBLD44MPE3KXUF3NBXNBH2IYZHIHGG7KXZ3VANS7\"}]},{\"u32\":10}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0202\"},{\"address\":\"GBYKVKGE3Z3YZFR4X4OXV4R5U5VZ52BZKFVRO3QOD62CQDBHJ6EN4RBK\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0203\"},{\"address\":\"GAEQEWJ4SFB5U2HLW6RJA3Z2VWYKOTFVGRROFH3MNIOGDWWH5LDS3UG6\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0204\"},{\"address\":\"GD42KD354VPRHU3ZD3T6UEVXCABJU242H7MUEWNW7CNQTPSLK3AOIOGH\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0205\"},{\"address\":\"GC64TX2VL5YVNQ7YSTXQAYEQDTU3KYNWQUVPCTL5Z7XULA74LJLC4H27\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0301\"},{\"address\":\"GCPFKCAL2YBIHX66MBF465NAF5KL55A6EJI7C4ZHAVIRAUKTNM3BKSJT\"}]},{\"u32\":10}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0302\"},{\"address\":\"GBPFHCWQEKMD6TS6Z73N2FQHMIIHB3Q6223LOIN3RSHNKUFFFEEL2LQD\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0303\"},{\"address\":\"GCS53T3NE3TWJXZRW6KYNQV5HIQ45SOGVUEE5Q4UIX6Z4SUANTKBTDQ7\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0304\"},{\"address\":\"GAF4P42BZL2RX4P5ZZYW4XCW4EZGL5WD7EJTTOFZ5IH2P2HFMI6SPZ4Z\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SQL0305\"},{\"address\":\"GBA35OV7NKUAV7TD4P7UIK2RNQGY2LKFOO4XTNKSZLJRNBL5JXL5BBXL\"}]},{\"u32\":1}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SSQL01\"},{\"address\":\"GAOILZ7SVHGY7ZXBGRD2JPUFUR4BR2ZQCS4M2J4NKR2FSFSHFZPU44GY\"}]},{\"u32\":15}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SSQL02\"},{\"address\":\"GANLKSR75HSQOTZRMXYKV7O453XJKJ2ZZXCRUV3CYO5OIEOPQBU5HNPN\"}]},{\"u32\":15}],"
BADGE_MAP+="[{\"vec\":[{\"string\":\"SSQL03\"},{\"address\":\"GCT5XUV7IVJ4RFDE3ZYOSFTB6YQTZUZL22JACGNX35LVMHRYER5SCXBV\"}]},{\"u32\":15}]"
BADGE_MAP+="]"

INIT_ARGS="[{\"address\":\"$ADMIN_ADDRESS\"},{\"map\":$BADGE_MAP},{\"string\":\"New Scorer\"},{\"string\":\"This is a new scorer contract\"},{\"string\":\"icon.png\"}]"

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
mkdir -p .deploy
echo "NETWORK=$NETWORK" > .deploy/scorer.env
echo "SCORER_ADDRESS=$SCORER_ADDRESS" >> .deploy/scorer.env
echo "ADMIN_ADDRESS=$ADMIN_ADDRESS" >> .deploy/scorer.env
echo "DEPLOYMENT_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")" >> .deploy/scorer.env

echo -e "${GREEN}Scorer contract creation successful!${NC}"
echo "Scorer Address: $SCORER_ADDRESS"
echo "Deployment information saved to .deploy/scorer.env"