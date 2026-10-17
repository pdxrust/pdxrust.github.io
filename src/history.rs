//! When each event file was first committed and last changed, from `git log`.
//!
//! Feeds date an entry by when the event was announced, not when it happens. Upcoming events
//! would otherwise carry future dates, which feed readers treat as implausible.

use std::{collections::HashMap, path::Path, process::Command};

use jiff::Timestamp;

use crate::event::Event;

/// Commit times of one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileDates {
    /// Commit that added the file.
    pub added: Timestamp,
    /// Most recent commit that changed the file.
    pub changed: Timestamp,
}

/// Commit times of event files, keyed by path relative to the repository root.
pub type Dates = HashMap<String, FileDates>;

/// The commit times of `event`'s file, or `fallback` for a file git doesn't know.
pub fn dates_of(event: &Event, dates: &Dates, fallback: Timestamp) -> FileDates {
    let path = format!("events/{}/{}.md", event.slug.year(), event.slug);
    dates.get(&path).copied().unwrap_or(FileDates {
        added: fallback,
        changed: fallback,
    })
}

/// Commit times of every file under `dir`, keyed by its path relative to the repository root.
///
/// Returns an empty map with a warning on stderr when `git` or the repository is missing, so a
/// source tarball still builds. A shallow clone is an error, because every file would appear to
/// have been added by the newest commit.
pub fn file_dates(dir: &Path) -> Result<Dates, String> {
    let shallow = Command::new("git")
        .args(["rev-parse", "--is-shallow-repository"])
        .output();
    match shallow {
        Ok(out) if out.status.success() => {
            if String::from_utf8_lossy(&out.stdout).trim() == "true" {
                return Err(
                    "the git clone is shallow, so feed dates would be wrong. Fetch the \
                            full history (actions/checkout: fetch-depth: 0)"
                        .into(),
                );
            }
        }
        _ => {
            eprintln!("warning: no git history, so feed entries are dated with the build time");
            return Ok(HashMap::new());
        }
    }
    let out = Command::new("git")
        .args(["log", "--format=@%cI", "--name-only", "--no-renames", "--"])
        .arg(dir)
        .output()
        .map_err(|e| format!("git log: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git log: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    parse_log(&String::from_utf8_lossy(&out.stdout))
}

/// Parses `git log --format=@%cI --name-only` output, which lists the newest commit first.
fn parse_log(log: &str) -> Result<Dates, String> {
    let mut dates = Dates::new();
    let mut commit = None;
    for line in log.lines().filter(|l| !l.is_empty()) {
        if let Some(date) = line.strip_prefix('@') {
            let date = date
                .parse()
                .map_err(|e| format!("git log date `{date}`: {e}"))?;
            commit = Some(date);
            continue;
        }
        let Some(date) = commit else {
            return Err(format!("git log: file `{line}` before any commit"));
        };
        // Walking newest to oldest, the first sighting is the last change and the last sighting
        // is the commit that added the file.
        dates
            .entry(line.to_owned())
            .and_modify(|d| d.added = date)
            .or_insert(FileDates {
                added: date,
                changed: date,
            });
    }
    Ok(dates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_and_last_commit_per_file() {
        let log = "@2026-03-01T10:00:00-08:00\n\nevents/2026/b.md\nevents/2026/a.md\n\
                   @2026-02-01T10:00:00-08:00\n\nevents/2026/a.md\n";
        let dates = parse_log(log).unwrap();
        let a = dates["events/2026/a.md"];
        assert_eq!(a.added.to_string(), "2026-02-01T18:00:00Z");
        assert_eq!(a.changed.to_string(), "2026-03-01T18:00:00Z");
        let b = dates["events/2026/b.md"];
        assert_eq!(b.added, b.changed);
    }

    #[test]
    fn rejects_malformed_output() {
        assert!(parse_log("events/2026/a.md\n").is_err());
        assert!(parse_log("@yesterday\n").is_err());
    }
}
