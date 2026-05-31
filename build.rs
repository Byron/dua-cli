use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("heuristics_includes.rs");

    let heuristics_dir = Path::new("etc/heuristics");
    let mut includes = String::new();
    includes.push_str("[\n");

    if heuristics_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(heuristics_dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        
        entries.sort();

        for path in entries {
            let path_str = path.to_str().unwrap().replace('\\', "/");
            includes.push_str(&format!("    include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\")),\n", path_str));
        }
    }

    includes.push_str("]\n");
    fs::write(&dest_path, includes).unwrap();

    println!("cargo:rerun-if-changed=etc/heuristics");
}
