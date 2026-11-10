use embed_manifest::{embed_manifest, new_manifest};

// The manifest turns on themed controls and per-monitor DPI scaling. It is
// built in Rust because windres fails on a project path that contains spaces.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest(new_manifest("CL650.Installer")).expect("the application manifest must be embedded");
    }
}
