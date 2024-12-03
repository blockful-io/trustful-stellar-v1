use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    
    // Cria a pasta wasm se não existir
    fs::create_dir_all("wasm").unwrap();
    
    // O build.rs roda antes da compilação, então precisamos garantir que 
    // só vamos copiar o arquivo depois que ele existir
    let source = "target/wasm32-unknown-unknown/release/trustful_stellar_v1.wasm";
    let dest = "wasm/trustful_stellar_v1.wasm";
    
    if Path::new(source).exists() {
        fs::copy(source, dest).unwrap();
    }
}
