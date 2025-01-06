# Trustful

A Verifiable Reputation Aggregation System on Stellar Soroban

Trustful is an innovative reputation aggregation system built on the Stellar Soroban platform that enables communities and projects to issue digital badges recognizing user contributions. These badges serve as verifiable proof of achievements and skills within the Stellar ecosystem.

## Project Structure

The project is organized as follows:

```
Trustful/
├── contracts/       # Smart contracts for the system
├── src/             # Main file
├── scripts/         # Utility scripts for deployment and testing
├── tests/           # Test cases for the contracts
└── README.md        # Project documentation
```

## Architecture Overview

The Trustful system is built using a modular architecture consisting of multiple smart contracts to facilitate the creation, management, and verification of digital badges. Below is a brief summary of the key contracts:

### Contract Summaries

- **Deployer Contract**: Handles the atomic deployment and initialization of other contracts in the system.
- **Scorer Contract**: Manages badges, scores and users.
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

