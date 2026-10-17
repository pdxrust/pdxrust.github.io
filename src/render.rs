//! Renders the HTML pages with Leptos.
//!
//! Every page is rendered to a string once at build time. Pages contain no scripts.

use std::path::PathBuf;

use jiff::{Timestamp, Zoned, tz::TimeZone};
use leptos::prelude::*;

use crate::{
    event::{Event, Kind, Location, Status},
    load::Site,
};

/// Content-Security-Policy for every page.
///
/// It's a `<meta>` tag because GitHub Pages can't set response headers. Browsers ignore
/// `frame-ancestors`, `report-uri` and `sandbox` in a `<meta>` policy, so it doesn't set them.
const CSP: &str = "default-src 'none'; style-src 'self'; img-src 'self'; base-uri 'none'; \
                   form-action 'none'";

/// Time zone that event times are shown in.
const ZONE: &str = "America/Los_Angeles";

/// How many past talks the home page lists.
const RECENT_TALKS: usize = 5;

/// License of the Rust logo artwork the site's logo is adapted from.
const CC_BY: &str = "https://creativecommons.org/licenses/by/4.0/";

/// A rendered page and where it goes in the output directory.
pub struct Page {
    /// Path relative to the output directory.
    pub path: PathBuf,
    /// Complete HTML document.
    pub html: String,
}

/// Renders every page of the site. Events ending before `now` go to the past page.
pub fn site(site: &Site, events: &[Event], now: Timestamp) -> Vec<Page> {
    let (upcoming, mut past): (Vec<_>, Vec<_>) =
        events.iter().cloned().partition(|e| e.is_upcoming(now));
    past.reverse();
    let recent: Vec<_> = past
        .iter()
        .filter(|e| e.kind == Kind::Talk && e.status == Status::Scheduled)
        .take(RECENT_TALKS)
        .cloned()
        .collect();

    let mut pages = vec![
        page("index.html", site, None, Nav::Upcoming, {
            let site = site.clone();
            move || view! { <Home site upcoming recent /> }
        }),
        page(
            "past/index.html",
            site,
            Some("Past events".into()),
            Nav::Past,
            move || {
                view! { <Past past /> }
            },
        ),
        page(
            "404.html",
            site,
            Some("Page not found".into()),
            Nav::None,
            || view! { <NotFound /> },
        ),
    ];
    for event in events {
        let path = format!("events/{}/index.html", event.slug);
        let event = event.clone();
        pages.push(page(
            &path,
            site,
            Some(event.title.clone()),
            Nav::None,
            move || {
                view! { <EventPage event /> }
            },
        ));
    }
    pages
}

/// Which navigation entry is the current page.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Nav {
    /// The home page.
    Upcoming,
    /// The past events page.
    Past,
    /// A page with no navigation entry.
    None,
}

/// Renders `body` inside the shared document shell.
///
/// `body` lays out its own width. Most pages wrap their content in `.wrap`.
fn page<V: IntoView + 'static>(
    path: &str,
    site: &Site,
    title: Option<String>,
    nav: Nav,
    body: impl FnOnce() -> V + 'static,
) -> Page {
    let site = site.clone();
    let html = Owner::new().with(move || {
        let title = match title {
            Some(title) => format!("{title} | {}", site.title),
            None => format!("{} | {}", site.title, site.headline),
        };
        view! {
            <html lang="en">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <meta http-equiv="Content-Security-Policy" content=CSP />
                    <meta name="description" content=site.description.clone() />
                    <title>{title}</title>
                    <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
                    <link rel="stylesheet" href="/style.css" />
                </head>
                <body>
                    <a class="skip" href="#main">
                        "Skip to content"
                    </a>
                    <header class="site">
                        <div class="wrap row">
                            <a class="brand" href="/" aria-label="PDX Rust home">
                                <img src="/logo.svg" alt="" width="52" height="52" />
                                <span class="wordmark">
                                    "pdx"<span class="accent">"rust"</span>
                                </span>
                            </a>
                            <nav aria-label="Main">
                                <NavLink href="/" label="Upcoming" current=nav == Nav::Upcoming />
                                <NavLink
                                    href="/past/"
                                    label="Past events"
                                    current=nav == Nav::Past
                                />
                            </nav>
                        </div>
                    </header>
                    <main id="main">{body()}</main>
                    <Footer site />
                </body>
            </html>
        }
        .to_html()
    });
    Page {
        path: path.into(),
        html: format!("<!doctype html>{html}"),
    }
}

