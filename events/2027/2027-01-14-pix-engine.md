+++
title = "One Pixel at a Time: Seven Years of Graphics in Rust"
start = 2027-02-11T19:00:00-08:00
kind = "talk"
status = "scheduled"

[location]
name = "Portland State University, Fourth Avenue Building"
address = "1900 SW 4th Ave, Portland, OR"

[[speakers]]
name = "Luke Petherbridge"

[[links]]
kind = "repo"
url = "https://github.com/lukexor/pix-engine"
+++

pix-engine is a small Rust library for games, visualizations and generative
art. Luke Petherbridge will trace its history from drawing every pixel on the
CPU, to SDL2's hardware renderer, to a recent port in pure Rust using wgpu,
winit, egui, cpal and gilrs.

The project started in 2019, inspired by javidx9's olcPixelGameEngine and The
Coding Train's p5.js tutorials on YouTube. Back then, Rust graphics mostly meant
picking a crate that eventually called into a C or C++ library. Even "fast" Rust
can't make per-pixel drawing run at decent frame rates with a modest
resolution. Luke will cover what each rewrite bought, what challenges each
encountered, and how a simple drawing API rides on top of a modern GPU
pipeline. Expect live demos.
