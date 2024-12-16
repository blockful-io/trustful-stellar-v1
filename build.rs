use std::fs;
use std::path::Path;

fn copy_wasm(contract_name: &str) {
    let source = format!(
        "target/wasm32-unknown-unknown/release/{}.wasm",
        contract_name
    );
    let dest = format!("wasm/{}.wasm", contract_name);
    
    if Path::new(&source).exists() {
        fs::copy(&source, &dest).unwrap();
    }
}

fn main() {
    println!("cargo:rerun-if-changed=contracts/");
    
    // Cria a pasta wasm se não existir
    fs::create_dir_all("wasm").unwrap();
    
    // Lista de contratos para build
    let contracts = vec![
        "scorer",
        "deployer",
        "scorer_factory"
    ];
    
    // Copia o WASM de cada contrato
    for contract in contracts {
        copy_wasm(contract);
    }
}
