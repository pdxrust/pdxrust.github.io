+++
title = "Implementing a Window Manager: developer workflows, C bindings, and Rust tooling"
start = 2026-02-12T19:00:00-08:00
kind = "talk"
status = "scheduled"

[location]
name = "Portland State University, Fourth Avenue Building"
address = "1900 SW 4th Ave, Portland, OR"

[[speakers]]
name = "Ben Massey"
+++

An artist might paint their parent a picture, a sculptor may craft them a vase,
but a programmer writes them an x11 window manager.

For Christmas of 2025, Ben Massey started on a project for his dad: an x11
window manager written in Rust. Development workflow is an important part of a
programmer's life: we use these computer things quite a bit every single day, so
basic usage of them should aid the creative process, rather than fight it. After
some discussions on the topic of workflow, Ben decided that there were some needs
not met by traditional window managers, and that it was worth taking a stab at a
solution.

This talk will go over the project "pick-full-wm", which takes full advantage of
the wonderful tooling offered to Rust like Cargo, and the niceties of aspects
like memory safety and compiler rule enforcement. This will also naturally lead
to topics of Rust bindings to C, and more general discussions of Linux and
software development.
