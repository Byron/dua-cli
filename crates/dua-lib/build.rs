use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rustc-check-cfg=cfg(dua_macos_bulk)");
    println!("cargo:rustc-check-cfg=cfg(dua_macos_clock_gettime)");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    let version = env::var("MACOSX_DEPLOYMENT_TARGET")
        .ok()
        .or_else(|| {
            // Ask the target compiler, since the host OS may differ when cross-compiling.
            let output = Command::new(env::var_os("RUSTC")?)
                .args(["--print=deployment-target", "--target"])
                .arg(env::var_os("TARGET")?)
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            String::from_utf8(output.stdout)
                .ok()?
                .trim()
                .strip_prefix("MACOSX_DEPLOYMENT_TARGET=")
                .map(str::to_owned)
        })
        .unwrap_or_default();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let (bulk, clock_gettime) = macos_api_support(&arch, &version);
    if bulk {
        println!("cargo:rustc-cfg=dua_macos_bulk");
    }
    if clock_gettime {
        println!("cargo:rustc-cfg=dua_macos_clock_gettime");
    }
}

fn macos_api_support(arch: &str, version: &str) -> (bool, bool) {
    // PowerPC never shipped with macOS 10.10; unknown compilers/targets use the fallback.
    if !matches!(arch, "x86" | "x86_64" | "aarch64") {
        return (false, false);
    }
    let mut parts = version.split('.');
    let major = parts.next().and_then(|part| part.parse::<u32>().ok());
    let minor = parts.next().unwrap_or("0").parse::<u32>().ok();
    let (Some(major), Some(minor)) = (major, minor) else {
        return (false, false);
    };
    if parts.any(|part| part.parse::<u32>().is_err()) {
        return (false, false);
    }
    ((major, minor) >= (10, 10), (major, minor) >= (10, 12))
}
