# Scorer Contract

The Scorer Contract is a key component of the Trustful system responsible for managing user badges, scores, and access control. It implements an upgradeable pattern and manages a registry of users and their reputation badges.

## Overview

The Scorer Contract provides functionality to:
- Manage digital badges with associated scores
- Control user registration and management
- Handle access control through a manager system
- Support contract upgrades

## Contract Interface

### Core Methods

#### `initialize`
```rust
pub fn initialize(env: Env, scorer_creator: Address, scorer_badges: Map<u32, ScorerBadge>)
```
Initializes the contract with initial manager and badges.

**Parameters:**
- `env`: The Soroban environment
- `scorer_creator`: Address to be set as contract creator
- `scorer_badges`: Initial set of badges to be registered

#### `upgrade`
```rust
pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>)
```
Upgrades the contract code to a new version.

**Parameters:**
- `env`: The Soroban environment
- `new_wasm_hash`: Hash of the new contract WASM

### User Management

#### `add_user`
```rust
pub fn add_user(env: Env, sender: Address, user: Address)
```
Registers a new user in the system.

#### `remove_user`
```rust
pub fn remove_user(env: Env, sender: Address, user: Address)
```
Removes a user from the system.

#### `get_users`
```rust
pub fn get_users(env: Env) -> Map<Address, bool>
```
Returns the registry of all users and their status.

### Manager Administration

#### `add_manager`
```rust
pub fn add_manager(env: Env, sender: Address, new_manager: Address)
```
Adds a new manager to the contract.

#### `remove_manager`
```rust
pub fn remove_manager(env: Env, sender: Address, manager_to_remove: Address)
```
Removes a manager from the contract.

### Badge Management

#### `get_badges`
```rust
pub fn get_badges(env: Env) -> Map<u32, ScorerBadge>
```
Returns all registered badges in the system.

## Data Structures

### ScorerBadge
```rust
pub struct ScorerBadge {
    pub name: String,
    pub issuer: Address,
    pub score: u32,
}
```

### Storage Keys
```rust
enum DataKey {
    ScorerCreator,   // Contract creator address
    ScorerBadges,    // Map of badges
    Users,           // Map of registered users
    Managers,        // List of managers
    Initialized,     // Initialization status
}
```

## Testing

The contract includes comprehensive tests that verify:
- Contract initialization
- User management operations
- Manager administration
- Badge management
- Contract upgrades
- Authorization checks

For detailed test examples, refer to the test module in the contract source code.