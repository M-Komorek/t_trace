use std::env;
use std::fs;
use std::path::Path;

fn main() {
    const BASH_HOOK_FILENAME: &str = "bash_hooks.sh";
    const BASH_HOOK_PATH: &str = "scripts/bash_hooks.sh";

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join(BASH_HOOK_FILENAME);

    let hook_script_content =
        fs::read_to_string(BASH_HOOK_PATH).expect("Failed to read {BASH_HOOK_PATH}");

    fs::write(
        &dest_path,
        format!(
            "const BASH_HOOK_SCRIPT: &str = r#\"{}\"#;",
            hook_script_content
        ),
    )
    .expect("Failed to write generated bash hook file");

    println!("cargo:rustc-env=BASH_HOOK_FILENAME={}", BASH_HOOK_FILENAME);
    println!("cargo:rerun-if-changed=hooks.sh");
}
