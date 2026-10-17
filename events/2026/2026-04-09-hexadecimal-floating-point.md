+++
title = "Hexadecimal Floating Point? Why? How?"
start = 2026-04-09T19:00:00-07:00
kind = "talk"
status = "scheduled"

[location]
name = "Portland State University, Fourth Avenue Building"
address = "1900 SW 4th Ave, Portland, OR"

[[speakers]]
name = "Jim Blandy"
+++

Why in the world would anyone want to write a floating-point number in
hexadecimal? Is the '.' in such a number still a "decimal point", or is it a
"hexadecimal point"? What would that even mean??

As it turns out, hex floating point literals are, if not strictly necessary,
still pretty sweet when you get serious about numeric computation. The WebGPU
Shading Language (WGSL), a cross-platform language for writing code to run on
GPUs, supports hex floats, and as one of the maintainers of the Naga shader
translator crate, Jim Blandy has to support them.

Tonight Jim will narrate the bit-fiddling derring-do and uncalled-for
consistency and completeness he indulged in while reimplementing Naga's
hexadecimal floating-point literal parsing code. It'll be geeky and obscure. But
the truth is that floating-point arithmetic is actually much nicer and more
predictable than people think, and getting that nasty decimal conversion out of
the way makes that easier to see.
