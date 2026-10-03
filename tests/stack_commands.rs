use std::{fs, process::Command};

#[test]
fn stacks_and_flamegraph_commands_use_the_folded_output() {
    let fixture = tempfile::tempdir().unwrap();
    let input = fixture.path().join("input");
    fs::create_dir(&input).unwrap();
    fs::write(input.join("payload"), b"data").unwrap();

    let stacks = Command::new(env!("CARGO_BIN_EXE_dua"))
        .args(["stacks", input.to_str().unwrap()])
        .output()
        .unwrap();
    let legacy = Command::new(env!("CARGO_BIN_EXE_dua"))
        .args(["aggregate", "--stack", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(stacks.status.success());
    assert!(legacy.status.success());
    assert_eq!(stacks.stdout, legacy.stdout);

    let flamegraph = Command::new(env!("CARGO_BIN_EXE_dua"))
        .current_dir(fixture.path())
        .args([
            "flamegraph",
            "--output",
            "usage.svg",
            "--palette",
            "blue",
            "--width",
            "640",
            "--min-width",
            "0",
            "--title",
            "Fixture Disk Usage",
            "--inverted",
            "input",
        ])
        .output()
        .unwrap();
    assert!(flamegraph.status.success());
    assert!(flamegraph.stdout.is_empty());
    let svg = fs::read_to_string(fixture.path().join("usage.svg")).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("width=\"640\""));
    assert!(svg.contains(">Fixture Disk Usage</text>"));
    assert!(svg.contains("bytes"));
    assert!(svg.contains("Path:"));
    assert!(svg.contains("payload"));
}

#[test]
fn failed_flamegraph_export_preserves_the_previous_report() {
    let fixture = tempfile::tempdir().unwrap();
    let input = fixture.path().join("input");
    let empty = fixture.path().join("empty");
    let report = fixture.path().join("usage.svg");
    fs::create_dir(&input).unwrap();
    fs::create_dir(&empty).unwrap();
    fs::write(input.join("payload"), b"data").unwrap();

    let export = |input: &str, title: &str| {
        Command::new(env!("CARGO_BIN_EXE_dua"))
            .current_dir(fixture.path())
            .args([
                "flamegraph",
                "--output",
                "usage.svg",
                "--title",
                title,
                input,
            ])
            .output()
            .unwrap()
    };
    assert!(export("input", "Original report").status.success());
    let original = fs::read(&report).unwrap();

    let failed = export("empty", "Failed report");
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("No stack counts found"));
    assert_eq!(fs::read(&report).unwrap(), original);
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 3);

    // A successful export must still replace an existing report.
    assert!(export("input", "Updated report").status.success());
    let updated = fs::read_to_string(&report).unwrap();
    assert!(updated.contains(">Updated report</text>"));
    assert!(!updated.contains("Original report"));
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 3);
}

#[test]
fn failed_flamegraph_export_does_not_leave_a_partial_report() {
    let fixture = tempfile::tempdir().unwrap();
    fs::create_dir(fixture.path().join("empty")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_dua"))
        .current_dir(fixture.path())
        .args(["flamegraph", "--output", "usage.svg", "empty"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No stack counts found"));
    assert!(!fixture.path().join("usage.svg").exists());
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 1);
}
