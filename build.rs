use std::{env, fs, path::Path};

fn textures(root: &Path, out: &Path) {
    let dir = root.join("assets/tex");
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
    fs::write(out.join("textures.rs"), format!("pub static TEXTURES: &[(&str, &[u8])] = &[\n{table}];\n")).expect("write textures.rs");
}

fn maps(root: &Path, out: &Path) {
    let dir = root.join("assets/map");
    let list = dir.join("campaign.txt");
    println!("cargo:rerun-if-changed={}", list.display());
    let table: String = fs::read_to_string(&list)
        .expect("assets/map/campaign.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with(';'))
        .map(|n| format!("    ({n:?}, include_str!({:?})),\n", dir.join(format!("{n}.map")).display().to_string()))
        .collect();
    fs::write(out.join("maps.rs"), format!("static MAPS: &[(&str, &str)] = &[\n{table}];\n")).expect("write maps.rs");
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = env::var("OUT_DIR").expect("OUT_DIR");
    textures(root, Path::new(&out));
    maps(root, Path::new(&out));
}
