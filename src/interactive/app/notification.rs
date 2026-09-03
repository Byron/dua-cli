//! Utilities to produce notifications.
use std::{io, time::Duration};

use crate::interactive::widgets::Language;
use dua::ByteFormat;

pub fn scan_finished(
    language: Language,
    entries: u64,
    bytes: u128,
    elapsed: Duration,
    errors: u64,
    format: ByteFormat,
) -> String {
    language.notification_summary(
        language.ui_text().notification_scan,
        entries,
        &format.display(bytes).to_string(),
        &duration(elapsed),
        errors,
    )
}

pub fn deletion_finished(
    language: Language,
    action: &str,
    entries: usize,
    bytes: u128,
    elapsed: Duration,
    errors: usize,
    format: ByteFormat,
) -> String {
    language.notification_summary(
        action,
        u64::try_from(entries).unwrap_or(u64::MAX),
        &format.display(bytes).to_string(),
        &duration(elapsed),
        u64::try_from(errors).unwrap_or(u64::MAX),
    )
}

pub fn emit_if_unfocused(enabled: bool, terminal_focussed: bool, message: &str) -> io::Result<()> {
    let stderr = io::stderr();
    emit_if_unfocused_to(stderr.lock(), enabled, terminal_focussed, message)
}

fn emit_if_unfocused_to(
    mut write: impl io::Write,
    enabled: bool,
    terminal_focussed: bool,
    message: &str,
) -> io::Result<()> {
    if !enabled || terminal_focussed {
        return Ok(());
    }
    let message = message
        .chars()
        .map(|character| match character {
            ';' => ',',
            character if character.is_control() => ' ',
            character => character,
        })
        .collect::<String>();
    write!(write, "\x1b]777;notify;DUA;{message}\x1b\\")?;
    write.flush()
}

fn duration(duration: Duration) -> String {
    let seconds = duration.as_secs_f64();
    if seconds < 10.0 {
        format!("{seconds:.1}s")
    } else {
        format!("{seconds:.0}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_sanitized_osc_777_notification() {
        let mut output = Vec::new();
        emit_if_unfocused_to(&mut output, true, false, "Done; unsafe\nmessage")
            .expect("notification written");
        assert_eq!(output, b"\x1b]777;notify;DUA;Done, unsafe message\x1b\\");
    }

    #[test]
    fn emits_only_when_enabled_and_unfocused() {
        let mut output = Vec::new();
        emit_if_unfocused_to(&mut output, true, true, "focused").expect("no-op succeeds");
        emit_if_unfocused_to(&mut output, false, false, "disabled").expect("no-op succeeds");
        assert!(output.is_empty());

        emit_if_unfocused_to(&mut output, true, false, "ready").expect("notification written");
        assert_eq!(
            output, b"\x1b]777;notify;DUA;ready\x1b\\",
            "enabled notifications in unfocussed terminals are emitted"
        );
    }

    #[test]
    fn formats_scan_statistics_concisely() {
        assert_eq!(
            scan_finished(
                Language::English,
                42,
                2_000_000,
                Duration::from_millis(1250),
                2,
                ByteFormat::Metric,
            ),
            "Scan finished: 42 entries, 2.00 MB in 1.2s, 2 errors"
        );

        assert_eq!(
            scan_finished(
                Language::English,
                42,
                2_000_000,
                Duration::from_millis(11250),
                2,
                ByteFormat::Metric,
            ),
            "Scan finished: 42 entries, 2.00 MB in 11s, 2 errors",
            "time rounding happens beyond the 10s mark"
        );
    }
}
