# Third-party notices

GxOled Advanced is GPL-3.0 software built on [JerwuQu/ggoled](https://github.com/JerwuQu/ggoled) (GPL-3.0) and, for
the animations, [pozitronik/steelclock-go](https://github.com/pozitronik/steelclock-go) (GPL-3.0; see
[`go_animations/UPSTREAM.md`](go_animations/UPSTREAM.md)). The modern settings interface (crate `gxoled_ui`) and the
GIF Studio editor (crate `gxoled_studio`) add the material below. All of it is under licences that permit use in a
GPL-3.0 program.

This file is shipped next to the executable of every release. It names what the Windows executable and its embedded
animation helper contain and reproduces the licence texts that have to travel with a binary. The full texts of the
common Rust-library licences (MIT, Apache-2.0, BSD, Zlib, ...) are in each library's own repository; the exact
versions are pinned in `Cargo.lock`.

## Assets and tools embedded in the executable

| Component | Used for | Licence | Text |
| --- | --- | --- | --- |
| **Inter** — Regular, Medium, SemiBold, Bold; © The Inter Project Authors, <https://github.com/rsms/inter> | UI text. The embedded files are **subsets** (about 440 glyphs: Basic Latin, Latin-1, Latin Extended-A, dashes, quotation marks, bullet, ellipsis, four arrows, minus sign and a check mark) cut from the upstream fonts for English and Turkish text; they are not the complete font and are not sold on their own. | SIL Open Font License 1.1 | Appendix A |
| **Lucide** icons — © Lucide Icons and Contributors, <https://lucide.dev>; some derived from **Feather** (© Cole Bemis, MIT) | UI icons, inlined as SVG paths | ISC (Feather part: MIT) | Appendix B |
| **Pixel Operator** — Jayvee Enaguas (HarvettFox96), 2009-2018 | The bitmap font of the OLED texts (`gxoled_draw/fonts/PixelOperator.ttf`) | CC0 1.0 (public domain dedication, <https://creativecommons.org/licenses/zero/1.0/>) | stated in the font file's own metadata |
| **Go runtime and standard library** — © The Go Authors, <https://go.dev> | Linked into the embedded animation helper `go_animations/gxoled-animations.exe` (built with Go 1.23, no third-party Go modules besides the steelclock-go code in the repository) | BSD-3-Clause | Appendix C |
| **steelclock-go** widgets (Matrix Rain, Hyperspace, Star Wars Intro, Game of Life) — © pozitronik and contributors | The animation algorithms of the helper | GPL-3.0 | repository `LICENSE` |
| **ggoled** — © JerwuQu and contributors | The device protocol, the drawing core and the CLI this program grew from | GPL-3.0 | repository `LICENSE` |
| **Donation QR image** (`gxoled_app/assets/donation-usdt-tron.png`) | Shown on the About page | supplied by the project's maintainer | — |
| **GxOled logo and icons** (`assets/branding/`, `gxoled_app/assets/`) | Application, tray and window icons | original artwork of this project, drawn by `tools/make_branding.py`; part of this repository | — |

## Rust libraries compiled into the Windows executable

Generated for v0.6.2 with `cargo tree --target x86_64-pc-windows-msvc --workspace --features gxoled_app/sdl3-static -e normal,build --prefix none --format "{p}|{l}"`
(140 crates besides the six `gxoled_*` crates), grouped by the licence expression the crate declares. Every option of an
`OR` offer, and the Unicode data licence of `unicode-ident`, is compatible with GPL-3.0.

- **Apache-2.0 OR MIT** (93): `anstream`, `anstyle`, `anstyle-parse`, `anstyle-query`, `anstyle-wincon`, `anyhow`, `arrayvec`, `autocfg`, `base64`, `bit-vec`, `bit_field`, `bitflags`, `cc`, `cfg-if`, `chrono`, `clap`, `clap_builder`, `clap_derive`, `clap_lex`, `cmake`, `colorchoice`, `crc32fast`, `data-url`, `directories`, `dirs-sys`, `fdeflate`, `find-msvc-tools`, `flate2`, `font-types`, `gif`, `half`, `heck`, `image`, `image-webp`, `is_terminal_polyfill`, `itoa`, `kurbo`, `libc`, `lock_api`, `log`, `num-traits`, `once_cell`, `once_cell_polyfill`, `parking_lot`, `parking_lot_core`, `pkg-config`, `png`, `polycool`, `proc-macro2`, `qoi`, `quick-error`, `quote`, `read-fonts`, `resvg`, `roxmltree`, `rusttype`, `scopeguard`, `serde`, `serde_core`, `serde_derive`, `serde_json`, `serde_spanned`, `shlex`, `simplecss`, `siphasher`, `skrifa`, `smallvec`, `svgtypes`, `syn`, `toml`, `toml_datetime`, `toml_parser`, `toml_writer`, `ttf-parser`, `unicode-bidi`, `unicode-script`, `unicode-vo`, `usvg`, `utf8parse`, `version_check`, `weezl`, `windows`, `windows-collections`, `windows-core`, `windows-future`, `windows-implement`, `windows-interface`, `windows-link`, `windows-numerics`, `windows-result`, `windows-strings`, `windows-sys`, `windows-threading`
- **MIT** (18): `color_quant`, `fax`, `fax_derive`, `float-cmp`, `fontdb`, `harfrust`, `hidapi`, `imagesize`, `pico-args`, `rfd`, `rgb`, `simd-adler32`, `strict-num`, `strsim`, `tiff`, `winnow`, `xmlwriter`, `zmij`
- **Apache-2.0 OR MIT OR Zlib** (9): `bytemuck`, `bytemuck_derive`, `miniz_oxide`, `raw-window-handle`, `rpkg-config`, `tinyvec`, `zune-core`, `zune-inflate`, `zune-jpeg`
- **BSD-3-Clause** (4): `exr`, `lebe`, `tiny-skia`, `tiny-skia-path`
- **Apache-2.0** (3): `ab_glyph_rasterizer`, `owned_ttf_parser`, `spin_sleep`
- **Zlib** (3): `sdl3-src`, `sdl3-sys`, `slotmap`
- **Apache-2.0 OR BSD-2-Clause OR MIT** (2): `zerocopy`, `zerocopy-derive`
- **Apache-2.0 OR BSD-3-Clause** (2): `moxcms`, `pxfm`
- **MIT OR Unlicense** (2): `byteorder-lite`, `memchr`
- **(MIT OR Apache-2.0) AND Unicode-3.0** (1): `unicode-ident`
- **0BSD OR Apache-2.0 OR MIT** (1): `adler2`
- **BSD-2-Clause** (1): `arrayref`
- **MPL-2.0** (1): `option-ext`

Notes:

- `option-ext` (MPL-2.0) is used unmodified, through the `directories` crate; its source is at <https://crates.io/crates/option-ext>.
- `hidapi` (MIT) compiles the C library HIDAPI, which is offered under the GPL-3.0, a BSD licence or the original
  HIDAPI licence; GxOled uses it under the GPL-3.0.
- `sdl3-src` / `sdl3-sys` (Zlib) build SDL 3.4.2 into the executable (`sdl3-static`): tray icon, events and timers.
- `resvg` and its helper crates render the modern settings interface; `tiny-skia` is its rasteriser (BSD-3-Clause).
- `image` is built **without** its default features since v0.6.2: the AVIF encoder (`rav1e`, `ravif`) and a yanked
  crate it depended on (`core2`) are no longer part of the Windows build; the CLI still opens BMP, DDS, EXR, Farbfeld,
  GIF, HDR, ICO, JPEG, PNG, PNM, QOI, TGA, TIFF and WebP pictures.

## Derived from

**[GGIF-Studio](https://github.com/D4rkTT/GGIF-Studio)** — MIT licence, Copyright (c) 2026 Abdelnour Osman. The GIF
Studio editor (v0.6.0, crate `gxoled_studio`) is an independent Rust reimplementation of its processing model: the
option set and ranges (fit, rotate, mirror, zoom, shift, background fill, brightness, contrast, gamma, auto levels,
threshold, five dither modes, speed, minimum frame time, trim, loop) and the tone/dither algorithm of its `pixelate`
function (luminance weights, gamma/brightness/contrast order, Floyd–Steinberg, Atkinson and 4 × 4 Bayer dithering).
No JavaScript/TypeScript code is included and the program does not use Node.js, Electron or GameSense. The MIT licence
permits this and is compatible with GPL-3.0; as the MIT licence requires, its notice is reproduced here:

```
MIT License

Copyright (c) 2026 Abdelnour Osman

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

(The licence text above is the standard MIT text with GGIF-Studio's copyright line; it was checked against the
`LICENSE` file of the repository.)

## Appendix A — SIL Open Font License 1.1 (Inter)

```
Copyright (c) 2016-2023 The Inter Project Authors (https://github.com/rsms/inter)

This Font Software is licensed under the SIL Open Font License, Version 1.1.
This license is copied below, and is also available with a FAQ at: https://openfontlicense.org

-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font
creation efforts of academic and linguistic communities, and to
provide a free and open framework in which fonts may be shared and
improved in partnership with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any
reserved names are not used by derivative works. The fonts and
derivatives, however, cannot be released under any other type of
license. The requirement for fonts to remain under this license
does not apply to any document created using the fonts or their
derivatives.

DEFINITIONS
Font Software refers to the set of files released by the
Copyright Holder(s) under this license and clearly marked as such.
This may include source files, build scripts and documentation.

Reserved Font Name refers to any names specified as such after
the copyright statement(s).

Original Version refers to the collection of Font Software
components as distributed by the Copyright Holder(s).

Modified Version refers to any derivative made by adding to,
deleting, or substituting in part or in whole any of the components
of the Original Version, by changing formats or by porting the Font
Software to a new environment.

Author refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person
obtaining a copy of the Font Software, to use, study, copy, merge,
embed, modify, redistribute, and sell modified and unmodified copies
of the Font Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
   in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
   redistributed and/or sold with any software, provided that each
   copy contains the above copyright notice and this license. These
   can be included either as stand-alone text files, human-readable
   headers or in the appropriate machine-readable metadata fields
   within text or binary files as long as those fields can be easily
   viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
   Name(s) unless explicit written permission is granted by the
   corresponding Copyright Holder. This restriction only applies to
   the primary font name as presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the
   Font Software shall not be used to promote, endorse or advertise
   any Modified Version, except to acknowledge the contribution(s)
   of the Copyright Holder(s) and the Author(s) or with their
   explicit written permission.

5) The Font Software, modified or unmodified, in part or in whole,
   must be distributed entirely under this license, and must not be
   distributed under any other license. The requirement for fonts to
   remain under this license does not apply to any document created
   using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions
are not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED AS IS, WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL
THE COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR
CONSEQUENTIAL DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR
OTHERWISE, ARISING FROM, OUT OF THE USE OR INABILITY TO USE THE FONT
SOFTWARE OR FROM OTHER DEALINGS IN THE FONT SOFTWARE.
```

## Appendix B — Lucide (ISC) and Feather (MIT)

```
ISC License

Copyright (c) 2026 Lucide Icons and Contributors

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

---

The following Lucide icons are derived from the Feather project:

airplay, alert-circle, alert-octagon, alert-triangle, aperture, arrow-down-circle, arrow-down-left, arrow-down-right, arrow-down, arrow-left-circle, arrow-left, arrow-right-circle, arrow-right, arrow-up-circle, arrow-up-left, arrow-up-right, arrow-up, at-sign, calendar, cast, check, chevron-down, chevron-left, chevron-right, chevron-up, chevrons-down, chevrons-left, chevrons-right, chevrons-up, circle, clipboard, clock, code, columns, command, compass, corner-down-left, corner-down-right, corner-left-down, corner-left-up, corner-right-down, corner-right-up, corner-up-left, corner-up-right, crosshair, database, divide-circle, divide-square, dollar-sign, download, external-link, feather, frown, hash, headphones, help-circle, info, italic, key, layout, life-buoy, link-2, link, loader, lock, log-in, log-out, maximize, meh, minimize, minimize-2, minus-circle, minus-square, minus, monitor, moon, more-horizontal, more-vertical, move, music, navigation-2, navigation, octagon, pause-circle, percent, plus-circle, plus-square, plus, power, radio, rss, search, server, share, shopping-bag, sidebar, smartphone, smile, square, table-2, tablet, target, terminal, trash-2, trash, triangle, tv, type, upload, x-circle, x-octagon, x-square, x, zoom-in, zoom-out

The MIT License (MIT) (for the icons listed above)

Copyright (c) 2013-present Cole Bemis

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Appendix C — Go (BSD-3-Clause)

```
Copyright 2009 The Go Authors.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

   * Redistributions of source code must retain the above copyright
notice, this list of conditions and the following disclaimer.
   * Redistributions in binary form must reproduce the above
copyright notice, this list of conditions and the following disclaimer
in the documentation and/or other materials provided with the
distribution.
   * Neither the name of Google LLC nor the names of its
contributors may be used to endorse or promote products derived from
this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```
