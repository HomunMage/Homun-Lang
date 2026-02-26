use std::process::Command;

fn main() {
    // Prefer HOMUN_VERSION env (set by Docker/CI), fall back to git tag, then "dev".
    let version = std::env::var("HOMUN_VERSION")
        .ok()
        .filter(|s| !s.is_empty() && s != "dev")
        .or_else(|| {
            Command::new("git")
                .args(["describe", "--tags", "--always"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "dev".to_string());

    println!("cargo:rustc-env=HOMUN_VERSION={}", version);
}
