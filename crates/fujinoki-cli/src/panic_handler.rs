use std::{panic::PanicHookInfo, process};

use const_format::formatcp;
use fujinoki_core::{get_version, DISPLAY_NAME, GITHUB_REPO, NPM_PACKAGE};
use human_panic::report::{Method, Report};

const OPEN_ISSUE_MESSAGE: &str =
    formatcp!("Please open an issue at https://github.com/{GITHUB_REPO}/issues/new/choose");

pub fn panic_handler(panic_info: &PanicHookInfo) {
    let cause = panic_info
        .payload_as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| "Unknown".to_string());

    let explanation = match panic_info.location() {
        Some(location) => format!("file '{}' at line {}\n", location.file(), location.line()),
        None => "unknown.".to_string(),
    };

    // Turbopack's failing issues are automatically logged and panicked, so this
    // would be redundant.
    if cause.contains("Fatal issue(s) occurred") {
        process::exit(1);
    }

    dbg!(cause.clone(), explanation.clone());

    // If we're in CI we don't persist the backtrace to a temp file as this is hard
    // to retrieve.
    let should_persist = !turborepo_ci::is_ci() && turborepo_ci::Vendor::infer().is_none();

    let report = Report::new(
        NPM_PACKAGE,
        get_version(),
        Method::Panic,
        explanation,
        cause,
    );

    let report_message = if should_persist {
        match report.persist() {
            Ok(f) => {
                format!(
                    "A report has been written to {}\n\n{OPEN_ISSUE_MESSAGE} and include this file",
                    f.display()
                )
            }
            Err(e) => {
                format!(
                    "An error has occurred while attempting to write a \
                     report.\n\n{OPEN_ISSUE_MESSAGE} and include the following error in your \
                     issue: {}",
                    e
                )
            }
        }
    } else if let Some(backtrace) = report.serialize() {
        format!(
            "Caused by \n{backtrace}\n\n{OPEN_ISSUE_MESSAGE} and include this message in your \
             issue"
        )
    } else {
        format!(
            "Unable to serialize backtrace.\n\n{OPEN_ISSUE_MESSAGE} and include this message in \
             your issue"
        )
    };

    eprintln!(
        "{DISPLAY_NAME} has crashed.

{}",
        report_message
    );
    process::exit(1);
}
