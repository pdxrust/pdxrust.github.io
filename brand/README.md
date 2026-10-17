# Brand assets

<!-- markdownlint-disable line-length -->

| File                                | Use                                                         |
| ----------------------------------- | ----------------------------------------------------------- |
| `pdxrust-avatar.svg`                | Square avatar on the site's dark background                 |
| `png/pdxrust-avatar-{32..1024}.png` | Discord server icon (512), Buttondown icon, social previews |
| `pdxrust-mark-dark-bg.svg`          | Transparent mark for dark backgrounds, such as the site     |
| `pdxrust-mark-light-bg.svg`         | Transparent mark for light backgrounds                      |

<!-- markdownlint-enable line-length -->

The lettering is drawn as paths, so the files don't depend on installed fonts.
The 32 px avatar is too detailed to read well, so a favicon needs a simpler
mark.

Regenerate the PNG files after changing an SVG:

```sh
for s in 32 64 128 256 512 1024; do
  rsvg-convert -w $s -h $s pdxrust-avatar.svg -o png/pdxrust-avatar-$s.png
done
rsvg-convert -w 512 pdxrust-mark-dark-bg.svg -o png/pdxrust-mark-dark-bg-512.png
rsvg-convert -w 512 pdxrust-mark-light-bg.svg -o png/pdxrust-mark-light-bg-512.png
```

## Attribution

The gear is from `logo/rust-logo-gear-only.svg` in
[rust-lang/rust-artwork](https://github.com/rust-lang/rust-artwork), © Rust
Foundation, licensed
[CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/). It was recolored and
the "PDX" lettering added. Rust and the Rust logo are trademarks of the Rust
Foundation. PDX Rust is a community meetup, not an official Rust Project group,
and this mark doesn't imply endorsement by the Rust Project.
