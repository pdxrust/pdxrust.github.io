+++
# Copy this file to events/YYYY/YYYY-MM-DD-short-name.md, using the event's date.
# The file name is the event's permanent ID. Don't rename it after it's published.

title = "Your talk title"

# Date, time and UTC offset. Portland is -08:00 in winter (PST), -07:00 in summer (PDT).
start = 2026-12-10T19:00:00-08:00
# Optional. Without it, the event is treated as two hours long.
end = 2026-12-10T21:00:00-08:00

# talk, hack, social or other
kind = "talk"
# scheduled or cancelled
status = "scheduled"

# Either location = "online", or a venue with an optional address:
[location]
name = "Portland State University, Fourth Avenue Building"
address = "1900 SW 4th Ave, Portland, OR"

# Optional, one block per speaker. The url is optional too.
[[speakers]]
name = "Your Name"
url = "https://example.com/"

# Optional, one block per link. kind is slides, repo, recording or other.
[[links]]
kind = "slides"
url = "https://example.com/slides"
+++

One or two sentences about the talk. This first paragraph appears on the home
page, so make it stand alone.

Anything after it shows only on the event page. Markdown works: **bold**,
_italics_, `code`, lists and [links](https://www.rust-lang.org/). Raw HTML is
shown as plain text.
