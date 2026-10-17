# Contributing

## Proposing an event

Every event is one Markdown file. To propose one, open a pull request that adds
a single file.

1. Copy [`templates/event.md`](templates/event.md) into the `events/` folder
   for the event's year, named with the date and a short name. For a December
   10, 2026 talk on async closures, that's 2026-12-10-async-closures.md in
   `events/2026/`.
2. Fill in the fields. The comments in the template explain each one, and you
   can delete them.
3. Write a short description below the closing `+++`. The first paragraph
   appears on the home page.
4. Open a pull request. CI checks the file and lists every problem it finds.

The file name rules are lowercase letters, digits and single hyphens after the
date. The year folder must match the year in the file name.

To check the file locally, install Rust and [just](https://just.systems), then
run:

```sh
just check   # validate site.toml and every event file
just build   # build the site into dist/
just serve   # view it at http://localhost:8000
```

## Changing an event

- **Rescheduled:** change `start` and `end`. Keep the file name, even though its
  date no longer matches. The file name is the event's permanent ID, used in its
  URL and in calendar apps, so renaming it breaks links and duplicates the event
  in subscribers' calendars.
- **Cancelled:** set `status = "cancelled"`. Keep the file. The event shows a
  "Cancelled" label and calendar apps mark it cancelled.
- **After the event:** add slides, a repository or a recording under
  `[[links]]`.

## Times

Write times in Portland local time with the UTC offset: `-08:00` from early
November to mid March (PST), `-07:00` the rest of the year (PDT). For example,
`2026-12-10T19:00:00-08:00` is 7 PM on December 10. The site shows every time in
Portland time.

## Descriptions

Descriptions are Markdown. Raw HTML isn't rendered: it appears as plain text.
Links must be `http://` or `https://`.

## Code changes

Run `just lint` and `just test` before opening a pull request. The
[README](README.md) describes how the site is built.
