# Deployer Contract

The Deployer Contract is a fundamental component of the Trustful system, designed to handle the atomic deployment and initialization of other contracts within the ecosystem. This contract implements a secure mechanism to deploy and initialize contracts in a single transaction, preventing potential initialization vulnerabilities.

## Overview

The Deployer Contract serves as a deployment mechanism that:
- Enables atomic deployment and initialization of contracts
- Manages authorization of deployment operations
- Provides a standardized way to deploy contracts within the Trustful ecosystem

## Contract Interface

### Methods

#### `deploy`

```rust
pub fn deploy(
    env: Env,
    deployer: Address,
    wasm_hash: BytesN<32>,
    salt: BytesN<32>,
    init_fn: Symbol,
    init_args: Vec<Val>,
) -> (Address, Val)
```

This is the main method of the Deployer contract that handles the deployment and initialization process.

**Parameters:**
- `env`: The environment object providing access to blockchain context
- `deployer`: The address authorized to perform the deployment
- `wasm_hash`: The hash of the WASM bytecode to be deployed
- `salt`: A 32-byte value used to generate a unique contract address
- `init_fn`: The name of the initialization function to call after deployment
- `init_args`: Arguments to pass to the initialization function

**Returns:**
- A tuple containing:
  - The address of the newly deployed contract
  - The result value from the initialization function call