/// A navigation link, marked with `aria-current` on its own page.
#[component]
fn NavLink(href: &'static str, label: &'static str, current: bool) -> impl IntoView {
    view! {
        <a href=href aria-current=current.then_some("page")>
            {label}
        </a>
    }
}

/// Links to email, Discord and the source, and the trademark notice.
#[component]
fn Footer(site: Site) -> impl IntoView {
    view! {
        <footer class="site">
            <div class="wrap row">
                <nav aria-label="Elsewhere" class="elsewhere">
                    <a href=site.signup_url.as_str().to_owned()>
                        <img src="/icon-mail.svg" alt="" width="22" height="22" />
                        "Email"
                    </a>
                    <a href=site.discord_url.as_str().to_owned()>
                        <img src="/icon-discord.svg" alt="" width="22" height="22" />
                        "Discord"
                    </a>
                    <a href=site.repo_url.as_str().to_owned()>
                        <img src="/icon-github.svg" alt="" width="22" height="22" />
                        "Source on GitHub"
                    </a>
                </nav>
                <p class="legal">
                    "A community meetup, not an official Rust Project event or endorsed by the Rust \
                     Project. Rust and the Rust logo are trademarks of the Rust Foundation. The logo's \
                     gear is adapted from the Rust logo, © Rust Foundation, "
                    <a href=CC_BY>"CC-BY 4.0"</a> "."
                </p>
            </div>
        </footer>
    }
}

/// The home page: banner with signup links, the next event and recent talks.
#[component]
fn Home(site: Site, upcoming: Vec<Event>, recent: Vec<Event>) -> impl IntoView {
    view! {
        <section class="hero" aria-labelledby="hero-heading">
            <img class="skyline" src="/skyline.svg" alt="" />
            <div class="wrap hero-text">
                <h1 id="hero-heading">{site.headline}</h1>
                <p class="lede">{site.description}</p>
                <p class="cta">
                    <a class="button" href=site.signup_url.as_str().to_owned()>
                        <img src="/icon-mail-dark.svg" alt="" width="20" height="20" />
                        "Get event emails"
                    </a>
                    <a class="button secondary" href=site.discord_url.as_str().to_owned()>
                        "Join the Discord"
                    </a>
                </p>
            </div>
        </section>
        <div class="wrap">
            <NextEvent upcoming />
            {(!recent.is_empty())
                .then(|| {
                    view! {
                        <section class="recent" aria-labelledby="recent-heading">
                            <div class="section-head">
                                <h2 id="recent-heading" class="label">
                                    "Recent talks"
                                </h2>
                                <a href="/past/">"All past events"</a>
                            </div>
                            <EventRows events=recent />
                        </section>
                    }
                })}
        </div>
    }
}

