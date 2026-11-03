# GIF Studio (v0.6.0)

The **GIF Studio** page of the modern settings window turns any GIF into a 1-bit animation for the 128 × 64 OLED, lets
you tune how it looks while you watch the result, plays it on the base station, and exports it again as a GIF.

It is a Rust reimplementation of the processing model of [GGIF-Studio](https://github.com/D4rkTT/GGIF-Studio)
(MIT, © 2026 Abdelnour Osman; see `THIRD_PARTY_NOTICES.md`). No Node.js, Electron or GameSense is involved; Rust stays
the only OLED writer.

## Using it

1. **Add GIF…** (or *Open project…*). The page shows the file name, size, frame count and duration.
2. Two previews animate in the page: the **source** picture and the **OLED result** with your settings. They start
   by themselves when a GIF is opened; the **Pause / Play** button in the header of the *Preview* card freezes and
   resumes them. They only run while the window is open and this page is selected: closing the window or choosing
   another page stops them (nothing is rendered), and they continue from the same frame when you come back, unless
   you paused them. The panel on the right (*Live OLED*) still shows only what the base station really shows: the GIF
   appears there when you press **Start / Stop GIF** and disappears when you stop it. Pause / Play does not touch
   *Start / Stop GIF*.
3. Change settings. If the GIF is playing on the OLED it is re-rendered a moment after you stop moving a slider
   (the playback restarts from its first frame).
4. **Save project…** keeps the settings, **Export GIF…** writes the finished two-colour animation.

Settings are stored in `config.toml` (`[animation_settings]`) and apply to the GIF you start next, also from the tray
menu or the classic window.

> The preview **Pause / Play** button and the memory rules under *Limits* were added after v0.6.0 (see `CHANGELOG.md`,
> "Unreleased").

## Settings

| Group | Control | Range / values | Default |
| --- | --- | --- | --- |
| Picture | Fit | Contain · Cover · Stretch | Contain |
| | Rotate | 0° · 90° · 180° · 270° (clockwise) | 0° |
| | Mirror | horizontal, vertical (applied after the rotation, as seen on the screen) | off |
| | Zoom | 0.20× – 3.00× on top of *Fit* | 1.00× |
| | Horizontal / vertical shift | −64 … +64 screen pixels | 0 |
| Tone & dither | Transparent areas | black or white background | black |
| | Invert source | inverts the picture before everything else (the background is not inverted) | off |
| | Auto levels | stretches each frame's darkest–brightest range to 0–255 | off |
| | Brightness / contrast | −100 … +100 | 0 |
| | Gamma | 0.10 – 3.00 | 1.00 |
| | Threshold | 0 – 255 | 128 |
| | Dither | Floyd–Steinberg · Atkinson · Ordered (Bayer 4×4) · Threshold only · None | Floyd–Steinberg |
| | Invert result | swaps lit and dark pixels at the end | off |
| Playback | Loop | repeat, or play once and hold the last frame | on |
| | Speed | 0.10× – 5.00× | 1.00× |
| | Minimum frame time | 10 – 200 ms | 30 ms |
| | First / last frame | trims the animation (inclusive) | whole GIF |

*Threshold only* gives the crisp black/white look of earlier GxOled versions; *None* always cuts at the middle (128)
and ignores the threshold slider, as GGIF does.

## Processing order

For every source frame (RGBA, transparent pixels allowed):

1. Luminance `0.299 R + 0.587 G + 0.114 B`, *Invert source*, transparent pixels replaced by the background colour.
2. Placement on the 128 × 64 screen: rotate, mirror, *Fit* × *Zoom*, shift. Areas outside the picture are background.
   Big pictures are averaged (up to 4 × 4 sub-samples per screen pixel) instead of point-sampled, so thin lines do not alias.
3. *Auto levels* → brightness / gamma / contrast: `v = 255·(g/255)^(1/gamma) + brightness·2.55`, then
   `(v − 128)·(1 + c)/(1 − c) + 128`, clamped to 0–255.
4. Dither / threshold → *Invert result* → 1 bit per pixel.

Timing: `delay = max(source delay ÷ speed, minimum frame time)`. GIF delays below 20 ms are read as 100 ms, like
browsers do. The minimum frame time is a send-rate cap for the USB link, so it is applied after the speed.

## Project files

`*.gxgif.json` — format `gxoled-gif-project`, version 1: the path of the GIF and all settings. The picture is **not**
embedded, so moving the GIF breaks the project. Unknown fields fall back to defaults and every value is clamped on load;
files of other programs (including GGIF's own `ggif-project`) are refused with a clear message.

## Limits

GIF file ≤ 64 MiB, ≤ 2048 × 2048 px, ≤ 2000 frames, ≤ 64 Mpx in total. Pictures larger than 480 px are shrunk while
loading (the screen is 128 px wide; this leaves room for zoom). The decoded pictures are the largest memory item of the
application (`width × height × 4` bytes per frame: about 40 MiB for 60 frames of 480 × 360, 88 MiB for 100 frames of 480 × 480), so they
are kept only while they are used: a GIF that is not being edited or re-rendered is dropped 10 s after its last use,
and at once when the settings window is closed. It is decoded again from the file when needed; the frames that play on
the OLED (1 kB each) are not affected.

## What is not in this version

GGIF-Studio is a multi-layer editor; this first integration is a single-GIF editor. Not included yet: several
layers, text layers, a draggable timeline, undo/redo, video files (MP4/WebM would need a decoder such as ffmpeg).

## Where the code is

| Part | Location |
| --- | --- |
| Engine (no OS, no window): settings, decoding, geometry, tone/dither, playback plan, GIF export, project file | crate `gxoled_studio` |
| Page, sliders, previews and the settings ↔ controls mapping | `gxoled_ui` (`pages.rs`, `studio_bind.rs`) |
| Loading, throttled re-render, dialogs, config | `gxoled_app/src/main.rs`, `modern_ui.rs` |
| Play-once animation layer | `gxoled_draw` (`DrawLayer::Animation { repeat }`) |

## Verification status (be precise)

Unit-tested without a device: every dither mode, tone formula, rotation / mirror / fit / zoom / shift on marker pictures,
background and inversion rules, playback plan, GIF export (decoded back pixel for pixel), project file round trip,
the control ↔ setting mapping of every chip and slider, config round trip, slider dragging in the UI, memory release.
Compiled for Windows and Linux in CI.

**Not verified on a physical PC / base station:** how the page behaves on a real desktop (file dialogs, DPI changes,
repaint cost of the two animated previews), and how each dither mode looks on the real OLED.
