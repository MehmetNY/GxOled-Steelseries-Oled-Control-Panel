# Documentation index

Everything that is not the product README lives here. Turkish guides are marked **(TR)**.

| Document | What it answers |
| --- | --- |
| [`BRANCHES-AND-RELEASES.md`](BRANCHES-AND-RELEASES.md) | Which branch/tag/release is what, how a release is made, what must not be touched. |
| [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md) | What the v0.6.2 audit found and did **not** fix, what is unverified, and the limits users can hit. |
| [`BRANDING.md`](BRANDING.md) | The logo and icon set: what each file is for, the colour rules (black = running, red = no base station), how to regenerate everything. |
| [`VOLUME_OSD_HANDOFF.md`](VOLUME_OSD_HANDOFF.md) | Why GxOled used to return 4–6 s after a knob turn, how v0.5.2 fixes it, what was ruled out. |
| [`GIF-STUDIO.md`](GIF-STUDIO.md) | The GIF Studio editor: every setting and its range, the processing order, playback timing, project files, limits, what is not included, what is verified. |
| [`MODERN-UI.md`](MODERN-UI.md) | The modern settings interface: how to switch between it and the classic window, what is where, how the OLED mirror and window scaling work, what is verified. |
| [`TR-WINDOWS-DERLEME.md`](TR-WINDOWS-DERLEME.md) **(TR)** | Building GxOled from source on Windows with `Validate-Windows.cmd`, incl. the "path too long" (FTK1011) failure. |
| [`images/`](images/) | Banner, logo set, the main picture and the 3 × 3 screenshot gallery used by the README ([`images/gallery/README.md`](images/gallery/README.md) says how to replace the gallery pictures), plus the reference artwork of the clock designs. |
| [`casio-f91w-proposed.png`](casio-f91w-proposed.png) | Proposed Casio-style clock reference. |

Related files outside `docs/`:

- [`../README.md`](../README.md) — product overview, features, "Developers & AI agents" notes and the per-version *Changelogs* (generated from `CHANGELOG.md`).
- [`../CHANGELOG.md`](../CHANGELOG.md) — release history, the single place where changes are written in full.
- [`../release-notes/`](../release-notes/) — the short, heading-only text that becomes the body of each GitHub release.
- [`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) — licences of everything that ships inside the EXE.
- [`../go_animations/UPSTREAM.md`](../go_animations/UPSTREAM.md) — steelclock-go attribution and license notes.

## Ground rules that apply to every change

1. **Rust is the only OLED writer.** Go helpers, Electron tools or a future GIF editor must never write HID reports.
2. **The original Win32 settings window is not redesigned.** New features get their own control group or window.
3. **Compiled ≠ unit-tested ≠ verified on the DAC.** Release notes and the changelog say which of the three applies.
4. **Git tags and release branches are never deleted, retagged or force-pushed.** GitHub *releases* are tidied by the
   owner (the Releases page shows the current release and *GxOled Light* `stable`); see
   [`BRANCHES-AND-RELEASES.md`](BRANCHES-AND-RELEASES.md).
5. **The GitHub release body is short** (descriptive headings only); every detail is written once in `CHANGELOG.md`
   and copied into the README by `tools/sync_readme_changelog.py`.
