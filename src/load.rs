//! Reads and validates `site.toml` and the `events/` tree.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use jiff::Timestamp;
use serde::Deserialize;

use crate::{
    event::{Event, FrontMatter, HttpUrl, Slug},
    markdown,
};

/// Site-wide settings from `site.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    /// Site name, shown in page titles.
    pub title: String,
    /// Home page heading.
    pub headline: String,
    /// One-line description for the home page and `<meta name="description">`.
    pub description: String,
    /// External email signup form.
    pub signup_url: HttpUrl,
    /// Discord invite.
    pub discord_url: HttpUrl,
    /// Source repository of this site.
    pub repo_url: HttpUrl,
}

/// A problem with one input file.
#[derive(Debug)]
pub struct LoadError {
    /// File or directory the problem is in.
    pub path: PathBuf,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.message)
    }
}

impl LoadError {
    fn new(path: &Path, message: impl Into<String>) -> Self {
        Self {
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

/// Reads `site.toml`.
pub fn site(path: &Path) -> Result<Site, LoadError> {
    let text = fs::read_to_string(path).map_err(|e| LoadError::new(path, e.to_string()))?;
    toml::from_str(&text).map_err(|e| LoadError::new(path, e.to_string()))
}

/// Reads every event under `dir`, sorted by start time.
///
/// `dir` contains only year directories, and those contain only `.md` event files. Dot files are
/// skipped. Every problem found is returned, not only the first.
pub fn events(dir: &Path) -> Result<Vec<Event>, Vec<LoadError>> {
    let mut events = Vec::new();
    let mut errors = Vec::new();
    let years = match sorted_entries(dir) {
        Ok(years) => years,
        Err(e) => return Err(vec![e]),
    };
    for year_dir in years {
        let year = year_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !year_dir.is_dir() || year.len() != 4 || !year.bytes().all(|b| b.is_ascii_digit()) {
            errors.push(LoadError::new(
                &year_dir,
                "expected a year directory such as `2026/`",
            ));
            continue;
        }
        let files = match sorted_entries(&year_dir) {
            Ok(files) => files,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        for path in files {
            let result = fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|text| parse(&path, year, &text));
            match result {
                Ok(event) => events.push(event),
                Err(message) => errors.push(LoadError::new(&path, message)),
            }
        }
    }
    // Slugs are unique without a separate check. The year directory must match the slug's year,
    // so two files with the same stem would have to share a directory.
    if errors.is_empty() {
        events.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.slug.cmp(&b.slug)));
        Ok(events)
    } else {
        Err(errors)
    }
}

/// Lists `dir`, skipping dot files, in name order.
fn sorted_entries(dir: &Path) -> Result<Vec<PathBuf>, LoadError> {
    let entries = fs::read_dir(dir).map_err(|e| LoadError::new(dir, e.to_string()))?;
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|e| LoadError::new(dir, e.to_string()))?
            .path();
        if !path
            .file_name()
            .is_some_and(|n| n.as_encoded_bytes().starts_with(b"."))
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

/// Parses one event file. `year` is the name of the directory it is in.
pub fn parse(path: &Path, year: &str, text: &str) -> Result<Event, String> {
    if path.extension().is_none_or(|ext| ext != "md") {
        return Err("event files must end in `.md`".into());
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("file name is not UTF-8")?;
    let slug = Slug::parse(stem)?;
    if slug.year() != year {
        return Err(format!(
            "belongs in `events/{}/`, not `events/{year}/`",
            slug.year()
        ));
    }

    let (front, body) = split_front_matter(text)?;
    let front: FrontMatter = toml::from_str(front).map_err(|e| format!("front matter: {e}"))?;

    let title = front.title.trim();
    if title.is_empty() {
        return Err("`title` must not be empty".into());
    }
    let start = timestamp("start", &front.start)?;
    let end = front
        .end
        .as_ref()
        .map(|end| timestamp("end", end))
        .transpose()?;
    if end.is_some_and(|end| end <= start) {
        return Err("`end` must be after `start`".into());
    }
    if let Some(speaker) = front.speakers.iter().find(|s| s.name.trim().is_empty()) {
        return Err(format!(
            "speaker name must not be empty (url: {:?})",
            speaker.url
        ));
    }

    Ok(Event {
        slug,
        title: title.to_owned(),
        start,
        end,
        kind: front.kind,
        status: front.status,
        location: front.location,
        speakers: front.speakers,
        links: front.links,
        body_html: markdown::to_html(body),
        summary_html: markdown::first_paragraph_html(body),
        summary_text: markdown::first_paragraph_text(body),
    })
}

/// Splits `+++`-delimited TOML front matter from the Markdown body.
fn split_front_matter(text: &str) -> Result<(&str, &str), String> {
    const MISSING: &str = "file must start with a `+++` line, then TOML front matter, then `+++`";
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines = text.split_inclusive('\n');
    if lines.next().map(str::trim_end) != Some("+++") {
        return Err(MISSING.into());
    }
    let front_start = text.find('\n').map_or(text.len(), |i| i + 1);
    let mut offset = front_start;
    for line in lines {
        if line.trim_end() == "+++" {
            return Ok((&text[front_start..offset], &text[offset + line.len()..]));
        }
        offset += line.len();
    }
    Err(MISSING.into())
}

/// Converts a TOML date-time to a timestamp, requiring a UTC offset.
fn timestamp(field: &str, value: &toml::value::Datetime) -> Result<Timestamp, String> {
    let complete = value.date.is_some() && value.time.is_some() && value.offset.is_some();
    let parsed = complete
        .then(|| value.to_string().parse::<Timestamp>())
        .and_then(Result::ok);
    parsed.ok_or_else(|| {
        format!(
            "`{field} = {value}` must be a date, time and UTC offset, \
             such as `2026-11-12T18:30:00-08:00`"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Kind, Location, Status};

    const VALID: &str = r#"+++
title = "Async <Rust>"
start = 2026-11-12T18:30:00-08:00
end = 2026-11-12T20:30:00-08:00
kind = "talk"
status = "scheduled"

[location]
name = "Library"
address = "801 SW 10th Ave"

[[speakers]]
name = "Ferris"
url = "https://example.com/ferris"

[[links]]
kind = "slides"
url = "https://example.com/slides"
+++

Hello *world*.
"#;

    fn parse_valid(text: &str) -> Result<Event, String> {
        parse(Path::new("events/2026/2026-11-12-async.md"), "2026", text)
    }

    #[test]
    fn parses_a_complete_event() {
        let event = parse_valid(VALID).unwrap();
        assert_eq!(event.slug.to_string(), "2026-11-12-async");
        assert_eq!(event.title, "Async <Rust>");
        assert_eq!(event.start.to_string(), "2026-11-13T02:30:00Z");
        assert_eq!(event.kind, Kind::Talk);
        assert_eq!(event.status, Status::Scheduled);
        assert!(matches!(event.location, Location::Venue { .. }));
        assert_eq!(event.speakers.len(), 1);
        assert_eq!(event.links.len(), 1);
        assert!(event.body_html.contains("<em>world</em>"));
    }

    #[test]
    fn online_location_and_optional_fields() {
        let text = "+++\ntitle = \"x\"\nstart = 2026-11-12T18:30:00-08:00\nkind = \"hack\"\n\
                    status = \"cancelled\"\nlocation = \"online\"\n+++\n";
        let event = parse_valid(text).unwrap();
        assert_eq!(event.location, Location::Online);
        assert_eq!(event.end, None);
        assert_eq!(event.status, Status::Cancelled);
    }

    #[test]
    fn rescheduled_event_keeps_its_slug() {
        // The file name says the 12th. The event now starts on the 19th.
        let text = VALID
            .replace("2026-11-12T18:30", "2026-11-19T18:30")
            .replace("2026-11-12T20:30", "2026-11-19T20:30");
        let event = parse_valid(&text).unwrap();
        assert_eq!(event.slug.to_string(), "2026-11-12-async");
        assert_eq!(event.start.to_string(), "2026-11-20T02:30:00Z");
    }

    #[test]
    fn rejects_invalid_files_with_a_message() {
        let cases = [
            (VALID.replace("+++\ntitle", "title"), "must start with"),
            (VALID.replacen("+++\n\n", "\n", 1), "must start with"),
            (
                VALID.replace("\"Async <Rust>\"", "\"  \""),
                "`title` must not be empty",
            ),
            (
                VALID.replace("kind = \"talk\"", "kind = \"party\""),
                "unknown variant",
            ),
            (
                VALID.replace("status = \"scheduled\"", "status = \"maybe\""),
                "unknown variant",
            ),
            (VALID.replace("T18:30:00-08:00", "T18:30:00"), "UTC offset"),
            (
                VALID.replace("2026-11-12T20:30", "2026-11-12T17:30"),
                "`end` must be after",
            ),
            (
                VALID.replace("title =", "venue = \"x\"\ntitle ="),
                "unknown field",
            ),
            (
                VALID.replace("[location]\nname = \"Library\"", "[location]\nname = \"\""),
                "name",
            ),
            (
                VALID.replace(
                    "[location]\nname = \"Library\"\naddress = \"801 SW 10th Ave\"",
                    "location = \"Library\"",
                ),
                "\"online\"",
            ),
            (
                VALID.replace("https://example.com/slides", "javascript:alert(1)"),
                "https://",
            ),
        ];
        for (text, expected) in cases {
            let err = parse_valid(&text).expect_err(expected);
            assert!(err.contains(expected), "expected {expected:?} in {err:?}");
        }
    }

    #[test]
    fn rejects_bad_paths() {
        let err = parse(Path::new("events/2025/2026-11-12-x.md"), "2025", VALID).unwrap_err();
        assert!(err.contains("events/2026/"), "{err}");
        let err = parse(Path::new("events/2026/2026-11-12-x.txt"), "2026", VALID).unwrap_err();
        assert!(err.contains(".md"), "{err}");
        let err = parse(Path::new("events/2026/Nov-12.md"), "2026", VALID).unwrap_err();
        assert!(err.contains("YYYY-MM-DD-name.md"), "{err}");
    }

    #[test]
    fn loads_the_repository_events() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("events");
        let events = events(&dir).unwrap_or_else(|errors| {
            panic!(
                "{}",
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        });
        assert!(!events.is_empty());
    }
}
