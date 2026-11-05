# GIF Studio

*Added in v0.6.0 · preview Pause / Play and the memory rules in v0.6.1 · GIFs play at their real speed on the OLED in v0.6.2.*

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

*Threshold only* gives the crisp black/white look of earlier GxOled versions and is the only dither mode that follows the
**Threshold** slider. *None* always cuts at the middle (128), as GGIF does, and *Ordered (Bayer 4×4)* compares every pixel
with its own matrix cell — both ignore the slider.

## Processing order

For every source frame (RGBA, transparent pixels allowed):

1. Luminance `0.299 R + 0.587 G + 0.114 B`, *Invert source*, transparent pixels replaced by the background colour.
2. Placement on the 128 × 64 screen: rotate, mirror, *Fit* × *Zoom*, shift. Areas outside the picture are background.
   Big pictures are averaged (up to 4 × 4 sub-samples per screen pixel) instead of point-sampled, which softens aliasing
   a lot; a source that is more than 4 × larger than the screen in one direction (the working size is 480 px, the
   screen 128 × 64) is still undersampled, so a one-pixel line can flicker. Known limitation, see `KNOWN-ISSUES.md`.
3. *Auto levels* → brightness / gamma / contrast: `v = 255·(g/255)^(1/gamma) + brightness·2.55`, then
   `(v − 128)·(1 + c)/(1 − c) + 128`, clamped to 0–255.
4. Dither / threshold → *Invert result* → 1 bit per pixel.

Timing: `delay = max(source delay ÷ speed, minimum frame time)`. GIF delays below 20 ms are read as 100 ms, like
browsers do. The minimum frame time is a send-rate cap for the USB link, so it is applied after the speed.

On the OLED the render loop ticks every 33 ms and shows **at most one frame per tick**. Since v0.6.2 a frame is due one
delay after the *previous frame was due* (not after the tick that noticed it), so a 40 ms (25 fps) GIF plays in about
40 ms per frame, as in the preview and in the exported file. Before v0.6.2 every delay was rounded up to whole ticks,
which made 20 and 25 fps GIFs play 35–68 % slower on the OLED. Frames shorter than one tick (30 ms and below) are still
limited to one per tick, and after a stall (volume screen, USB hiccup) playback continues at the normal pace instead of
bursting through the frames it missed.

## Project files

`*.gxgif.json` — format `gxoled-gif-project`, version 1: the path of the GIF and all settings. The picture is **not**
embedded, so moving the GIF breaks the project. Unknown fields fall back to defaults and every value is clamped on load
(a number outside its range, such as `threshold = 300`, is pulled back to the nearest valid value instead of discarding
the file); files of other programs (including GGIF's own `ggif-project`) are refused with a clear message.

Since v0.6.2 a project cannot name a GIF by network path (`\\server\share\…`, `//server/share/…`, `\\?\UNC\…`) or device
path (`\\.\…`): opening a network path can make Windows try to sign in to that computer (not tested here), and a project
received from someone else should not be able to trigger that. Drive letters (mapped network drives included) and the long local form `\\?\C:\…` are fine. Projects
saved from the file dialogs hold absolute paths; a hand-written relative path is resolved against the folder the
application was started from, not against the project file.

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

Other known limits of the editor (details in [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)): the *OLED preview* of the live
panel (shown when no base station is connected) loops a GIF that is set to play once; very large sources can
undersample thin lines (see *Processing order*); GIFs larger than 480 px are shrunk on straight (not premultiplied)
alpha, so the edge of a transparent shape can pick up a faint halo from the colour hidden under the transparent pixels.

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
the control ↔ setting mapping of every chip and slider, config round trip, slider dragging in the UI, memory release,
repair of hand-edited numbers (`threshold = 300`, `rotate = -90`), refusal of network / device paths in project files, and
(v0.6.2) the frame scheduling rule of the OLED animation layer — `next_due` and the play-once / looping state machine
are tested against simulated clocks.
Compiled for Windows and Linux in CI.

**Not verified on a physical PC / base station:** how the page behaves on a real desktop (file dialogs, DPI changes,
repaint cost of the two animated previews), how each dither mode looks on the real OLED, and the real playback speed on
the OLED: the 35–68 % figure for the earlier behaviour and the new frame times come from a model of the render loop and
from the unit tests, not from a stopwatch on a base station.
