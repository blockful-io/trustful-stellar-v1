# Scorer Factory Contract

The Scorer Factory Contract is a core component of the Trustful system that manages the creation and administration of Scorer contracts. It implements a factory pattern to efficiently deploy and track multiple Scorer instances while maintaining proper access control.

## Overview

The Scorer Factory Contract provides functionality to:
- Deploy new Scorer contracts with proper initialization
- Manage authorized deployers through a manager system
- Track all created Scorer contracts
- Control access to factory operations

## Contract Interface

### Core Methods

#### `initialize`
```rust
pub fn initialize(env: Env, scorer_creator: Address, scorer_wasm_hash: BytesN<32>)
```
Initializes the factory with initial manager and Scorer contract WASM hash.

**Parameters:**
- `env`: The Soroban environment
- `scorer_creator`: Address to be set as factory creator and initial manager
- `scorer_wasm_hash`: Hash of the Scorer contract WASM binary

#### `create_scorer`
```rust
pub fn create_scorer(
    env: Env,
    deployer: Address,
    salt: BytesN<32>,
    init_fn: Symbol,
    init_args: Vec<Val>,
) -> Address
```
Deploys a new Scorer contract instance.

**Parameters:**
- `env`: The Soroban environment
- `deployer`: Address authorized to deploy the contract
- `salt`: Unique value for contract address generation
- `init_fn`: Initialization function name
- `init_args`: Arguments for initialization
**Returns:**
- Address of the newly deployed Scorer contract

### Administrative Methods

#### `add_manager`
```rust
pub fn add_manager(env: Env, caller: Address, manager: Address)
```
Adds a new manager to the factory.

**Parameters:**
- `env`: The Soroban environment
- `caller`: Address requesting the manager addition
- `manager`: Address to be added as manager

#### `remove_manager`
```rust
pub fn remove_manager(env: Env, caller: Address, manager: Address)
```
Removes a manager from the factory.

**Parameters:**
- `env`: The Soroban environment
- `caller`: Address requesting the manager removal
- `manager`: Address to be removed as manager

### Query Methods

#### `get_scorers`
```rust
pub fn get_scorers(env: Env) -> Map<Address, bool>
```
Returns all Scorer contracts created by the factory.

#### `is_initialized`
```rust
pub fn is_initialized(env: Env) -> bool
```
Checks if the factory has been initialized.

#### `is_scorer_factory_creator`
```rust
pub fn is_scorer_factory_creator(env: Env, address: Address) -> bool
```
Verifies if an address is the factory creator.

#### `is_manager`
```rust
pub fn is_manager(env: Env, address: Address) -> bool
```
Checks if an address is a registered manager.

## Data Storage

The contract stores data using the following keys:

```rust
enum DataKey {
    CreatedScorers,      // Map of created Scorer contracts
    Initialized,         // Initialization status
    ScorerFactoryCreator, // Factory creator address
    Managers,            // Map of authorized managers
    ScorerWasmHash,      // Hash of Scorer contract WASM
}
```

