# PDX Rust website: project brief

Repo: `pdxrust/pdxrust.github.io`. Served at the org root via GitHub Pages. A
custom domain is planned later; do not hard-code the `github.io` hostname
anywhere.

## Purpose

Replace Meetup as the group's home. The site publishes the event schedule, event
history, an RSS feed, and an iCal feed. Events are irregular (talks, hack
nights, socials), not monthly. Members mostly learned of events by email, so the
site must make an email signup link prominent (external form, see Email below).

## Decisions already made (do not relitigate without a reason)

- **Static output only.** No server, no SSR, no database. Hosted on GitHub
  Pages.
- **Portability is a hard requirement.** It must be movable to Codeberg Pages,
  GitLab Pages, or any static host by changing CI config and DNS only.
  - The whole build is one plain command (e.g. `cargo run -p build-site` or a
    `just build`). CI is a thin wrapper that calls it.
  - No GitHub-only features in the build or contribution flow (no Discussions,
    issue forms, or Pages-only behavior). PRs are the contribution mechanism.
  - Events are plain files in the repo. One commit per event gives free history.
  - Set up a push mirror to Codeberg or GitLab later (not in the first phase).
- **Rust, with Leptos preferred** for rendering, since the maintainer already
  uses it (reference site is ../lukeworks). Keep complexity minimal. See Open
  question 1.
- **No third-party scripts, fonts, analytics, or CDNs.** The site must degrade
  gracefully with JavaScript disabled.
- **Out of scope:** comments, photo/video hosting, member database, accounts,
  email storage. The signup form is external.

## Open question 1: build approach (decide with the maintainer, do not pick silently)

Evaluate and report tradeoffs before writing any code. Verify current state
first; do not rely on memory.
- **A. Plain build binary (xtask-style)** that reads event files, renders HTML
  (Leptos `view!` rendered to a string, or another templating crate), and writes
  all outputs. Likely least complexity and easiest handoff.
- **B. Leptos static routes** (`leptos_router` `StaticRoute` plus
  cargo-leptos). Note: an older cargo-leptos issue called static support "very
  experimental". Check whether that is still true for the current Leptos
  release.
- **C. `cinnog`** (community Leptos SSG crate). Check maintenance status and
  compatibility with the current Leptos version.

Recommend one with evidence. The maintainer decides.

## Event data model

One file per event: `events/YYYY/YYYY-MM-DD-slug.md`, TOML front matter plus a
Markdown body.

Fields (parse into typed structs with serde; fail the build with a clear error
on any invalid file):
- `title`: string
- `start`, `end` (optional): RFC 3339 with offset; local zone is
  America/Los_Angeles
- `kind`: enum `talk | hack | social | other`
- `status`: enum `scheduled | cancelled`
- `location`: name plus address (optional), or `online`
- `speakers`: list of names, optional social links
- `links`: list (slides, repo, recording), optional
- body: Markdown description

The filename slug is the stable event ID (used for the page URL and the iCal
UID). Never change it after publishing. Needs to be able to handle rescheduled
events (same markdown content, but different dates)

Seed the past-events archive later from the old `pdxrust/meetups` repo (if
useful) or the still-active Meetup (<https://www.meetup.com/pdxrust/>), only
after the maintainer reviews it.

## Outputs

- `/` upcoming events, plus a prominent email signup link and the Discord invite
  link (permalink: <https://discord.gg/ZpvYBCP6xc>)
- `/past/` event history
- `/events/<slug>/` per-event page
- `/feed.xml` RSS 2.0 (confirm RSS vs Atom with the maintainer; default RSS 2.0,
  matching the stated goal. Might be worth supporting both).
- `/events.ics` iCalendar feed (stable UIDs, correct time zone handling,
  `STATUS:CANCELLED` for cancelled events)
- `/404.html`, `robots.txt`; a sitemap is optional

"Upcoming" vs "past" is computed at build time. A page can go stale after an
event passes, so CI must also rebuild on a daily schedule, not only on
push. Alternatively, some client-side JS is likely better than updating the code
daily.

## CI and contribution flow

- Build and validate on PRs and on pushes to the default branch; deploy only
  from the default branch.
- Scheduled daily rebuild (cron) to refresh upcoming/past. (pending the
  client-side JS decision)
- PR validation checks event files only: schema, dates, unique
  slugs. Contributors propose events by adding one file. Write a short
  `CONTRIBUTING.md` and an event template.
- Pin Actions by commit SHA. Do not run untrusted PR code with secrets (avoid
  `pull_request_target`).
- Optional later phase: on merge to the default branch, post new events to
  Discord via a webhook stored as a secret.

## Email

Phase 1: only a link to an external signup form. Candidate service is Buttondown
(free for the first 100 subscribers; the maintainer already decided on it as the
lean). Do not store emails in this repo or on any server. The maintainer will
create the account; ask for the signup URL. Whether Buttondown's RSS-to-email
works with our feed is unverified; confirm in their docs before relying on it.

Link is https://buttondown.com/pdxrust

## Security and quality (maintainer is a defense contractor engineer; correctness and security come first)

- `#![forbid(unsafe_code)]` in all crates; minimal dependencies; `cargo deny`
  and `cargo audit` in CI.
- Validate and escape all rendered content. Markdown from PRs is untrusted
  input: sanitize or disallow raw HTML.
- GitHub Pages cannot set response headers. If a CSP is wanted, use a `<meta>`
  CSP and note the limitation.
- Accessibility basics: semantic HTML, contrast, keyboard navigation, no JS
  requirement.
- Use the type system: enums for kind/status, newtypes for slugs, no
  stringly-typed dates.

## Suggested phases

1. Scaffold: repo layout, event schema and validation, one hand-written sample
event.
2. Render index, past, and event pages; minimal styling.
3. Generate `feed.xml` and `events.ics`; validate against public validators.
4. CI: build, validate, deploy to Pages, daily rebuild.
5. `CONTRIBUTING.md`, event template, README for future maintainers (this
project must be easy to hand over).
6. Later: custom domain, mirror to Codeberg/GitLab, Discord webhook, past-event
backfill.

## How to work with the maintainer

- Be concise. Do not explain Rust basics.
- Challenge flawed reasoning and raise known issues. Present options with
  evidence on tradeoffs, and let the maintainer decide.
- Research current API and tool details before relying on them. If unsure, say
  so instead of guessing.
- Ask when a request is ambiguous. Prefer targeted fixes over broad refactors.

## Not verified (check before relying on them)

- Current Leptos static-routing and cargo-leptos behavior; `cinnog` maintenance
  status.
- Codeberg Pages and Woodpecker CI specifics (for the later migration path).
- Buttondown RSS-to-email details and current pricing tiers.
