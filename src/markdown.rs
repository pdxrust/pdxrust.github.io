//! Markdown to sanitized HTML.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};

/// Renders untrusted Markdown to HTML that is safe to embed in a page.
///
/// Raw HTML in the source is shown as escaped text. The output then goes through [`ammonia`],
/// which removes unsafe URL schemes such as `javascript:` from links and images.
pub fn to_html(markdown: &str) -> String {
    render(markdown.len(), events(markdown))
}

/// Renders only the first paragraph of `markdown`, sanitized the same way as [`to_html`].
///
/// Returns an empty string when there is no paragraph.
pub fn first_paragraph_html(markdown: &str) -> String {
    let paragraph = events(markdown)
        .skip_while(|event| !matches!(event, Event::Start(Tag::Paragraph)))
        .scan(false, |done, event| {
            if *done {
                return None;
            }
            *done = matches!(event, Event::End(TagEnd::Paragraph));
            Some(event)
        });
    render(markdown.len(), paragraph)
}

/// Parses `markdown`, turning raw HTML into text.
fn events(markdown: &str) -> impl Iterator<Item = Event<'_>> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    // pulldown-cmark has no option to disable raw HTML, so it is turned into text here. Escaped
    // text shows a contributor what was dropped. ammonia is the backstop for everything else.
    Parser::new_ext(markdown, options).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    })
}

/// Writes `events` as HTML and sanitizes the result.
fn render<'a>(source_len: usize, events: impl Iterator<Item = Event<'a>>) -> String {
    let mut unsafe_html = String::with_capacity(source_len * 3 / 2);
    html::push_html(&mut unsafe_html, events);
    ammonia::clean(&unsafe_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_html_is_escaped() {
        let html = to_html("hi <script>alert(1)</script>\n\n<div onclick=x>block</div>\n");
        assert!(!html.contains("<script"), "{html}");
        assert!(!html.contains("<div"), "{html}");
        assert!(html.contains("&lt;script&gt;"), "{html}");
    }

    #[test]
    fn unsafe_link_schemes_are_removed() {
        let html = to_html("[x](javascript:alert(1)) ![y](data:text/html,hi)");
        assert!(!html.contains("javascript:"), "{html}");
        assert!(!html.contains("data:"), "{html}");
    }

    #[test]
    fn first_paragraph_stops_at_the_first_paragraph() {
        let html = first_paragraph_html("# Heading\n\nOne <b>x</b>.\n\nTwo.\n");
        assert_eq!(html.trim(), "<p>One &lt;b&gt;x&lt;/b&gt;.</p>");
        assert_eq!(first_paragraph_html("# Only a heading\n"), "");
    }

    #[test]
    fn ordinary_markdown_survives() {
        let html = to_html("Some *emphasis* and [a link](https://www.rust-lang.org/).");
        assert!(html.contains("<em>emphasis</em>"), "{html}");
        assert!(
            html.contains(r#"href="https://www.rust-lang.org/""#),
            "{html}"
        );
    }
}
