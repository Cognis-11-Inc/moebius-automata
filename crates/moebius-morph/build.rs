use std::{collections::BTreeMap, env, fs, path::Path, process::Command};
fn collect(dir: &Path, root: &Path, map: &mut BTreeMap<String, String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            collect(&p, root, map);
        } else if p.extension().is_some_and(|s| s == "rs") {
            println!("cargo:rerun-if-changed={}", p.display());
            map.insert(
                p.strip_prefix(root).unwrap().to_string_lossy().into_owned(),
                blake3::hash(&fs::read(p).unwrap()).to_hex().to_string(),
            );
        }
    }
}
fn main() {
    let root = env::current_dir().unwrap();
    let mut sources = BTreeMap::new();
    collect(&root.join("src"), &root, &mut sources);
    collect(&root.join("examples"), &root, &mut sources);
    collect(&root.join("../moebius-morph-lab/src"), &root, &mut sources);
    for name in [
        "Cargo.toml",
        "build.rs",
        "../../Cargo.lock",
        "../../Cargo.toml",
    ] {
        println!("cargo:rerun-if-changed={name}");
        sources.insert(
            name.into(),
            blake3::hash(&fs::read(root.join(name)).unwrap())
                .to_hex()
                .to_string(),
        );
    }
    let mut settings: BTreeMap<String, String> = BTreeMap::new();
    for name in [
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
        "TARGET",
        "HOST",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        settings.insert(name.into(), env::var(name).unwrap_or_default());
    }
    let rustc = Command::new(env::var("RUSTC").unwrap())
        .arg("-vV")
        .output()
        .unwrap();
    assert!(rustc.status.success());
    settings.insert("rustc".into(), String::from_utf8(rustc.stdout).unwrap());
    let features: BTreeMap<_, _> = env::vars()
        .filter(|(k, _)| k.starts_with("CARGO_FEATURE_"))
        .collect();
    let identity = serde_json::json!({"schema":1,"sources_blake3":sources,"settings":settings,"features":features});
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("benchmark-identity.json"),
        serde_json::to_vec(&identity).unwrap(),
    )
    .unwrap();
}
