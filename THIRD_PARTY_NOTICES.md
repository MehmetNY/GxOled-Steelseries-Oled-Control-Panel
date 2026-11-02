# Third-party notices

GxOled Advanced is GPL-3.0 software built on [JerwuQu/ggoled](https://github.com/JerwuQu/ggoled) and, for the
animations, [pozitronik/steelclock-go](https://github.com/pozitronik/steelclock-go)
(see [`go_animations/UPSTREAM.md`](go_animations/UPSTREAM.md)). The modern settings interface (v0.5.3,
crate `gxoled_ui`) and the GIF Studio editor (v0.6.0, crate `gxoled_studio`) add the following material. All of it is under licences that permit use in a GPL-3.0 program.

## Embedded in the executable

| Component | Used for | Licence | Full text |
| --- | --- | --- | --- |
| **Inter** (Regular, Medium, SemiBold, Bold; © The Inter Project Authors, <https://github.com/rsms/inter>) | UI text; the font files are embedded and are not modified | SIL Open Font License 1.1 | [`gxoled_ui/licenses/INTER-OFL-1.1.txt`](gxoled_ui/licenses/INTER-OFL-1.1.txt) |
| **Lucide** icons (© Lucide Icons and Contributors, <https://lucide.dev>) | UI icons, inlined as SVG paths | ISC | [`gxoled_ui/licenses/LUCIDE-ISC.txt`](gxoled_ui/licenses/LUCIDE-ISC.txt) |
| [`resvg`](https://github.com/linebender/resvg), `usvg`, `svgtypes`, `roxmltree`, `simplecss`, `xmlwriter` | SVG parsing and rendering | Apache-2.0 OR MIT | each crate's `LICENSE-*` |
| [`tiny-skia`](https://github.com/linebender/tiny-skia), `tiny-skia-path` | Software rasteriser | BSD-3-Clause | each crate's `LICENSE` |
| `fontdb`, `harfrust` (text shaping), `skrifa`, `read-fonts`, `font-types`, `kurbo`, `unicode-bidi`, `unicode-script`, `unicode-vo`, `png`, `flate2`, `miniz_oxide`, `fdeflate`, `bytemuck`, `base64`, `data-url`, `imagesize`, `rgb`, `log`, `memchr`, `once_cell`, `smallvec`, `tinyvec`, `slotmap`, `strict-num`, `float-cmp`, `arrayref`, `arrayvec`, `siphasher`, `polycool`, `simd-adler32`, `adler2`, `crc32fast`, `bitflags`, `cfg-if`, `pico-args` | Supporting libraries of the renderer | MIT, Apache-2.0, BSD-2-Clause, Zlib, 0BSD, Unlicense or combinations of these | each crate's licence files |

The exact dependency list and licence expressions come from `Cargo.lock` / `cargo tree -p gxoled_ui`. Inter is
bundled unchanged and is not sold on its own, as the OFL requires; its copyright and licence text are kept in the
repository and in the font files' own metadata.

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

## Also linked (GIF Studio)

| Component | Used for | Licence |
| --- | --- | --- |
| [`gif`](https://github.com/image-rs/image-gif) | Writing exported GIFs (also read through `image`) | MIT OR Apache-2.0 |
| [`image`](https://github.com/image-rs/image) | GIF decoding and picture resizing | MIT OR Apache-2.0 |
| `serde`, `serde_json` | Settings and project files | MIT OR Apache-2.0 |
