use std::process::Command;
use std::path::Path;

// ── Helper ──────────────────────────────────────────────────────────────

fn cargo_run(args: &[&str]) -> std::process::Output {
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(workspace).parent().unwrap().parent().unwrap();

    Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("pastel")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .current_dir(workspace_root)
        .output()
        .expect("failed to execute cargo run")
}

fn example_path(name: &str) -> String {
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(workspace).parent().unwrap().parent().unwrap();
    workspace_root
        .join("examples")
        .join(name)
        .join("main.pastel")
        .to_string_lossy()
        .into_owned()
}

// ── check command ───────────────────────────────────────────────────────

#[test]
fn check_hello_world_succeeds() {
    let path = example_path("hello-world");
    let output = cargo_run(&["check", &path]);
    assert!(output.status.success(), "check should succeed for hello-world");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("is valid"), "stdout should say 'is valid': {}", stdout);
}

#[test]
fn check_landing_page_succeeds() {
    let path = example_path("landing-page");
    let output = cargo_run(&["check", &path]);
    assert!(output.status.success(), "check should succeed for landing-page");
}

#[test]
fn check_nonexistent_file_fails() {
    let output = cargo_run(&["check", "/tmp/nonexistent_pastel_file.pastel"]);
    assert!(
        !output.status.success(),
        "check should fail for nonexistent file"
    );
}

// ── plan command ────────────────────────────────────────────────────────

#[test]
fn plan_hello_world_output() {
    let path = example_path("hello-world");
    let output = cargo_run(&["plan", &path]);
    assert!(output.status.success(), "plan should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // plan should print the tree structure
    assert!(stdout.contains("Document:"), "plan should show Document: header");
    assert!(stdout.contains("frame"), "plan should mention frame nodes");
}

#[test]
fn plan_landing_page_shows_tree() {
    let path = example_path("landing-page");
    let output = cargo_run(&["plan", &path]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("navbar"), "should show navbar node");
    assert!(stdout.contains("hero-section"), "should show hero-section node");
}

// ── inspect command ─────────────────────────────────────────────────────

#[test]
fn inspect_json_produces_valid_json() {
    let path = example_path("hello-world");
    let output = cargo_run(&["inspect", &path, "--json"]);
    assert!(output.status.success(), "inspect --json should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("inspect --json should produce valid JSON");
    assert_eq!(json["version"], 1);
    assert!(json["canvas"].is_object());
    assert!(json["nodes"].is_array());
}

#[test]
fn inspect_without_json_flag() {
    let path = example_path("hello-world");
    let output = cargo_run(&["inspect", &path]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Document:"), "should show Document header");
    assert!(stdout.contains("Canvas:"), "should show Canvas info");
}

// ── build command ───────────────────────────────────────────────────────

#[test]
fn build_outputs_ir_json() {
    let path = example_path("hello-world");
    let output_path = std::env::temp_dir().join("pastel_test_build_output.json");

    let output = cargo_run(&["build", &path, "-o", &output_path.to_string_lossy()]);
    assert!(output.status.success(), "build should succeed: stderr={}", String::from_utf8_lossy(&output.stderr));

    // Verify the output file is valid JSON
    let content = std::fs::read_to_string(&output_path).expect("should read output file");
    let json: serde_json::Value =
        serde_json::from_str(&content).expect("build output should be valid JSON");
    assert_eq!(json["version"], 1);
    assert!(json["nodes"].is_array());

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}
