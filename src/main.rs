//! Builds the PDX Rust website from `site.toml`, `events/` and `static/`.
//!
//! ```text
//! build-site build [--out DIR] [--now RFC3339]   write the site (default: dist)
//! build-site check                               validate site.toml and events/ only
//! ```
//!
//! Run from the repository root.

mod event;
mod load;
mod markdown;
mod render;

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use jiff::Timestamp;

/// Usage text printed on a bad command line.
const USAGE: &str = "usage: build-site build [--out DIR] [--now RFC3339]\n       build-site check";

/// What the command line asked for.
enum Command {
    /// Write the site to `out`, splitting upcoming and past events at `now`.
    Build {
        /// Output directory.
        out: PathBuf,
        /// The build time.
        now: Timestamp,
    },
    /// Validate the inputs and write nothing.
    Check,
}

fn main() -> ExitCode {
    let command = match parse_args(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            eprintln!("{} error(s)", errors.len());
            ExitCode::FAILURE
        }
    }
}

/// Parses the arguments after the program name.
fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let command = args.next().unwrap_or_else(|| "build".into());
    let mut out = PathBuf::from("dist");
    let mut now = Timestamp::now();
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
        match (command.as_str(), flag.as_str()) {
            ("build", "--out") => out = value()?.into(),
            ("build", "--now") => {
                now = value()?.parse().map_err(|e| format!("--now: {e}"))?;
            }
            _ => return Err(format!("unexpected argument `{flag}`")),
        }
    }
    match command.as_str() {
        "build" => Ok(Command::Build { out, now }),
        "check" => Ok(Command::Check),
        other => Err(format!("unknown command `{other}`")),
    }
}

/// Loads and validates the inputs, then writes the site for [`Command::Build`].
fn run(command: Command) -> Result<(), Vec<String>> {
    let site = load::site(Path::new("site.toml")).map_err(|e| vec![e.to_string()]);
    let events = load::events(Path::new("events"))
        .map_err(|errors| errors.iter().map(ToString::to_string).collect::<Vec<_>>());
    let (site, events) = match (site, events) {
        (Ok(site), Ok(events)) => (site, events),
        (site, events) => {
            return Err(site
                .err()
                .into_iter()
                .flatten()
                .chain(events.err().into_iter().flatten())
                .collect());
        }
    };

    match command {
        Command::Check => {
            println!("ok: site.toml and {} event(s)", events.len());
            Ok(())
        }
        Command::Build { out, now } => {
            let pages = render::site(&site, &events, now);
            write(&out, &pages).map_err(|e| vec![e])?;
            println!("wrote {} page(s) to {}", pages.len(), out.display());
            Ok(())
        }
    }
}

/// Writes `pages` and copies `static/` into `out`. Existing files are overwritten.
fn write(out: &Path, pages: &[render::Page]) -> Result<(), String> {
    let write_file = |path: &Path, contents: &[u8]| -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(path, contents).map_err(|e| format!("{}: {e}", path.display()))
    };
    for page in pages {
        write_file(&out.join(&page.path), page.html.as_bytes())?;
    }
    let static_dir = Path::new("static");
    let entries = fs::read_dir(static_dir).map_err(|e| format!("{}: {e}", static_dir.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|e| format!("{}: {e}", static_dir.display()))?
            .path();
        // `static/` is flat. A subdirectory would be silently skipped, so refuse it.
        if !path.is_file() {
            return Err(format!(
                "{}: static/ must contain only files",
                path.display()
            ));
        }
        let contents = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        write_file(&out.join(path.file_name().unwrap_or_default()), &contents)?;
    }
    Ok(())
}
