# Trustful

A Verifiable Reputation Aggregation System on Stellar Soroban

Trustful is an innovative reputation aggregation system built on the Stellar Soroban platform that enables communities and projects to issue digital badges recognizing user contributions. These badges serve as verifiable proof of achievements and skills within the Stellar ecosystem.

## Project Structure

The project is organized as follows:

```
Trustful/
├── contracts/           # Smart contracts for the system
│   ├── deployer/       # Contract deployment and initialization
│   ├── scorer/         # Badge and user management
│   └── scorer_factory/ # Factory for creating scorer instances
├── src/                # Main file
├── scripts/            # Utility scripts for deployment and testing
├── tests/              # Test cases for the contracts
└── README.md           # Project documentation
```

## Architecture Overview

The Trustful system is built using a modular architecture consisting of multiple smart contracts to facilitate the creation, management, and verification of digital badges. Below is a brief summary of the key contracts:

### Contract Summaries

- **Deployer Contract**: Handles the atomic deployment and initialization of other contracts in the system.
- **Scorer Contract**: Manages badges, scores, users, and metadata (name, description, icon).
- **Scorer Factory Contract**: Implements a factory pattern to deploy multiple Scorer Contracts efficiently.

For detailed information on each contract's methods and functionalities, please refer to the README files in the `contracts/` folder.

## Design Patterns

Trustful implements specific design patterns to ensure security, modularity, and maintainability of contracts. Below are the main patterns used in our implementation:

### Contract Patterns

#### Upgradeable Pattern
Enables contract code updates without losing state or address, crucial for system maintenance and evolution over time.

#### Factory Pattern
Used to create and manage multiple instances of Scorer contracts in a standardized way, enabling system scalability.

#### Access Control Pattern
Implements a role-based permission system to protect critical contract functions.

#### Storage Pattern
Defines an organized structure for contract state storage, facilitating data access and modification.

#### Testing Pattern
Establishes a comprehensive testing framework including unit, fuzzing, and integration tests.

### Security Patterns

#### Check-Effects-Interactions Pattern
Organizes operations in a secure sequence: validations, state modifications, and external interactions.

#### Atomic Deployment
Ensures system initialization occurs in a single atomic transaction, preventing invalid intermediate states and Front-Running.

## Key Features

- **Badge Management**:  Manage digital badges with associated scores (0-10000)
- **User Management**: Register and manage users with active/inactive status
- **Manager System**: Role-based access control for contract administration
- **Metadata Support**: Each scorer instance includes name, description, and icon
- **Event System**: Comprehensive event emission for all major operations
- **Factory Pattern**: Efficient deployment of multiple scorer instances
- **Upgradeable Contracts**: Support for contract upgrades while preserving state

## Development

### Prerequisites

Before you begin, ensure you have the following tools installed:
- Rust toolchain
- Soroban CLI

### Build

To build the project, run the following command:

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Test

To run the tests, use the following command:

```bash
cargo test --workspace
```

## Deployment Setup

### Prerequisites

1. Install the Stellar CLI. You can do this via Homebrew:
```bash
brew install stellar-cli
```

Or using Cargo:
```bash
cargo install --locked stellar-cli@22.1.0 --features opt
```

### Initial Setup

1. Generate a test account (this example uses 'alice' as the account name):
```bash
stellar keys generate --global alice --network testnet --fund
```

You can verify the address was created with:
```bash
stellar keys address alice
```

2. Make the scripts executable:
```bash
chmod +x scripts/deploy_deployer.sh
chmod +x scripts/deploy_factory.sh
chmod +x scripts/create_scorer.sh
chmod +x scripts/add_user_manager.sh
```

### Deployment Steps

1. Deploy the deployer contract:
```bash
./scripts/deploy_deployer.sh -n testnet -s alice
```

2. Deploy the factory contract:
```bash
./scripts/deploy_factory.sh -n testnet -s alice
```

3. Create a scorer instance:
```bash
./scripts/create_scorer.sh -s alice -n testnet
```

4. Add/remove users and managers (requires two accounts):
```bash
./scripts/add_user_manager.sh -s alice -t bob -n testnet
```

Note: Replace `alice` and `bob` with your actual account names. The source account (`-s`) should be the admin account that deployed the contracts, while the target account (`-t`) is the account you want to add as a user/manager.

## Security Considerations

- All contract functions implement proper authorization checks
- Badge scores are limited to a maximum of 10000
- Contract upgrades require proper authorization
- User and manager operations are protected by role-based access control
- All state changes emit events for auditability

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

