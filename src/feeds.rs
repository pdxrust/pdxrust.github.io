//! RSS 2.0 and Atom feeds of events.
//!
//! Both feeds list the newest [`FEED_LENGTH`] events by start time, upcoming ones included. An
//! entry's date is the event's start, so rescheduling an event updates its entry.

use std::fmt::Write as _;

use jiff::{Timestamp, tz::TimeZone};

use crate::{
    event::{Event, Status},
    load::Site,
    render::{event_url, place, when},
};

/// How many events each feed lists.
const FEED_LENGTH: usize = 25;

/// The RSS 2.0 feed, served at `/feed.xml`.
pub fn rss(site: &Site, base_url: &str, events: &[Event], now: Timestamp) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n\
         <channel>\n\
         <title>{title}</title>\n\
         <link>{base_url}/</link>\n\
         <description>{description}</description>\n\
         <language>en-us</language>\n\
         <lastBuildDate>{now}</lastBuildDate>\n\
         <atom:link href=\"{base_url}/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>\n",
        title = escape(&site.title),
        description = escape(&site.description),
        base_url = escape(base_url),
        now = rfc2822(now),
    );
    for event in newest(events) {
        let url = escape(&event_url(base_url, &event.slug));
        let _ = write!(
            out,
            "<item>\n\
             <title>{title}</title>\n\
             <link>{url}</link>\n\
             <guid isPermaLink=\"true\">{url}</guid>\n\
             <pubDate>{date}</pubDate>\n\
             <description>{content}</description>\n\
             </item>\n",
            title = escape(&entry_title(event)),
            date = rfc2822(event.start),
            content = escape(&content_html(event)),
        );
    }
    out.push_str("</channel>\n</rss>\n");
    out
}

/// The Atom feed, served at `/atom.xml`.
pub fn atom(site: &Site, base_url: &str, events: &[Event], now: Timestamp) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <feed xmlns=\"http://www.w3.org/2005/Atom\" xml:lang=\"en-us\">\n\
         <id>{base_url}/</id>\n\
         <title>{title}</title>\n\
         <subtitle>{description}</subtitle>\n\
         <updated>{now}</updated>\n\
         <author><name>{title}</name></author>\n\
         <link rel=\"alternate\" type=\"text/html\" href=\"{base_url}/\"/>\n\
         <link rel=\"self\" type=\"application/atom+xml\" href=\"{base_url}/atom.xml\"/>\n",
        title = escape(&site.title),
        description = escape(&site.description),
        base_url = escape(base_url),
        now = now,
    );
    for event in newest(events) {
        let url = escape(&event_url(base_url, &event.slug));
        let _ = write!(
            out,
            "<entry>\n\
             <id>{url}</id>\n\
             <title>{title}</title>\n\
             <updated>{date}</updated>\n\
             <link rel=\"alternate\" type=\"text/html\" href=\"{url}\"/>\n\
             <content type=\"html\">{content}</content>\n\
             </entry>\n",
            title = escape(&entry_title(event)),
            date = event.start,
            content = escape(&content_html(event)),
        );
    }
    out.push_str("</feed>\n");
    out
}

/// The newest [`FEED_LENGTH`] events, newest first. `events` is sorted oldest first.
fn newest(events: &[Event]) -> impl Iterator<Item = &Event> {
    events.iter().rev().take(FEED_LENGTH)
}

/// The entry title, prefixed when the event is cancelled.
fn entry_title(event: &Event) -> String {
    match event.status {
        Status::Scheduled => event.title.clone(),
        Status::Cancelled => format!("Cancelled: {}", event.title),
    }
}

/// The entry body as HTML: when and where, speakers, then the sanitized description.
fn content_html(event: &Event) -> String {
    let mut html = format!(
        "<p>{} · {}</p>",
        escape(&when(event)),
        escape(&place(&event.location))
    );
    if !event.speakers.is_empty() {
        let names: Vec<_> = event.speakers.iter().map(|s| escape(&s.name)).collect();
        let _ = write!(html, "<p>{}</p>", names.join(", "));
    }
    html.push_str(&event.body_html);
    html
}

/// `t` in RFC 2822 form with a `+0000` offset, as RSS requires.
fn rfc2822(t: Timestamp) -> String {
    jiff::fmt::rfc2822::to_string(&t.to_zoned(TimeZone::UTC))
        .expect("every Timestamp in UTC is representable in RFC 2822")
}

/// Escapes text for XML character data and attribute values.
fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            // XML 1.0 forbids other control characters, even escaped.
            c if c.is_control() && !matches!(c, '\t' | '\n' | '\r') => {}
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::load;

    fn fixture() -> (Site, Vec<Event>) {
        let site: Site = toml::from_str(
            r#"
            title = "PDX Rust"
            headline = "Rustaceans of Portland"
            description = "Rust & friends"
            signup_url = "https://buttondown.com/pdxrust"
            discord_url = "https://discord.gg/ZpvYBCP6xc"
            repo_url = "https://github.com/pdxrust/pdxrust.github.io"
            "#,
        )
        .unwrap();
        let event = |stem: &str, title: &str, status: &str| {
            let text = format!(
                "+++\ntitle = \"{title}\"\nstart = 2026-11-12T18:30:00-08:00\nkind = \"talk\"\n\
                 status = \"{status}\"\nlocation = \"online\"\n+++\nA <b>body</b>.\n"
            );
            load::parse(Path::new(&format!("events/2026/{stem}.md")), "2026", &text).unwrap()
        };
        let events = vec![
            event("2026-01-01-first", "<script>x</script> & co", "scheduled"),
            event("2026-02-01-second", "Second", "cancelled"),
        ];
        (site, events)
    }

    #[test]
    fn rss_escapes_and_links_absolutely() {
        let (site, events) = fixture();
        let xml = rss(
            &site,
            "https://example.test",
            &events,
            Timestamp::UNIX_EPOCH,
        );
        assert!(xml.contains("<description>Rust &amp; friends</description>"));
        assert!(xml.contains("<title>&lt;script&gt;x&lt;/script&gt; &amp; co</title>"));
        assert!(xml.contains("<link>https://example.test/events/2026-01-01-first/</link>"));
        assert!(xml.contains("<pubDate>Fri, 13 Nov 2026 02:30:00 +0000</pubDate>"));
        assert!(xml.contains("<title>Cancelled: Second</title>"));
        // Body HTML is escaped once more, as RSS description text.
        assert!(xml.contains("&amp;lt;b&amp;gt;body&amp;lt;/b&amp;gt;"));
        assert!(!xml.contains("<script") && !xml.contains("<b>"));
        // Newest first: the second file comes before the first.
        assert!(xml.find("2026-02-01-second") < xml.find("2026-01-01-first"));
    }

    #[test]
    fn atom_has_required_elements() {
        let (site, events) = fixture();
        let xml = atom(
            &site,
            "https://example.test",
            &events,
            Timestamp::UNIX_EPOCH,
        );
        assert!(xml.contains("<id>https://example.test/</id>"));
        assert!(xml.contains("<updated>1970-01-01T00:00:00Z</updated>"));
        assert!(xml.contains("<author><name>PDX Rust</name></author>"));
        assert!(xml.contains("<id>https://example.test/events/2026-01-01-first/</id>"));
        assert!(xml.contains("<updated>2026-11-13T02:30:00Z</updated>"));
        assert_eq!(xml.matches("<entry>").count(), 2);
    }
}
