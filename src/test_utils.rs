pub mod old_contract {
    soroban_sdk::contractimport!(
        file = "wasm/trustful_stellar_v1_test_upgradable.wasm"
    );
}

pub mod new_contract {
    soroban_sdk::contractimport!(
        file = "wasm/trustful_stellar_v1.wasm"
    );
} 