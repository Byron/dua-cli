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
