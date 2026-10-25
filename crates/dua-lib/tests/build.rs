include!("../build.rs");

#[test]
fn macos_apis_follow_the_target_version_and_architecture() {
    for arch in ["x86", "x86_64", "aarch64"] {
        for (version, expected) in [
            ("10.9", (false, false)),
            ("10.9.5", (false, false)),
            ("10.10", (true, false)),
            ("10.11.6", (true, false)),
            ("10.12", (true, true)),
            ("11", (true, true)),
            ("26.0.1", (true, true)),
            ("", (false, false)),
            ("invalid", (false, false)),
            ("10.bad", (false, false)),
            ("10.12.bad", (false, false)),
        ] {
            assert_eq!(
                macos_api_support(arch, version),
                expected,
                "{arch}: {version}"
            );
        }
    }
    for arch in ["powerpc", "powerpc64", "unknown"] {
        for version in ["", "10.5.8", "10.12", "26"] {
            assert_eq!(macos_api_support(arch, version), (false, false));
        }
    }
}