/// The soonest upcoming event in full, and a one-line mention of any after it.
#[component]
fn NextEvent(upcoming: Vec<Event>) -> impl IntoView {
    let mut upcoming = upcoming.into_iter();
    let Some(next) = upcoming.next() else {
        return view! {
            <section aria-labelledby="next-heading">
                <h2 id="next-heading" class="label">
                    "Next event"
                </h2>
                <p class="empty">
                    "Nothing is scheduled right now. Sign up for email to hear about the next one."
                </p>
            </section>
        }
        .into_any();
    };
    let later: Vec<_> = upcoming.collect();
    let also = (!later.is_empty()).then(|| {
        let links = later
            .into_iter()
            .enumerate()
            .map(|(i, event)| {
                let href = format!("/events/{}/", event.slug);
                let date = local(event.start).strftime(", %a %b %-d").to_string();
                view! {
                    {(i > 0).then_some("; ")}
                    <a href=href>{event.title}</a>
                    {date}
                }
            })
            .collect_view();
        view! { <p class="also">"Also coming up: " {links}</p> }
    });

    let href = format!("/events/{}/", next.slug);
    let start = local(next.start);
    let speakers = speaker_names(&next);
    // `summary_html` was sanitized by `markdown::first_paragraph_html`. `inner_html` must be given
    // an expression. A string literal renders as a plain attribute named `inner_html`.
    let summary = next.summary_html.clone();
    view! {
        <section aria-labelledby="next-heading">
            <h2 id="next-heading" class="label">
                "Next event"
            </h2>
            <article class="featured">
                <time class="date-block" datetime=machine_time(next.start)>
                    <span class="dow">{start.strftime("%a").to_string()}</span>
                    <span class="day">{start.strftime("%-d").to_string()}</span>
                    <span class="month">{start.strftime("%b %Y").to_string()}</span>
                </time>
                <div class="featured-body">
                    <KindLine event=next.clone() />
                    <h3>
                        <a href=href.clone()>{next.title.clone()}</a>
                    </h3>
                    {(!speakers.is_empty()).then(|| view! { <p class="speakers">{speakers}</p> })}
                    <p class="meta">
                        {format!("{} · {}", time_range(&next), place(&next.location))}
                    </p>
                    <div class="summary" inner_html=summary></div>
                    <p class="actions">
                        <a href=href>"Details"</a>
                    </p>
                </div>
            </article>
            {also}
        </section>
    }
    .into_any()
}

/// The past events page, newest first.
#[component]
fn Past(past: Vec<Event>) -> impl IntoView {
    view! {
        <div class="wrap page">
            <h1>"Past events"</h1>
            {if past.is_empty() {
                view! { <p class="empty">"No past events yet."</p> }.into_any()
            } else {
                view! { <EventRows events=past /> }.into_any()
            }}
        </div>
    }
}

/// One row per event: month, title, and speakers or kind.
#[component]
fn EventRows(events: Vec<Event>) -> impl IntoView {
    let rows = events
        .into_iter()
        .map(|event| {
            let href = format!("/events/{}/", event.slug);
            let who = match speaker_names(&event) {
                names if names.is_empty() => event.kind.label().to_owned(),
                names => names,
            };
            let cancelled = (event.status == Status::Cancelled).then_some(" · Cancelled");
            view! {
                <li>
                    <time datetime=machine_time(
                        event.start,
                    )>{local(event.start).strftime("%b %Y").to_string()}</time>
                    <span class="title">
                        <a href=href>{event.title}</a>
                        <span class="who">{format!(" · {who}")} {cancelled}</span>
                    </span>
                </li>
            }
        })
        .collect_view();
    view! { <ul class="rows">{rows}</ul> }
}

/// The kind label, preceded by a badge when the event is cancelled.
#[component]
fn KindLine(event: Event) -> impl IntoView {
    view! {
        <p class="kind">
            {(event.status == Status::Cancelled)
                .then(|| view! { <strong class="badge">"Cancelled"</strong> })}
            <span>{event.kind.label()}</span>
        </p>
    }
}

