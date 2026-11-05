# Branches, tags and releases

## One rule

**Git tags and release branches are never deleted, retagged or force-pushed.** GitHub *releases* are different: the
Releases page is meant to show the current release and the *GxOled Light* `stable` release, so older releases are removed
by the owner (in the GitHub UI or with `gh release delete <tag>`). A Claude session cannot edit or delete releases (the
API answers 403), so that step is always the owner's. Deleting a release leaves its tag and branch in place, which is
why the release workflow also refuses a tag that already exists.

## Releases (what users can download)

| Tag | Release | Notes |
| --- | --- | --- |
| `v0.6.2` | GxOled Advanced v0.6.2 | Current. New logo and icons, GIFs and animations at their real speed, burn-in mode fix for the big clocks, supervised Go helper, deep bug audit. Assets: `GxOled.Advanced.v0.6.2.exe`, `SHA256SUMS.txt`, `THIRD_PARTY_NOTICES.md`. |
| `stable` | GxOled Light v0.4.0 | Older light build (`GxOled.Light.v0.4.0.exe`, `gxoled-cli.exe`). Fallback; do not touch. |

Older releases (`v0.6.1`, `v0.6.0`, `v0.5.3` and the earlier ones) are superseded; their change notes stay in
[`CHANGELOG.md`](../CHANGELOG.md) and in the README's *Changelogs* section. Short, heading-only replacements for the long bodies of
the three that may still be listed are in [`release-notes/`](../release-notes/); the owner can apply them with
`gh release edit v0.6.1 --notes-file release-notes/v0.6.1.md` (likewise `v0.6.0`, `v0.5.3`).

## Branches

| Branch | Role |
| --- | --- |
| `main` | Source of the latest release. |
| `release/v<version>-advanced` | One per release; pushing it builds the Windows EXE (MSVC, `--locked`), records its SHA-256 and publishes the GitHub release (`.github/workflows/release.yml`). The workflow refuses to run when the branch name does not match the version in `Cargo.toml`, and refuses an existing release **or tag**. |
| `release/v0.5.0-advanced` … `release/v0.6.1-advanced` | Source lines of the earlier releases (references; their releases are removed or being removed). |
| `release/v0.5.3-beta`, `release/v0.5.3-beta.2` | Source of the two trial pre-releases (the beta workflow was retired with v0.5.3). |
| `development/modern-ui-beta` | Working branch of the modern interface and everything after it (v0.5.3 – v0.6.2). Kept. |
| `diagnostics/osd-frame-stamp`, `diagnostics/volume-osd-timing` | The frame-stamp tooling that measured the volume-screen delay for v0.5.2 (see [`VOLUME_OSD_HANDOFF.md`](VOLUME_OSD_HANDOFF.md)); not part of the product. The tags `diag-osd-stamp-*` mark its builds. |
| `fix/media-overlay-and-version-v050` | Older fix branch, already contained in `main`. |
| `automation/republish-exact-v0.5.0` | CI that rebuilds and replaces v0.5.0 from user-provided source. Do not re-run casually. |
| `release/v0.5.0-studio.1`, `development/studio-2026-10-27` | Early "Studio" experiments, historical. Do not merge. |
| `development/advanced-gif-studio`, `development/studio-classic-editor-2026-10-27` | Early GIF-editor experiments (a separate editor window; undo/redo work). Historical: GIF Studio shipped in v0.6.0 as a page of the modern window, written from scratch in `gxoled_studio` ([`GIF-STUDIO.md`](GIF-STUDIO.md)). |

## How a release is made

1. Change on a branch; keep the classic Win32 settings layout (only add controls, never redesign) — the modern interface lives in `gxoled_ui` and `modern_*.rs`.
2. Bump `version` in the root `Cargo.toml`, the six workspace entries (`gxoled_app`, `gxoled_cli`, `gxoled_draw`, `gxoled_lib`, `gxoled_studio`, `gxoled_ui`) in `Cargo.lock` and the `GxOled Advanced vX.Y.Z` label in
   `gxoled_app/src/settings_window.rs` (the release workflow checks all three).
3. Add a `## GxOled Advanced vX.Y.Z — …` entry at the top of `CHANGELOG.md` (details, with a "Verification status (be precise)" bullet) and write
   `release-notes/vX.Y.Z.md`: **short, descriptive headings only** (at most 30 lines of at most 140 characters) — that file is the body of the
   GitHub release. Update the README (*New in this release*, the badge and the download links name the version). Then run
   `python3 tools/sync_readme_changelog.py` to refresh the README's *Changelogs* section. CI runs the same script with `--check`: it fails when
   the section is stale, when the newest changelog entry is not the version in `Cargo.toml`, when the README does not name that version in its
   download links, or when the release notes are missing or too long.
4. Push the work branch and wait for the *Build* workflow (unit tests on Linux, the Go helper, Windows MSVC build and tests).
5. Push `release/vX.Y.Z-advanced`. The *Release* workflow checks the branch name, the version, the classic-UI markers and the notes file,
   runs `cargo check`, the unit tests and the release build, and creates tag `vX.Y.Z` with `GxOled.Advanced.vX.Y.Z.exe`,
   `SHA256SUMS.txt` (valid for `sha256sum -c`) and `THIRD_PARTY_NOTICES.md`. Fast-forward `main` to the released commit.
6. State honestly what was compiled, what was unit-tested and what was verified on the DAC (the changelog entry does; the release notes link to it).

## GIF Studio and its licence

GIF Studio (crate `gxoled_studio`, page in `gxoled_ui`) is an independent Rust reimplementation of the processing model of
[D4rkTT/GGIF-Studio](https://github.com/D4rkTT/GGIF-Studio) (MIT). The licence was checked against GPL-3.0 and its notice is in
[`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md); attribution for the Go animations stays in
[`go_animations/UPSTREAM.md`](../go_animations/UPSTREAM.md). Not included (and not promised): layers, text layers, a timeline, video files.
