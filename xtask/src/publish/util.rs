use crate::command::Command;

pub fn default_empty_string() -> String {
    String::new()
}

pub fn get_nightly_version() -> String {
    let now = chrono::Local::now();
    let nightly_tag = Command::program("git")
        .args(["tag", "-l", "nightly-*"])
        .error_message("Failed to list nightly tags")
        .output_string();

    let latest_nightly_tag = nightly_tag
        .lines()
        .filter(|tag| tag.starts_with("nightly-"))
        .max()
        .unwrap_or("");

    let patch = if latest_nightly_tag.is_empty() {
        None
    } else {
        latest_nightly_tag
            .split('.')
            .last()
            .and_then(|s| s.parse::<u64>().ok())
    };

    format!(
        "0.0.0-{}.{}",
        now.format("%y%m%d"),
        patch.map(|p| p + 1).unwrap_or(0)
    )
}