/// A single event's page.
#[component]
fn EventPage(event: Event) -> impl IntoView {
    let speakers = (!event.speakers.is_empty()).then(|| {
        let items = event
            .speakers
            .iter()
            .map(|speaker| {
                let name = speaker.name.clone();
                match &speaker.url {
                    Some(url) => view! {
                        <li>
                            <a href=url.as_str().to_owned()>{name}</a>
                        </li>
                    }
                    .into_any(),
                    None => view! { <li>{name}</li> }.into_any(),
                }
            })
            .collect_view();
        view! {
            <h2>"Speakers"</h2>
            <ul class="plain">{items}</ul>
        }
    });
    let links = (!event.links.is_empty()).then(|| {
        let items = event
            .links
            .iter()
            .map(|link| {
                view! {
                    <li>
                        <a href=link.url.as_str().to_owned()>{link.kind.label()}</a>
                    </li>
                }
            })
            .collect_view();
        view! {
            <h2>"Links"</h2>
            <ul class="plain">{items}</ul>
        }
    });
    // `body_html` was sanitized by `markdown::to_html`. `inner_html` must be given an expression.
    // A string literal renders as a plain attribute named `inner_html`.
    let body = event.body_html.clone();
    let meta = format!("{} · {}", when(&event), place(&event.location));
    view! {
        <div class="wrap page">
            <article class="event">
                <KindLine event=event.clone() />
                <h1>{event.title.clone()}</h1>
                <p class="meta">
                    <time datetime=machine_time(event.start)>{meta}</time>
                </p>
                {speakers}
                <div class="body" inner_html=body></div>
                {links}
            </article>
            <p>
                <a href="/past/">"All past events"</a>
            </p>
        </div>
    }
}

/// The 404 page.
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="wrap page">
            <h1>"Page not found"</h1>
            <p>"That page doesn't exist. " <a href="/">"See upcoming events"</a> "."</p>
        </div>
    }
}

