# Modern settings interface (v0.5.3)

Since v0.5.3 the settings windows have a modern look (sidebar, cards, a live 128 × 64 OLED mirror). It replaces the
*look* of the windows, not the application: Rust is still the only OLED writer, the control ids and `MenuEvent`s are
the classic ones, and profile files, config keys and GIF handling are shared. The **classic window is still inside the
same EXE** and can be chosen at any time.

## Switching between the two windows

Any one of these is enough; none needs a re-install.

| How | Effect |
| --- | --- |
| Tray icon → **Settings window style** → **Modern** / **Classic** | Switches at once and opens the window of that style. Saved. |
| Modern window → **General → Settings window** → **Modern** / **Classic** | Same, from inside the new window. |
| Classic window → **Modern UI** button (left rail, above *Quit*, every page) | Back to the modern window. |
| Start `gxoled.exe --classic-ui` / `--modern-ui` | Forces that style and saves it. |
| Edit `%APPDATA%\GxOled\config.toml`: `ui_style = "Classic"` or `"Modern"` | Same, offline. |
| Install **v0.5.2** | Release without any of this code; the extra `ui_style` key is ignored. |

**Türkçe özet:** Arayüzler arasında geçiş: tepsi simgesi → *Ayar penceresi biçimi* → *Modern* / *Klasik*; modern
pencerede *Genel → Ayar penceresi*; klasik pencerede sol menüdeki *Modern UI* düğmesi; ya da `gxoled.exe --classic-ui` /
`--modern-ui`. Klasik pencere aynı EXE'nin içinde değişmeden durur.

Automatic fall-back (saved as Classic): the modern window cannot be created · a panic inside its window procedure ·
the previous modern start never painted its first frame (marker file `%APPDATA%\GxOled\.modern-ui-pending`).

## Behaviour worth knowing

* **Resizing zooms, it does not re-flow.** The settings window is always laid out for a 1450-unit-wide design and
  the whole picture is scaled to the window width (no lower or upper limit; the window opens at 1015 × 682 and the minimum size is 800 × 538).
  Every card, icon and caption keeps its place and proportions; a window that is too short for a page scrolls
  vertically. (`gxoled_ui::Ui::scale_for`, covered by `layout_is_identical_at_every_window_size`.)
* **The OLED mirror shows the real screen.** The render thread hands every frame it composes (clock, GIF, media text,
  notification, OLED pixel shift, Go animation) to the window. The panel is titled *Live OLED* while those frames
  are real, so a GIF appears when it is started and disappears when it is stopped, exactly as on the device. When no
  device frame exists (base station disconnected, or showing its own volume screen) it shows what the configured GIF
  or clock would look like and the panel is titled *OLED preview*.
* **Nothing runs for a window nobody sees.** The mirror, the editor previews and the pixel buffers exist only while a
  settings window is visible: when it is closed the buffers are freed (they are rebuilt in about 15 ms when it is shown
  again), the previews stop, a decoded GIF is dropped and the main loop falls back from a 10 ms to a 50 ms rhythm (it
  still wakes exactly at each full second for the OLED clock). On Windows the working set is trimmed afterwards. See
  the CHANGELOG entry "lower resource use" for the figures and for what is *not* measured.
* **General → Settings Summary** lists the choices made on that page (opening mode, language, appearance, start with
  Windows, what happens when GxOled closes). It is read-only; the controls are the cards on the left.

## What is where

| Part | Location | OS dependency |
| --- | --- | --- |
| UI toolkit: model, layout, SVG scene, interaction state machine, `resvg` rendering | `gxoled_ui/` | none (runs and is tested on Linux) |
| Win32 host: window class, input, DPI, DWM title-bar colours, native text control of the editor | `gxoled_app/src/modern_window.rs` | Windows (`windows-sys`) |
| Bridge: control ids → `MenuEvent`, profile list/editor logic, OLED mirror, crash guard | `gxoled_app/src/modern_ui.rs` | Windows |
| Style switch and fall-back | `gxoled_app/src/settings_ui.rs` | Windows |
| Frame tap, shared clock composition, device-free GIF preview | `gxoled_draw/src/lib.rs` (`DrawDevice::latest_frame`, `clock_screen`, `GifPreview`) | none |
| Classic window | `gxoled_app/src/settings_window.rs` | Windows |

Rendering: each window is described as SVG (generated in Rust), rasterised by `resvg` with embedded Inter fonts and
inlined Lucide icons, and blitted with `StretchDIBits`. The live OLED mirror and the QR code are raster overlays
blitted nearest-neighbour. Repaints are event-driven; the OLED mirror updates only its own rectangle.

## Verification status

| Level | What |
| --- | --- |
| Unit-tested | `gxoled_ui`: layout (including identical layout at every window size, both languages), hit testing, keyboard/mouse/scroll/slider/profile-list behaviour, editor surface. `gxoled_draw`: every clock style renders, styles differ, the clock follows the time and the burn-in offset, the GIF preview timing. `gxoled_app`: control-id → event table, config snapshot, volume-time choices, version label. All run in Linux and Windows CI. |
| Rendered and reviewed | All pages and the editor, English and Turkish, dark and light, at several sizes/DPIs (PNG review on Linux). |
| Compiled on Windows | Windows MSVC `cargo check`, `cargo test`, `cargo build` in CI for the commit named in the release notes. |
| **Not verified on a physical PC/base station** | Window creation/behaviour on a real desktop, DPI changes between monitors, DWM caption colours (Windows 11), the native text control of the editor, modal file dialogs, repaint cost while hovering, the live OLED mirror against a real base station (the frame tap is plain code but only runs with a device). |

## Not in the modern interface (yet)

* A custom (frameless) title bar — the native frame is used and coloured through DWM.
* The GIF editor inspired by GGIF-Studio; its licence has to be checked against GPL-3.0 first.

## Third-party material

See [`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md).

## GIF Studio page (v0.6.0)

The *GIF Studio* page contains the GIF editor described in [`GIF-STUDIO.md`](GIF-STUDIO.md). Its two previews (source and OLED result) are separate raster regions that are repainted on their own (`Ui::live_rects_px`), like the *Live OLED* mirror, so animating them does not re-render the page. The classic window has no editor; its GIF page still adds and starts/stops GIFs and says where the editor is.
