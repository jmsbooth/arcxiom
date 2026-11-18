//! Process-level end-to-end tests of the installed Rust CLI and exit contract.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arcxiom"))
        .args(args)
        .output()
        .expect("built arcxiom binary should execute")
}
fn published() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/emergency-response.arch.yaml")
        .display()
        .to_string()
}
fn test_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .canonicalize()
        .expect("resolve system temp root")
        .join(format!(
            "arcxiom-cli-e2e-{}-{test_name}",
            std::process::id()
        ));
    fs::create_dir_all(&dir).unwrap();
    dir
}
#[test]
fn source_to_export_to_verify_to_query_pipeline() {
    let example = published();
    let result = command(&["validate", &example, "--json"]);
    assert_eq!(result.status.code(), Some(0), "{result:?}");
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["valid"], true);
    let dir = test_dir("source-export-query");
    let output = dir.join("model.json");
    let filename = output.to_str().unwrap();
    let export = command(&["export", &example, "--output", filename]);
    assert_eq!(export.status.code(), Some(0), "{export:?}");
    let data: serde_json::Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    assert_eq!(data["sha256"], report["digest"]);
    let verify = command(&["verify", filename]);
    assert_eq!(verify.status.code(), Some(0), "{verify:?}");
    assert!(String::from_utf8_lossy(&verify.stdout).contains(data["sha256"].as_str().unwrap()));
    let q = command(&[
        "query",
        &example,
        "--id",
        "role.dispatcher",
        "--depth",
        "8",
        "--json",
    ]);
    assert_eq!(q.status.code(), Some(0), "{q:?}");
    let graph: serde_json::Value = serde_json::from_slice(&q.stdout).unwrap();
    assert!(graph["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["id"] == "mission.coordinate-response"));
    let _ = fs::remove_file(output);
    let _ = fs::remove_dir(dir);
}
#[test]
fn diff_exit_codes_and_baseline_guards() {
    let example = published();
    let original = fs::read_to_string(&example).unwrap();
    let dir = test_dir("diff-guardrails");
    let candidate = dir.join("candidate.arch.yaml");
    let unchanged = command(&["diff", &example, &example, "--json"]);
    assert_eq!(unchanged.status.code(), Some(0));
    fs::write(
        &candidate,
        original.replace(
            "Coordinate emergency response",
            "Coordinate emergency response revision",
        ),
    )
    .unwrap();
    let path = candidate.to_str().unwrap();
    let changed = command(&["diff", &example, path, "--json"]);
    assert_eq!(changed.status.code(), Some(3), "{changed:?}");
    let delta: serde_json::Value = serde_json::from_slice(&changed.stdout).unwrap();
    assert_eq!(delta["summary"]["entities"]["modified"], 1);
    fs::write(
        &candidate,
        original.replace("enforcement: hard", "enforcement: advisory"),
    )
    .unwrap();
    let blocked = command(&["diff", &example, path, "--check-guardrails", "--json"]);
    assert_eq!(blocked.status.code(), Some(2), "{blocked:?}");
    let guard: serde_json::Value = serde_json::from_slice(&blocked.stdout).unwrap();
    assert_eq!(guard["guardrail_violations"][0]["code"], "AXG002");
    let _ = fs::remove_file(candidate);
    let _ = fs::remove_dir(dir);
}
#[test]
fn invalid_source_and_argument_exit_contract() {
    let dir = test_dir("invalid-source");
    let bad = dir.join("invalid.arch.yaml");
    fs::write(
        &bad,
        "version: 0.1-draft\nversion: 0.1-draft\nnamespace: test\n",
    )
    .unwrap();
    let invalid = command(&["validate", bad.to_str().unwrap(), "--json"]);
    assert_eq!(invalid.status.code(), Some(2), "{invalid:?}");
    let report: serde_json::Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(report["diagnostics"][0]["code"], "AXY002");
    assert_eq!(command(&["new-unknown-command"]).status.code(), Some(64));
    assert_eq!(
        command(&["query", &published(), "--id", "not-a-node"])
            .status
            .code(),
        Some(2)
    );
    let _ = fs::remove_file(bad);
    let _ = fs::remove_dir(dir);
}

#[test]
fn bare_output_filename_in_current_directory_is_safe_and_supported() {
    let dir = test_dir("cwd-output");
    let example = published();
    let result = Command::new(env!("CARGO_BIN_EXE_arcxiom"))
        .current_dir(&dir)
        .args(["export", &example, "-o", "model.json"])
        .output()
        .expect("Rust CLI must execute");
    assert_eq!(result.status.code(), Some(0), "{result:?}");
    let output = dir.join("model.json");
    assert!(output.is_file());
    assert_eq!(
        command(&["verify", output.to_str().unwrap()]).status.code(),
        Some(0)
    );
    let _ = fs::remove_file(output);
    let _ = fs::remove_dir(dir);
}

#[test]
fn verifier_caps_oversized_ir_files_before_deserialization() {
    let dir = test_dir("oversized-json");
    let path = dir.join("oversized.json");
    let file = fs::File::create(&path).unwrap();
    file.set_len((4 * arcxiom_ir::MAX_BYTES + 2) as u64)
        .unwrap();
    let result = command(&["verify", path.to_str().unwrap()]);
    assert_eq!(result.status.code(), Some(2), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("AXIR07"));
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir(dir);
}

#[test]
fn verify_rejects_directory_without_blocking() {
    let dir = test_dir("verify-directory");
    let status = command(&["verify", dir.to_str().unwrap()]);
    assert_eq!(status.status.code(), Some(2), "{status:?}");
    assert!(String::from_utf8_lossy(&status.stderr).contains("AXIR20"));
    let _ = fs::remove_dir(dir);
}

#[test]
fn unsupported_subcommand_options_fail_without_side_effects() {
    let example = published();
    let dir = test_dir("unsupported-options");
    let output = dir.join("should-not-exist.json");
    let target = output.to_str().unwrap();
    let cases: &[&[&str]] = &[
        &["validate", &example, "--output", target],
        &["validate", &example, "--check-guardrails"],
        &["export", &example, "--id", "role.dispatcher"],
        &["export", &example, "--json"],
        &[
            "query",
            &example,
            "--id",
            "role.dispatcher",
            "--check-guardrails",
        ],
        &[
            "query",
            &example,
            "--id",
            "role.dispatcher",
            "--output",
            target,
        ],
        &["diff", &example, &example, "--depth", "2"],
        &["diff", &example, &example, "--output", target],
        &["verify", &example, "--json"],
        &["verify", &example, "--format", "json"],
    ];
    for args in cases {
        let result = command(args);
        assert_eq!(
            result.status.code(),
            Some(64),
            "accepted unsupported option: {args:?}; {result:?}"
        );
        assert!(result.stdout.is_empty(), "unexpected output for {args:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("AXCLI001"));
    }
    assert!(!output.exists(), "invalid command created an output file");
    let _ = fs::remove_dir(dir);
}
