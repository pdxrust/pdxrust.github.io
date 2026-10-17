//! Event types parsed from `events/YYYY/YYYY-MM-DD-name.md`.

use std::fmt;

use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Deserializer, de};

/// How long an event without an `end` is assumed to run, for the upcoming/past split.
const DEFAULT_LENGTH: SignedDuration = SignedDuration::from_hours(2);

/// A published event.
#[derive(Debug, Clone)]
pub struct Event {
    /// Stable identifier, used for the page URL and the iCal UID.
    pub slug: Slug,
    /// Event title, plain text.
    pub title: String,
    /// Start time.
    pub start: Timestamp,
    /// End time, after `start` when present.
    pub end: Option<Timestamp>,
    /// What kind of event this is.
    pub kind: Kind,
    /// Whether the event is still going ahead.
    pub status: Status,
    /// Where the event takes place.
    pub location: Location,
    /// Speakers, in listed order.
    pub speakers: Vec<Speaker>,
    /// Related links, in listed order.
    pub links: Vec<Link>,
    /// Sanitized HTML rendered from the Markdown body.
    pub body_html: String,
    /// Sanitized HTML of the body's first paragraph, empty when it has none.
    pub summary_html: String,
}

impl Event {
    /// Returns true until the event has ended at `now`.
    ///
    /// An event without an `end` counts as ending [`DEFAULT_LENGTH`] after it starts.
    pub fn is_upcoming(&self, now: Timestamp) -> bool {
        self.end.unwrap_or(self.start + DEFAULT_LENGTH) >= now
    }
}

/// The front matter of an event file, before validation.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrontMatter {
    /// Event title.
    pub title: String,
    /// Start time, an RFC 3339 date-time with an offset.
    pub start: toml::value::Datetime,
    /// End time, an RFC 3339 date-time with an offset.
    pub end: Option<toml::value::Datetime>,
    /// What kind of event this is.
    pub kind: Kind,
    /// Whether the event is still going ahead.
    pub status: Status,
    /// Where the event takes place.
    pub location: Location,
    /// Speakers, in listed order.
    #[serde(default)]
    pub speakers: Vec<Speaker>,
    /// Related links, in listed order.
    #[serde(default)]
    pub links: Vec<Link>,
}

/// The file stem of an event, `YYYY-MM-DD-name`.
///
/// The date is the date the event was first announced for. It is never compared with `start`, so
/// a rescheduled event keeps its slug, URL and iCal UID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slug(String);

impl Slug {
    /// Validates `stem` as a slug.
    pub fn parse(stem: &str) -> Result<Self, String> {
        let err = || {
            format!(
                "file name `{stem}` must be `YYYY-MM-DD-name.md`, where `name` is lowercase \
                 letters, digits and single hyphens"
            )
        };
        let (date, name) = stem.split_at_checked(10).ok_or_else(err)?;
        let name = name.strip_prefix('-').ok_or_else(err)?;
        // Checks the shape before parsing so `2026-1-1x-...` doesn't reach jiff's lenient parser.
        let date_shape = date.bytes().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        });
        if !date_shape {
            return Err(err());
        }
        date.parse::<jiff::civil::Date>()
            .map_err(|e| format!("file name `{stem}` has an invalid date: {e}"))?;
        let name_ok = !name.is_empty()
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--")
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !name_ok {
            return Err(err());
        }
        Ok(Self(stem.to_owned()))
    }

    /// The four-digit year at the start of the slug.
    pub fn year(&self) -> &str {
        &self.0[..4]
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What kind of event this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// One or more talks.
    Talk,
    /// A hack night.
    Hack,
    /// A social gathering.
    Social,
    /// Anything else.
    Other,
}

impl Kind {
    /// Human-readable label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Talk => "Talk",
            Self::Hack => "Hack night",
            Self::Social => "Social",
            Self::Other => "Event",
        }
    }
}

/// Whether the event is still going ahead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Going ahead as listed.
    Scheduled,
    /// Called off. The page stays up and the iCal entry is marked cancelled.
    Cancelled,
}

/// Where the event takes place.
///
/// Written as `location = "online"`, or as a table with `name` and an optional `address`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// Online only.
    Online,
    /// A physical venue.
    Venue {
        /// Venue name.
        name: String,
        /// Street address.
        address: Option<String>,
    },
}

impl<'de> Deserialize<'de> for Location {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // A hand-written visitor, so a bad value reports what is accepted. serde's untagged enums
        // report only "did not match any variant".
        struct Visitor;

        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Venue {
            name: String,
            address: Option<String>,
        }

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Location;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(r#""online" or a table with `name` and optional `address`"#)
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Location, E> {
                if v == "online" {
                    Ok(Location::Online)
                } else {
                    Err(E::invalid_value(de::Unexpected::Str(v), &self))
                }
            }

            fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<Location, A::Error> {
                let venue = Venue::deserialize(de::value::MapAccessDeserializer::new(map))?;
                if venue.name.trim().is_empty() {
                    return Err(de::Error::custom("location `name` must not be empty"));
                }
                Ok(Location::Venue {
                    name: venue.name,
                    address: venue.address.filter(|a| !a.trim().is_empty()),
                })
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// A speaker at the event.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Speaker {
    /// Display name.
    pub name: String,
    /// Personal or social link.
    pub url: Option<HttpUrl>,
}

/// A link related to the event.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    /// What the link points to.
    pub kind: LinkKind,
    /// Target URL.
    pub url: HttpUrl,
}

/// What a [`Link`] points to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    /// Slide deck.
    Slides,
    /// Source repository.
    Repo,
    /// Video recording.
    Recording,
    /// Anything else.
    Other,
}

impl LinkKind {
    /// Human-readable label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Slides => "Slides",
            Self::Repo => "Code",
            Self::Recording => "Recording",
            Self::Other => "Link",
        }
    }
}

/// An absolute `http://` or `https://` URL with no whitespace or control characters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct HttpUrl(String);

impl HttpUrl {
    /// The URL as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for HttpUrl {
    type Error = String;

    fn try_from(url: String) -> Result<Self, String> {
        let rest = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .ok_or_else(|| format!("URL `{url}` must start with https:// or http://"))?;
        if rest.is_empty() || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(format!("URL `{url}` is not a valid absolute URL"));
        }
        Ok(Self(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_accepts_well_formed_stems() {
        let slug = Slug::parse("2026-11-12-async-rust-2").unwrap();
        assert_eq!(slug.year(), "2026");
    }

    #[test]
    fn slug_rejects_malformed_stems() {
        for stem in [
            "2026-11-12",
            "2026-11-12-",
            "2026-11-12-Upper",
            "2026-11-12-two--hyphens",
            "2026-11-12-trailing-",
            "2026-11-12_name",
            "2026-02-30-bad-date",
            "26-11-12-short-year",
            "2026-11-12-ünï",
        ] {
            assert!(Slug::parse(stem).is_err(), "{stem} should be rejected");
        }
    }

    #[test]
    fn url_rejects_other_schemes() {
        for url in [
            "javascript:alert(1)",
            "ftp://x",
            "https://",
            "https://a b",
            "//x",
        ] {
            assert!(
                HttpUrl::try_from(url.to_owned()).is_err(),
                "{url} should be rejected"
            );
        }
    }
}
