# PDX Rust website

The website of PDX Rust, Portland's Rust meetup: upcoming events, past events,
RSS and Atom feeds, and an iCalendar feed.

To propose an event, see [CONTRIBUTING.md](CONTRIBUTING.md).

## How it works

A Rust program in `src/` reads `site.toml` and `events/`, then writes a static
site to `dist/`. There is no server, database or JavaScript. Pages are rendered
with [Leptos](https://leptos.dev) `view!` templates to plain HTML at build time.

<!-- markdownlint-disable line-length -->

| Path                   | Contents                                                       |
| ---------------------- | -------------------------------------------------------------- |
| `site.toml`            | Site title, headline, description, and signup and social links |
| `events/YYYY/*.md`     | One file per event: TOML front matter, then Markdown           |
| `templates/event.md`   | Starting point for a new event                                 |
| `static/`              | Stylesheet, logo, icons and skyline, copied to `dist/` as-is   |
| `brand/`               | Logo sources and PNG exports for Discord and Buttondown        |
| `src/load.rs`          | Reads and validates `site.toml` and the event files            |
| `src/event.rs`         | Event types                                                    |
| `src/markdown.rs`      | Markdown to sanitized HTML                                     |
| `src/render.rs`        | HTML pages                                                     |
| `src/feeds.rs`         | `/feed.xml` (RSS 2.0) and `/atom.xml`                          |
| `src/ical.rs`          | `/events.ics` and one `event.ics` per event                    |
| `src/history.rs`       | Feed entry dates from `git log`                                |
| `.github/workflows/`   | CI and deployment                                              |

<!-- markdownlint-enable line-length -->

## Commands

Install a Rust toolchain (`rust-toolchain.toml` pins the version) and
[just](https://just.systems). `just lint` also needs
[typos](https://github.com/crate-ci/typos), and `just audit` needs
[cargo-deny](https://github.com/EmbarkStudios/cargo-deny).

```sh
just check   # validate site.toml and events/
just build   # build into dist/ for http://localhost:8000
just serve   # serve dist/ at http://localhost:8000
just test    # unit tests
just lint    # rustfmt, clippy and typos
just audit   # cargo deny: advisories, licenses, sources
```

`just build URL` builds for a public address. Feeds and calendars need absolute
links, and the address isn't stored in the repository.

## Deployment

`.github/workflows/site.yml` runs `just check`, `lint`, `test` and `build` on
every push and pull request, and `just audit` in a separate job. On `main` it
builds with the address from the Pages settings and deploys to GitHub Pages. A
daily scheduled run rebuilds the site so that ended events move from the home
page to the past page.

GitHub disables scheduled workflows after 60 days without repository activity.
If the home page stops updating, re-enable the workflow under Actions.

CI needs the full git history (`fetch-depth: 0`). Feed entries are dated by
the commit that added each event file, and the build fails on a shallow clone.

### Moving to another host

The build is `just build https://new.example`, and the output is `dist/`. Any
static host works. To move:

1. Write a CI job for the new host that runs the same `just` recipes with full
   git history, then publishes `dist/`.
2. Point DNS at the new host.
3. Serve `dist/404.html` for missing pages if the host supports it.

Event URLs and calendar UIDs don't contain the domain, so subscribers keep their
events. Feed readers see entries again once, because feed entry IDs are URLs.

## Security

- Event descriptions come from pull requests, so they're untrusted. Raw HTML in
  Markdown becomes text, and [ammonia](https://docs.rs/ammonia) sanitizes the
  rendered HTML. Every other value is escaped by Leptos or by the feed writers.
- GitHub Pages can't set response headers, so the Content-Security-Policy is a
  `<meta>` tag. Browsers ignore `frame-ancestors` in a `<meta>` policy, so the
  site can still be framed.
- All crates use `#![forbid(unsafe_code)]` through `[lints.rust]` in
  `Cargo.toml`.
- Actions are pinned by commit SHA, and Dependabot proposes updates weekly. Pull
  requests run with a read-only token and no secrets.

## Logo and trademarks

The logo's gear is adapted from the Rust logo in
[rust-lang/rust-artwork](https://github.com/rust-lang/rust-artwork), © Rust
Foundation, [CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/). Rust
and the Rust logo are trademarks of the Rust Foundation. PDX Rust is a community
meetup, not an official Rust Project group. Keep the attribution and the "not
official" line in the footer. See [brand/README.md](brand/README.md).
