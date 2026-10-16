use std::{env, fs, path::Path};

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/tex");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut names: Vec<String> = fs::read_dir(&dir)
        .expect("assets/tex")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "tex"))
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    let table: String = names
        .iter()
        .map(|n| format!("    ({n:?}, include_bytes!({:?})),\n", dir.join(format!("{n}.tex")).display().to_string()))
        .collect();
    let out = Path::new(&env::var("OUT_DIR").expect("OUT_DIR")).join("textures.rs");
    fs::write(out, format!("pub static TEXTURES: &[(&str, &[u8])] = &[\n{table}];\n")).expect("write textures.rs");
}
