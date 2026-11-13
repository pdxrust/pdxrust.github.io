+++
title = "The Sync/Async Sandwich: The Dark Side of Async Traits"
start = 2027-09-09T19:00:00-07:00
kind = "talk"
status = "scheduled"

[location]
name = "Portland State University, Fourth Avenue Building"
address = "1900 SW 4th Ave, Portland, OR"

[[speakers]]
name = "Luke Petherbridge"
+++

Async Rust seems straight-forward until sync code calls async code that calls
sync code again. Luke Petherbridge will share lessons from a production codebase
on async traits and the sync/async sandwich.

He'll cover native async functions in traits versus the `async-trait` crate,
`Send` bounds that leak through every layer, blocking calls inside a runtime,
and the patterns that kept the boundary between sync and async code in one
place.
