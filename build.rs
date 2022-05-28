use std::path::Path;

use npm_rs::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=app");
    let status = NpmEnv::default()
        .with_node_env(&NodeEnv::from_cargo_profile().unwrap_or_default())
        .set_path(Path::new("app/"))
        .init_env()
        .install(None)
        .run("build")
        .exec()?;

    assert!(status.success());
    Ok(())
}