/// Speaker names joined with commas, empty when there are none.
fn speaker_names(event: &Event) -> String {
    event
        .speakers
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The location as one line of text.
fn place(location: &Location) -> String {
    match location {
        Location::Online => "Online".to_owned(),
        Location::Venue {
            name,
            address: None,
        } => name.clone(),
        Location::Venue {
            name,
            address: Some(address),
        } => format!("{name}, {address}"),
    }
}

/// `t` in the display time zone.
fn local(t: Timestamp) -> Zoned {
    // The tzdb is bundled into the binary (jiff `tzdb-bundle-always`), so this can't fail at
    // runtime on any host.
    let zone = TimeZone::get(ZONE).expect("bundled tzdb includes America/Los_Angeles");
    t.to_zoned(zone)
}

/// `t` as an RFC 3339 string with the local offset, for `<time datetime>`.
fn machine_time(t: Timestamp) -> String {
    local(t).strftime("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// Human-readable date and time range, such as
/// `Thursday, November 12, 2026, 6:30 PM – 8:30 PM PST`.
fn when(event: &Event) -> String {
    const DATE_TIME: &str = "%A, %B %-d, %Y, %-I:%M %p";
    let start = local(event.start);
    let Some(end) = event.end.map(local) else {
        return format!("{} {}", start.strftime(DATE_TIME), start.strftime("%Z"));
    };
    let end_format = if end.date() == start.date() {
        "%-I:%M %p"
    } else {
        DATE_TIME
    };
    format!(
        "{} – {} {}",
        start.strftime(DATE_TIME),
        end.strftime(end_format),
        end.strftime("%Z")
    )
}

/// Time range without the date, such as `6:30 PM – 8:30 PM PST`, for next to a date block.
///
/// Falls back to [`when`] for an event that ends on a later day.
fn time_range(event: &Event) -> String {
    let start = local(event.start);
    match event.end.map(local) {
        None => format!("{}", start.strftime("%-I:%M %p %Z")),
        Some(end) if end.date() == start.date() => {
            format!(
                "{} – {}",
                start.strftime("%-I:%M %p"),
                end.strftime("%-I:%M %p %Z")
            )
        }
        Some(_) => when(event),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::load;

    const SCHEDULED: &str = "status = \"scheduled\"\n";

    fn fixture() -> (Site, Vec<Event>) {
        let site: Site = toml::from_str(
            r#"
            title = "PDX Rust"
            headline = "Rustaceans of Portland"
            description = "Rust in Portland"
            signup_url = "https://buttondown.com/pdxrust"
            discord_url = "https://discord.gg/ZpvYBCP6xc"
            repo_url = "https://github.com/pdxrust/pdxrust.github.io"
            "#,
        )
        .unwrap();
        let event = |stem: &str, start: &str, title: &str, extra: &str| {
            let text = format!(
                "+++\ntitle = \"{title}\"\nstart = {start}\nkind = \"talk\"\n\
                 location = \"online\"\n{extra}+++\nBody.\n"
            );
            load::parse(
                &PathBuf::from(format!("events/2026/{stem}.md")),
                "2026",
                &text,
            )
            .unwrap()
        };
        let events = vec![
            event(
                "2026-01-10-old",
                "2026-01-10T18:00:00-08:00",
                "Old",
                SCHEDULED,
            ),
            event(
                "2026-11-12-new",
                "2026-11-12T18:30:00-08:00",
                "<script>x</script>",
                SCHEDULED,
            ),
            event(
                "2026-12-01-off",
                "2026-12-01T18:30:00-08:00",
                "Off",
                "end = 2026-12-01T20:00:00-08:00\nstatus = \"cancelled\"\n",
            ),
        ];
        (site, events)
    }

    fn find<'a>(pages: &'a [Page], path: &str) -> &'a str {
        &pages
            .iter()
            .find(|p| p.path == Path::new(path))
            .expect(path)
            .html
    }

    #[test]
    fn splits_upcoming_and_past_at_now() {
        let (site, events) = fixture();
        let now: Timestamp = "2026-06-01T00:00:00Z".parse().unwrap();
        let pages = super::site(&site, &events, now);
        let home = find(&pages, "index.html");
        let past = find(&pages, "past/index.html");
        // The next event is featured, the past talk is under "Recent talks".
        let (next, recent) = home
            .split_once("Recent talks")
            .expect("recent talks section");
        assert!(next.contains("/events/2026-11-12-new/") && !next.contains("2026-01-10-old"));
        assert!(recent.contains("/events/2026-01-10-old/"));
        assert!(next.contains("Also coming up") && next.contains("/events/2026-12-01-off/"));
        assert!(past.contains("/events/2026-01-10-old/") && !past.contains("2026-11-12-new"));
        assert!(find(&pages, "events/2026-01-10-old/index.html").contains("<h1>Old</h1>"));
    }

    #[test]
    fn home_without_events_says_so() {
        let (site, _) = fixture();
        let pages = super::site(&site, &[], Timestamp::UNIX_EPOCH);
        let home = find(&pages, "index.html");
        assert!(home.contains("Nothing is scheduled right now"));
        assert!(!home.contains("Recent talks"));
    }

    #[test]
    fn event_without_end_stays_upcoming_for_two_hours() {
        let (_, events) = fixture();
        let start = events[0].start;
        assert!(events[0].is_upcoming(start + jiff::SignedDuration::from_mins(119)));
        assert!(!events[0].is_upcoming(start + jiff::SignedDuration::from_mins(121)));
    }

    #[test]
    fn output_is_escaped_and_script_free() {
        let (site, events) = fixture();
        let pages = super::site(&site, &events, Timestamp::UNIX_EPOCH);
        for page in &pages {
            assert!(page.html.starts_with("<!doctype html><html lang=\"en\">"));
            assert!(!page.html.contains("<script"), "{}", page.path.display());
        }
        assert!(find(&pages, "index.html").contains("&lt;script&gt;x&lt;/script&gt;"));
        assert!(find(&pages, "events/2026-12-01-off/index.html").contains(">Cancelled<"));
        assert!(!find(&pages, "events/2026-11-12-new/index.html").contains(">Cancelled<"));
    }

    #[test]
    fn formats_times_in_pacific() {
        let (_, events) = fixture();
        assert_eq!(when(&events[1]), "Thursday, November 12, 2026, 6:30 PM PST");
        assert_eq!(
            when(&events[2]),
            "Tuesday, December 1, 2026, 6:30 PM – 8:00 PM PST"
        );
        assert_eq!(time_range(&events[1]), "6:30 PM PST");
        assert_eq!(time_range(&events[2]), "6:30 PM – 8:00 PM PST");
        assert_eq!(machine_time(events[1].start), "2026-11-12T18:30:00-08:00");
    }
}
