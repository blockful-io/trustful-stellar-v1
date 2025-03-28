# Scorer Contract

The Scorer Contract is a key component of the Trustful system responsible for managing user badges, scores, and access control. It implements an upgradeable pattern and manages a registry of users and their reputation badges.

## Overview

The Scorer Contract provides functionality to:
- Manage digital badges with associated scores (0-10000)
- Control user registration and management
- Handle access control through a manager system
- Support contract upgrades
- Store and manage contract metadata (name, description, icon)

## Contract Interface

### Core Methods

#### `initialize`
```rust
pub fn initialize(
    env: Env, 
    scorer_creator: Address, 
    scorer_badges: Map<BadgeId, u32>,
    name: String,
    description: String,
    icon: String
)
```
Initializes the contract with initial manager, badges, and metadata.

**Parameters:**
- `env`: The Soroban environment
- `scorer_creator`: Address to be set as contract creator
- `scorer_badges`: Initial set of badges to be registered
- `name`: Name of the scorer instance
- `description`: Description of the scorer instance
- `icon`: Icon URL or identifier for the scorer

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
pub fn add_user(env: Env, user: Address)
```
Registers a new user in the system. Users can add themselves.

#### `remove_user`
```rust
pub fn remove_user(env: Env, user: Address)
```
Removes a user from the system. Users can remove themselves.

#### `get_users`
```rust
pub fn get_users(env: Env) -> Map<Address, bool>
```
Returns the registry of all users and their status (true = active, false = inactive).

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

#### `add_badge`
```rust
pub fn add_badge(env: Env, sender: Address, name: String, issuer: Address, score: u32)
```
Adds a new badge to the contract.

**Parameters:**
- `env`: The Soroban environment
- `sender`: Address of the manager adding the badge
- `name`: Name of the badge
- `issuer`: Address of the badge issuer
- `score`: Score value (0-10000)

#### `remove_badge`
```rust
pub fn remove_badge(env: Env, sender: Address, name: String, issuer: Address)
```
Removes a badge from the contract.

#### `get_badges`
```rust
pub fn get_badges(env: Env) -> Map<BadgeId, u32>
```
Returns all registered badges in the system.

### Metadata Management

#### `get_metadata`
```rust
pub fn get_metadata(env: Env) -> (String, String, String)
```
Returns the contract metadata (name, description, icon).

## Data Structures

### BadgeId
```rust
pub struct BadgeId {
    pub name: String,
    pub issuer: Address,
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
    Name,           // Contract name
    Description,    // Contract description
    Icon           // Contract icon
}
```

## Events

The contract emits events for all major operations:

- User events: `(TOPIC_USER, "add")`, `(TOPIC_USER, "remove")`
- Manager events: `(TOPIC_MANAGER, "add")`, `(TOPIC_MANAGER, "remove")`
- Badge events: `(TOPIC_BADGE, "add")`, `(TOPIC_BADGE, "remove")`
- Upgrade events: `(TOPIC_UPGRADE, "upgrade")`
- Initialization events: `(TOPIC_INIT, "init")`

## Testing

The contract includes comprehensive tests that verify:
- Contract initialization with metadata
- User management operations
- Manager administration
- Badge management with score validation
- Contract upgrades
- Authorization checks
- Event emission
- Metadata management

For detailed test examples, refer to the test module in the contract source code.