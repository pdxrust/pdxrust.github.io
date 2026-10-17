//! iCalendar (RFC 5545) output: the `events.ics` feed and one file per event.

use std::fmt::Write as _;

use jiff::{Timestamp, tz::TimeZone};

use crate::{
    event::{Event, Status},
    render::{event_url, place},
};

/// Domain part of every UID.
///
/// It's a fixed name, not the site's host, so UIDs survive a move to another domain. Calendar
/// clients use the UID to match an updated event to the one they already have.
const UID_DOMAIN: &str = "pdxrust";

/// Longest content line in octets, not counting the CRLF (RFC 5545 section 3.1).
const LINE_LIMIT: usize = 75;

/// A calendar of `events`. `now` is the `DTSTAMP` of every event.
pub fn calendar(name: &str, base_url: &str, events: &[Event], now: Timestamp) -> String {
    let mut out = String::new();
    line(&mut out, "BEGIN:VCALENDAR");
    line(&mut out, "VERSION:2.0");
    line(&mut out, "PRODID:-//PDX Rust//build-site//EN");
    line(&mut out, "CALSCALE:GREGORIAN");
    line(&mut out, "METHOD:PUBLISH");
    line(&mut out, &format!("X-WR-CALNAME:{}", text(name)));
    // Subscribing clients poll at this interval. The site rebuilds daily.
    line(&mut out, "REFRESH-INTERVAL;VALUE=DURATION:PT12H");
    line(&mut out, "X-PUBLISHED-TTL:PT12H");
    for event in events {
        vevent(&mut out, base_url, event, now);
    }
    line(&mut out, "END:VCALENDAR");
    out
}

/// Appends one `VEVENT`.
fn vevent(out: &mut String, base_url: &str, event: &Event, now: Timestamp) {
    let url = event_url(base_url, &event.slug);
    let end = event
        .end
        .unwrap_or(event.start + crate::event::DEFAULT_LENGTH);
    let status = match event.status {
        Status::Scheduled => "CONFIRMED",
        Status::Cancelled => "CANCELLED",
    };
    let mut description = event.summary_text.clone();
    if !description.is_empty() {
        description.push_str("\n\n");
    }
    description.push_str(&url);

    line(out, "BEGIN:VEVENT");
    line(out, &format!("UID:{}@{UID_DOMAIN}", event.slug));
    line(out, &format!("DTSTAMP:{}", utc(now)));
    line(out, &format!("DTSTART:{}", utc(event.start)));
    line(out, &format!("DTEND:{}", utc(end)));
    line(out, &format!("SUMMARY:{}", text(&event.title)));
    line(out, &format!("DESCRIPTION:{}", text(&description)));
    line(out, &format!("LOCATION:{}", text(&place(&event.location))));
    line(out, &format!("URL:{url}"));
    line(out, &format!("STATUS:{status}"));
    line(out, "END:VEVENT");
}

/// `t` as a UTC date-time, such as `20261113T023000Z`.
///
/// UTC times need no `VTIMEZONE` block, and every client converts them to its own zone.
fn utc(t: Timestamp) -> String {
    t.to_zoned(TimeZone::UTC)
        .strftime("%Y%m%dT%H%M%SZ")
        .to_string()
}

/// Escapes a TEXT value (RFC 5545 section 3.3.11).
fn text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            ';' => escaped.push_str("\\;"),
            ',' => escaped.push_str("\\,"),
            '\n' => escaped.push_str("\\n"),
            // Other control characters aren't allowed in TEXT. Event data never needs them.
            c if c.is_control() => {}
            c => escaped.push(c),
        }
    }
    escaped
}

/// Appends `content` as one or more folded lines, each ending in CRLF.
///
/// Lines are split at 75 octets, on a character boundary, and continued with a leading space.
fn line(out: &mut String, content: &str) {
    let mut rest = content;
    let mut limit = LINE_LIMIT;
    loop {
        if rest.len() <= limit {
            out.push_str(rest);
            out.push_str("\r\n");
            return;
        }
        let mut split = limit;
        while !rest.is_char_boundary(split) {
            split -= 1;
        }
        let (head, tail) = rest.split_at(split);
        let _ = write!(out, "{head}\r\n ");
        rest = tail;
        // The leading space counts toward the next line's length.
        limit = LINE_LIMIT - 1;
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::load;

    fn event(extra: &str, title: &str) -> Event {
        let text = format!(
            "+++\ntitle = \"{title}\"\nstart = 2026-11-12T18:30:00-08:00\nkind = \"talk\"\n\
             location = {{ name = \"PSU\", address = \"1900 SW 4th Ave, Portland, OR\" }}\n\
             {extra}+++\nFirst; paragraph, here.\n\nSecond.\n"
        );
        load::parse(Path::new("events/2026/2026-11-12-x.md"), "2026", &text).unwrap()
    }

    fn unfold(ics: &str) -> String {
        ics.replace("\r\n ", "")
    }

    #[test]
    fn writes_a_complete_event() {
        let now: Timestamp = "2026-10-17T12:00:00Z".parse().unwrap();
        let events = [event("status = \"scheduled\"\n", "Talk")];
        let ics = calendar("PDX Rust", "https://example.test", &events, now);
        assert!(ics.starts_with("BEGIN:VCALENDAR\r\n") && ics.ends_with("END:VCALENDAR\r\n"));
        assert!(!ics.replace("\r\n", "").contains('\n'), "bare LF in output");
        let ics = unfold(&ics);
        for expected in [
            "UID:2026-11-12-x@pdxrust\r\n",
            "DTSTAMP:20261017T120000Z\r\n",
            "DTSTART:20261113T023000Z\r\n",
            // No `end`, so the default length applies.
            "DTEND:20261113T043000Z\r\n",
            "SUMMARY:Talk\r\n",
            "DESCRIPTION:First\\; paragraph\\, here.\\n\\nhttps://example.test/events/2026-11-12-x/\r\n",
            "LOCATION:PSU\\, 1900 SW 4th Ave\\, Portland\\, OR\r\n",
            "STATUS:CONFIRMED\r\n",
        ] {
            assert!(ics.contains(expected), "missing {expected:?} in\n{ics}");
        }
    }

    #[test]
    fn cancelled_events_say_so() {
        let events = [event("status = \"cancelled\"\n", "Off")];
        let ics = calendar(
            "PDX Rust",
            "https://example.test",
            &events,
            Timestamp::UNIX_EPOCH,
        );
        assert!(ics.contains("STATUS:CANCELLED\r\n"));
    }

    #[test]
    fn folds_long_lines_on_character_boundaries() {
        let mut out = String::new();
        let content = format!("SUMMARY:{}", "é".repeat(100));
        line(&mut out, &content);
        for physical in out.split_terminator("\r\n") {
            assert!(
                physical.len() <= LINE_LIMIT,
                "{} octets: {physical:?}",
                physical.len()
            );
        }
        assert_eq!(unfold(&out), format!("{content}\r\n"));
    }

    #[test]
    fn escapes_text_values() {
        assert_eq!(text("a\\b;c,d\ne\u{7}"), "a\\\\b\\;c\\,d\\ne");
    }
}
