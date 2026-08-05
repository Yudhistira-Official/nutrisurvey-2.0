fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root_env = manifest_dir.join("..").join(".env");
    let _ = dotenvy::from_path(root_env);
    for name in [
        "AI_API_KEY",
        "AI_MODEL",
        "AI_BASE_URL",
        "BASE_URL",
        "AI_PROVIDER",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        if let Ok(value) = std::env::var(name) {
            println!("cargo:rustc-env={name}={value}");
        }
    }
    tauri_build::build()
}
