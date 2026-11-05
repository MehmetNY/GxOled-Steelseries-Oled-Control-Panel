# Known issues and limits

*State of v0.6.2.* This page lists what the v0.6.2 audit found and did **not** fix, plus limits a user can run into.
What *was* fixed is in [`../CHANGELOG.md`](../CHANGELOG.md). Nothing here was tested on a real PC with a base station:
see [Not verified](#not-verified-on-real-hardware) at the end.

**Türkçe özet:** v0.6.2 öncesindeki derin denetimde bulunan sorunların büyük kısmı giderildi (ayrıntı: `CHANGELOG.md`).
Bu sayfa **giderilmeyenleri** ve kullanıcıyı etkileyebilecek sınırları listeler. En önemlileri: özel Go animasyon
profillerinde aşırı değerler yardımcı programı çökertebilir (artık fark edilip kapatılıyor, ama doğrulayıcı değerleri
hâlâ denetlemiyor); klasik pencere yüksek DPI'da küçük kalır (varsayılan modern pencerede sorun yok); Casio 12 saat
kipinde AM/PM işareti yok; GIF Studio çok büyük kaynaklarda ince çizgileri örnekleme hatasıyla bozabilir. Hiçbir şey
gerçek bir baz istasyonunda denenmedi.

## How this list was made

Six independent code reviews (CI / packaging / the Go renderer · the Win32 windows · the main loop · the drawing, HID,
CLI and GIF engines · app core, config, start-up, tray · the UI toolkit and the documentation) read the sources and the
docs; every finding was then checked against the code before it was fixed or listed here. These are **reviews plus Linux
unit tests plus a Windows type-check**, not runs on a Windows desktop. A finding can therefore be wrong about hardware
or Windows behaviour; where the reviewer was unsure the entry says so.

*User-visible* means a normal user can meet it. *Edge* needs hand-edited files or an unusual set-up. *Internal* is code
quality only.

## User-visible

- **Custom Go animation profiles with extreme values can crash, exhaust memory or spin the helper.** Examples (found on a
  Linux build of the same source): `position.h = 0` or `min_length > max_length` in *Matrix*, `position.w = 0` or
  `stars.brightness = 1` in *Star Wars*, a negative `position.w` or a size of 100000 in *Game of Life*, a count or
  `star_count` of two billion, `trail_length = 1e9` in *Hyperspace*. v0.6.2 notices a helper that **exits** (restarts it
  at most every 2 s; after three quick stops it returns to the clock and shows the helper's first error line). A helper that
  stays **alive but silent** (a CPU spin, a huge allocation) is not detected, and the profile checker
  (`valid_profile_text`, `gxoled_app/src/settings_window.rs`) still checks only widget type, size and delay.
  *Workaround:* keep custom values close to the shipped profiles. *Fix belongs in* `go_animations/cmd/renderer/main.go`
  (bounds checks and a `recover`; the upstream packages must stay byte-identical).
- **Imported profiles are stored pretty-printed.** The 64 KiB limit is checked on the text as imported; a minified
  ~48 KB profile can grow past 64 KiB when stored and is then refused by the helper (it shows up as the helper message above).
- **The classic settings window is not DPI-aware.** It is a fixed 1050 × 710 px window with an 18 px font; at 150 – 200 %
  scaling it looks small (`WM_DPICHANGED` is ignored on purpose). The modern window — the default — scales correctly.
- **Profile files need ASCII names.** Only `[A-Za-z0-9._-]`, at most 120 characters, are listed (`settings_window.rs`
  ~345). A hand-placed `Benim profilim.json` in the profile folder is silently ignored.
- **Casio clock, 12-hour mode: no AM/PM marker** (`gxoled_draw/src/casio.rs`), so `01:05` is ambiguous.
- **Command-line and config switches.** `--classic-ui` / `--modern-ui` are read after the single-instance check, so on an
  already running GxOled they are ignored (use the tray menu). A hand-written `[font]` table in `config.toml` needs both
  `path` and `size`; otherwise the file cannot be read and is copied to `config.toml.bad` (v0.6.2) before defaults are used.
- **Environment checks overwrite saved settings.** When the OS reports that it cannot supply media information or idle
  time (GNOME/X11 without D-Bus, a failed WinRT request), *Show media* / *Screensaver when idle* are set to off and the next
  save keeps them off (`main.rs`, tray set-up).
- **Start with Windows.** A start-up entry stored as `REG_EXPAND_SZ` is read as "off". An entry disabled in Task Manager
  (`…\Explorer\StartupApproved\Run`) still shows as "on" and GxOled cannot re-enable it. A registry error from the
  settings window is swallowed (the tray menu shows a dialog). v0.6.2 repoints an entry whose program no longer exists.
- **Media information.** On Linux and macOS a track without an artist (podcasts, many browser sources) is not shown;
  Windows shows it. Windows asks for the *current* media session only (`GetCurrentSession()`), so a paused browser session
  may hide a playing player — not verified. The start-up call into the media service has no time-out (a stalled service
  would delay the tray icon).
- **Wayland.** Requests created after start-up are never flushed, and one transient error switches idle tracking off for the
  session while the tray check box stays enabled (`gxoled_app/src/os/linux/`).
- **Window loops stall the main loop.** While a title-bar drag, a message box, a file dialog or a tray pop-up is open, the
  clock, media and hot-plug handling wait. There is no `WM_QUERYENDSESSION` / `WM_ENDSESSION` handling, so log-off or
  shut-down skips the *when GxOled closes* action and the last frame may stay on the OLED (low confidence).

## Classic settings window (Windows)

- **A fast double-click on a button acts twice** (`BN_DOUBLECLICKED` is not filtered): *Delete* removes the profile and then
  reports that built-in profiles cannot be deleted, *Add GIF* / *Import* can open two dialogs, *Start / Stop GIF* toggles back.
- **Profile lists are loaded when a window is created** and refreshed only after that window's own actions. Delete or import in
  one window, switch the style, and the other window can still list the deleted file; *Use selected* would then start a missing
  profile and *Edit → Save* would recreate it from the default template.
- **Dialogs are not modal** (`MessageBoxW(NULL, …)`, file dialogs without an owner): a second click nests another dialog.
- **Light theme:** labels, radio buttons and check boxes get grey button-face boxes on the white panels.
- **Mixed English / Turkish.** The quit, screen-shift, burn-in and status-icon radios, the OLED panel captions, "Light" / "Dark"
  and the validation errors are English-only; two captions point to *General → Settings window*, which exists only in the modern
  window; the text of the creation dialog ("Future editing tools will live here") is stale; ids 711 / 715 / 717 are dead.
- **AltGr counts as Ctrl** in the shortcuts of both windows (Turkish-Q: AltGr + A / S selects all / saves, then the posted
  character replaces the selection with "æ" / inserts "ß").
- **Selection jumps** to the first row after *Delete* or after saving a built-in profile; a double-click in the list does
  not mean *Use selected* (it does in the modern window).
- **Check boxes can disagree with the settings:** *Start with Windows*, *Screensaver* and *Show media* are auto-check boxes. If the
  main loop rejects the change, the box stays in the wrong state until the next real change.
- **Keyboard:** no `IsDialogMessage`, so Tab does not move between controls and there is no focus rectangle; the mouse wheel
  over the brightness slider is swallowed. The profile editor shares the window procedure, so resizing it re-lays-out the
  settings window and the wheel scrolls the settings page.
- **Profile editor:** opening another profile replaces unsaved text without asking; *Reset* on a custom profile overwrites the
  file; *Format* can push a profile past 64 KiB (the editor counts UTF-16 units, the validator UTF-8 bytes). If the first
  creation of the editor window fails, every retry says "Unable to register editor window".
- **Possible flicker** in the classic OLED preview (drawn straight to the window DC) — unverified.

## Modern settings window

- **Small screens:** at 150 % scaling or more on a 1366 × 768 display the minimum size (800 × 538 logical px) is larger than the
  work area.
- **Editor text box** can flash light colours in dark mode while a modal file dialog is open (`WM_CTLCOLORSTATIC`).
- **Re-entrancy:** `ReleaseCapture()` delivers the click synchronously, so modal dialogs run *inside* it. It works, but code
  placed after it must not assume queued events. If a modal closure panicked, the busy flag would stay set (the fall-back to
  the classic window masks that).
- **Contrast.** Muted text on dark cards is 3.6 : 1, accent text on white and the light-theme green pill are 2.86 : 1, the
  error colour on white is 3.76 : 1 and the hover of the primary button 2.56 : 1 — below WCAG AA (4.5 : 1) in places.
- **Slider keys.** With a slider focused, Home / End / PageUp / PageDown scroll the page. Sliders with a range above 100 move by
  2 (speed by 4) per key press, so odd values — and odd frame numbers of the trim sliders on GIFs over 101 frames — cannot be
  reached with the keyboard.
- **No fallback font.** The embedded Inter subset covers Basic Latin, Latin-1, Latin Extended-A and punctuation. A GIF or
  profile named in Cyrillic, Greek or CJK, an emoji, "€" / "₺" and a localised Windows error text show as blanks or boxes.
- **`fit()` is slow on the first draw of a very long label** (a 120-character name costs about 100 text measurements once and
  the results are never evicted). A first-draw hitch, not a leak.
- **Studio page.** *Open project* and Play / Pause decode a released GIF on the UI thread just to learn whether one exists;
  open errors only reach `eprintln!` (invisible in the windowed build), so the editor shows an empty state without a reason;
  the preview clock restarts while the page is hidden (a GIF added from the tray starts mid-GIF); re-rendering a very long GIF
  runs on the UI thread and could make Windows mark the window "Not responding".
- **Small things:** the keyboard focus ring disappears on the hovered *Quit* button; the editor title is English-only.
- **Internal:** dead layout branches (`Layout::MAX_W`, `two_cols`), 29 of the 85 embedded icons are unused, render tests only prove
  "opaque and not flat" (no test fails if all text disappeared), no tests for `text_width`, `fit` or `Lang::upper`.

## GIF Studio and the engines

- **Thin lines in very large sources alias.** The sampler averages at most 4 × 4 taps per screen pixel; the 480 px working size
  is up to 7.5 × larger than the screen height, so a one-pixel line can flicker. The test that covers the sampler passes because 64 → 8
  and 480 → 8 happen to land on texel borders. Fix: shrink with an area filter before sampling and add a 63 → 8 test.
- **Resize on straight alpha.** GIFs larger than 480 px (and the thumbnails) are resized without premultiplying, so the colour hidden
  under transparent pixels can leak into edges.
- **`GifPreview` loops a play-once GIF** (the *OLED preview* of the live panel when no base station is connected); the OLED layer
  and the Studio previews honour *play once*.
- **Relative project paths** are resolved against the process's working directory, not the project file. The GIF a project names is
  read with `fs::read` after a size check, so a pipe or a file that keeps growing is not bounded.
- **CLI (`gxoled-cli`).** `-x -5` / `-y -5` are rejected by the argument parser; brightness `0` or `11+` and a missing device end in a
  panic (`unwrap`); an image whose extension is missing or unknown panics in `decode_frames`; the *fell behind* message tests the
  time before drawing; `-l 0` with no decoded frame spins a core. The CLI decodes GIFs through `gxoled_draw::decode_frames` (hard
  threshold, none of the Studio limits), not through `gxoled_studio`.
- **HID (hardware behaviour not verified).** A partial draw whose height is not a multiple of 8 carries zero padding rows that can clear
  up to 7 rows below it; `y` is sent unaligned; `h == 0` still sends reports; a failed report blocks the render thread about 385 ms
  (`retry_report`); `probe()` writes through the info handle, which `VOLUME_OSD_HANDOFF.md` says fails on Windows (12E5), so
  `DrawCommand::Probe` probably never works there; debug builds print every received report.
- **Connection errors are swallowed** (`gxoled_lib`: "Failed to connect to USB device" hides *permission denied* on Linux) and
  `Device::connect` builds a new `HidApi` (a full enumeration) every second on the UI thread while waiting for a base station.
- **Text baseline depends on the string.** Line bitmaps are cropped to the ink, so `aaa` and `Aaa` stacked at the same y differ by 2 px
  at size 16 and multi-line blocks get an uneven line pitch.
- **Internal:** dead code in `gxoled_draw` (`try_decode_gif_frames`, `casio_clock_preview`, `LayerId::none`, the `follow_fps` branch,
  public `texter`) and in `gxoled_app` (`media_visible = false` remnants in the clock layout, an unread `media_mode`, stale
  comments and a debug `println!` per menu event); `media_layers` is not cleared when media ends (a harmless message to the render
  thread every second) and the idle `clear_layers()` leaves `time_layers` stale (the clock recovers); two wrong comments in the
  dither tests (the assertions are right).

## Build, CI and packaging

- **Actions are pinned by tag** (`@v7` …), not by commit SHA. `build.yml` is read-only except in its dormant tag job; `release.yml` needs
  `contents: write` for the whole run (since v0.6.2 the checkout no longer leaves the token in `.git/config` while dependency build
  scripts run).
- **The legacy `v*` tag job in `build.yml` is dormant, not removed.** A tag pushed by hand would upload the MinGW-built `gxoled.exe`,
  `gxoled-cli.exe` and the Flatpak with no checksums or notes. Tags created by `release.yml` (with `GITHUB_TOKEN`) do not trigger it.
  The official Windows EXE is the MSVC build of `release.yml`.
- **MinGW cross-build has no icon or version resource** (`build.rs` decides by the *host* OS). Only the legacy job publishes it.
- **No `cargo fmt --check`, no `clippy`, no `rust-version`.** `cargo fmt --check` differs in 491 places today; the code needs Rust 1.88 or
  newer (edition 2024, let-chains). CI applies `-D warnings` (through `setup-rust-toolchain`); `Validate-Windows.ps1` does not.
- **CI duplication:** `actions/cache` on `~/.cargo` and `target` overlaps the cache of `setup-rust-toolchain`, and push + pull-request
  runs everything twice for same-repository branches.
- **Flatpak.** `flatpak/build.sh` clones `flathub/shared-modules` at its moving HEAD and runs `cargo vendor` without `--locked`; the
  metainfo has no `<releases>`, `<url>` or content rating; the sandbox asks for `--device=all`; `.gitattributes` has no
  `*.sh text eol=lf`, so a CRLF checkout breaks `build.sh`.
- **udev rules** use `MODE="0666"` (any local user may talk to the HID device); `TAG+="uaccess"` would be tighter. All six supported
  product IDs are covered since v0.6.2.
- **The embedded Go helper cannot be proven identical to the source.** `go_animations/gxoled-animations.exe` (2 807 808 bytes,
  go1.23.2, `-trimpath`, stripped) was committed with the Go source in one commit (`4f5c49b`, 2026-10-27). Its profile bytes, flag and
  error strings and the 196 function names of the project's own packages match a fresh build of the current tree, but byte-for-byte
  equality could not be shown (the exact toolchain could not be downloaded). The new CI job builds and tests the Go *source*; it does not
  rebuild the committed EXE. `gxoled_app/src/go_bridge.rs` also reads the whole 2.8 MB EXE on every start and replaces the extracted copy
  by remove-then-rename (a locked old copy or a virus scanner can make an update fail).
- **Third-party notices** list crates by licence; they do not reproduce each crate's own copyright line (a generated `cargo about`
  bundle would).

## Release page

A Claude session can create a release but cannot edit or delete one (the GitHub API answers 403). The older releases `v0.6.1`, `v0.6.0`
and `v0.5.3` therefore keep their long descriptions until the owner replaces them:

```
gh release edit v0.6.1 --notes-file release-notes/v0.6.1.md
gh release edit v0.6.0 --notes-file release-notes/v0.6.0.md
gh release edit v0.5.3 --notes-file release-notes/v0.5.3.md
```

(or `gh release delete <tag>` — the tags and release branches stay). The repository description, topics and the social preview picture
(`docs/images/social-preview.png`, 1280 × 640) are also settings of the owner's GitHub account.

## Not verified on real hardware

Everything on this page and in the changelog was compiled and unit-tested (Linux; Windows in CI) or type-checked, never run on a PC with
a SteelSeries base station. Specifically **not** verified: how the new icons look in the tray, taskbar, title bars and Explorer (125 – 200 %
scaling included); the red tray icon and its tooltip when the DAC is unplugged; GIF and animation speed on the real OLED (the figures come
from a model of the render loop); the supervised Go helper with a bad profile; the *Start with Windows* repair; the minimised-window
behaviour; the movement of the two fixed clock styles on the real panel; file dialogs, DPI changes between monitors, DWM caption colours and the
native text control on a real desktop; the Nova Elite (`2244`, unofficial) and the Windows information-collection writes; the OSD hand-off
timing. GGIF-Studio's own repository could not be fetched during the audit, so "faithful to GGIF-Studio" rests on the documented option set
and algorithm, not on a line-by-line comparison.

If something here bites you, please open an issue on GitHub and say which window (classic / modern), which base station and whether it
was a fresh start.
