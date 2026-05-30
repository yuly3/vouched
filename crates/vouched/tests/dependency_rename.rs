use std::{env, path::PathBuf, process::Command};

#[test]
fn facade_dependency_can_be_renamed() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/fixtures/dependency-rename");
    let manifest_path = fixture_dir.join("Cargo.toml");
    let target_dir = fixture_dir.join("target");
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

    let output = Command::new(cargo)
        .args(["test", "--manifest-path"])
        .arg(&manifest_path)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()?;

    let status = output.status;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        status.success(),
        "dependency rename fixture failed\nstatus: {status}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    Ok(())
}
