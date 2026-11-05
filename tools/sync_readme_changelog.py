#!/usr/bin/env python3
"""Keeps the "Changelogs" section of README.md in step with CHANGELOG.md.

    python3 tools/sync_readme_changelog.py            rewrite the generated block of README.md
    python3 tools/sync_readme_changelog.py --check    change nothing; exit 1 when something is out of date

CHANGELOG.md is the one place where a change is written in full. The README shows it as one collapsed block per
version (<details>), newest first. Entries that are not a versioned "GxOled Advanced vX.Y.Z" release (the notes from
before the Advanced releases) are gathered in one last block, "Earlier history". The generated part sits between

    <!-- CHANGELOGS:START ... -->
    <!-- CHANGELOGS:END -->

and is never edited by hand; running the script twice changes nothing the second time.

`--check` (run by CI) additionally verifies that
  * the version in Cargo.toml is the newest "GxOled Advanced v..." entry of the changelog,
  * the README names that version in its download links,
  * release-notes/v<version>.md exists and is short: it is the body of the GitHub release, which carries only
    descriptive headings (the details live in the changelog).

Standard library only.
"""
from __future__ import annotations

import argparse
import difflib
import html
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The release notes are the body of the GitHub release: short, descriptive headings only.
MAX_NOTE_LINES = 30  # non-empty lines
MAX_NOTE_LINE_LENGTH = 140  # characters in one line

START = re.compile(r"<!-- CHANGELOGS:START.*?-->")
END = "<!-- CHANGELOGS:END -->"
VERSION_HEADING = re.compile(
    r"^## GxOled Advanced v(?P<version>\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?)\s+[—–-]+\s+(?P<title>\S.*?)\s*$"
)
HEADING = re.compile(r"^(#{1,6})\s+(\S.*?)\s*$")
FENCE = re.compile(r"^\s*(```|~~~)")


@dataclass
class Section:
    heading: str  # the text of the "## ..." line
    version: str | None = None  # set for "GxOled Advanced vX.Y.Z — title"
    title: str = ""
    lines: list[str] = field(default_factory=list)


def parse(text: str) -> list[Section]:
    """Splits the changelog at its level-2 headings (a heading inside a code fence does not count)."""
    sections: list[Section] = []
    in_fence = False
    for line in text.replace("\r\n", "\n").split("\n"):
        if FENCE.match(line):
            in_fence = not in_fence
        if not in_fence and line.startswith("## "):
            m = VERSION_HEADING.match(line)
            sections.append(
                Section(heading=line[3:].strip(), version=m["version"] if m else None, title=m["title"] if m else "")
            )
        elif sections:
            sections[-1].lines.append(line)
    return sections


def trimmed(lines: list[str]) -> list[str]:
    start, end = 0, len(lines)
    while start < end and not lines[start].strip():
        start += 1
    while end > start and not lines[end - 1].strip():
        end -= 1
    return lines[start:end]


def body(lines: list[str], deeper_by: int | None) -> list[str]:
    """The lines of a section. A heading inside one moves `deeper_by` levels down, or becomes a bold line (None)."""
    out: list[str] = []
    in_fence = False
    for line in lines:
        if FENCE.match(line):
            in_fence = not in_fence
        m = None if in_fence else HEADING.match(line)
        if m is None:
            out.append(line.rstrip())
        elif deeper_by is None:
            out.append(f"**{m.group(2)}**")
        else:
            out.append("#" * min(len(m.group(1)) + deeper_by, 6) + " " + m.group(2))
    return trimmed(out)


def inline(text: str) -> str:
    """Text for a <summary>: HTML-escaped, with `code` spans as <code> (Markdown is not parsed inside <summary>)."""
    return re.sub(r"`([^`]+)`", r"<code>\1</code>", html.escape(text, quote=False))


def render(sections: list[Section]) -> str:
    blocks: list[str] = []
    for s in sections:
        if s.version is None:
            continue
        blocks.append(
            "\n".join(
                ["<details>", f"<summary><b>v{s.version}</b> — {inline(s.title)}</summary>", ""]
                + body(s.lines, None)
                + ["", "</details>"]
            )
        )
    earlier = [s for s in sections if s.version is None]
    if earlier:
        parts: list[str] = []
        for s in earlier:
            parts += ["#### " + s.heading, ""] + body(s.lines, 2) + [""]
        blocks.append(
            "\n".join(
                [
                    "<details>",
                    "<summary><b>Earlier history</b> — notes from before the versioned <i>Advanced</i> releases</summary>",
                    "",
                ]
                + trimmed(parts)
                + ["", "</details>"]
            )
        )
    return "\n\n".join(blocks)


def apply(readme: str, block: str) -> str:
    m = START.search(readme)
    end = readme.find(END)
    if m is None or end < m.end():
        raise SystemExit(f"README.md: the markers '<!-- CHANGELOGS:START ... -->' and '{END}' are missing or in the wrong order")
    return readme[: m.end()] + "\n" + block + "\n" + readme[end:]


def cargo_version(root: Path) -> str:
    text = (root / "Cargo.toml").read_text(encoding="utf-8")
    m = re.search(r'(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', text)
    if m is None:
        raise SystemExit("Cargo.toml: no version under [workspace.package]")
    return m.group(1)


def consistency_problems(root: Path, sections: list[Section], readme: str) -> list[str]:
    problems: list[str] = []
    version = cargo_version(root)
    versions = [s.version for s in sections if s.version]
    if not versions or versions[0] != version:
        newest = f"v{versions[0]}" if versions else "missing"
        problems.append(f"CHANGELOG.md: the newest 'GxOled Advanced v...' entry is {newest}, but Cargo.toml says v{version}")
    for needle in (f"releases/tag/v{version}", f"GxOled.Advanced.v{version}.exe"):
        if needle not in readme:
            problems.append(f"README.md: '{needle}' not found - the download links must name v{version}")
    notes = root / "release-notes" / f"v{version}.md"
    if not notes.is_file():
        problems.append(f"release-notes/v{version}.md is missing (it is the body of the GitHub release)")
    else:
        lines = [line for line in notes.read_text(encoding="utf-8").splitlines() if line.strip()]
        if not lines:
            problems.append(f"release-notes/v{version}.md is empty")
        if len(lines) > MAX_NOTE_LINES:
            problems.append(
                f"release-notes/v{version}.md has {len(lines)} lines (at most {MAX_NOTE_LINES}): the release page carries "
                "short descriptive headings only, details belong in CHANGELOG.md"
            )
        for number, line in enumerate(notes.read_text(encoding="utf-8").splitlines(), 1):
            if len(line) > MAX_NOTE_LINE_LENGTH:
                problems.append(
                    f"release-notes/v{version}.md line {number} is {len(line)} characters (at most {MAX_NOTE_LINE_LENGTH}); "
                    "make it a short heading and put the detail into CHANGELOG.md"
                )
    return problems


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Sync the Changelogs section of README.md with CHANGELOG.md.")
    parser.add_argument("--check", action="store_true", help="change nothing; exit 1 when README.md or a release file is out of date")
    parser.add_argument("--root", type=Path, default=ROOT, help=argparse.SUPPRESS)  # for the tests
    args = parser.parse_args(argv)

    root: Path = args.root
    sections = parse((root / "CHANGELOG.md").read_text(encoding="utf-8"))
    if not any(s.version for s in sections):
        raise SystemExit("CHANGELOG.md: no '## GxOled Advanced vX.Y.Z - title' entry found")
    readme_path = root / "README.md"
    current = readme_path.read_text(encoding="utf-8").replace("\r\n", "\n")
    updated = apply(current, render(sections))
    count = sum(1 for s in sections if s.version)

    if args.check:
        failed = False
        if updated != current:
            failed = True
            print("README.md: the Changelogs block is out of date. Run: python3 tools/sync_readme_changelog.py", file=sys.stderr)
            diff = difflib.unified_diff(current.split("\n"), updated.split("\n"), "README.md (now)", "README.md (expected)", lineterm="", n=0)
            for number, line in enumerate(diff):
                if number >= 30:
                    print("  ...", file=sys.stderr)
                    break
                print("  " + line[:200], file=sys.stderr)
        for problem in consistency_problems(root, sections, current):
            failed = True
            print(problem, file=sys.stderr)
        if not failed:
            print(f"OK: README.md matches CHANGELOG.md ({count} versions); release v{cargo_version(root)} is consistent.")
        return 1 if failed else 0

    if updated != current:
        readme_path.write_text(updated, encoding="utf-8", newline="\n")
        print(f"README.md: Changelogs block updated ({count} versions).")
    else:
        print(f"README.md: Changelogs block already up to date ({count} versions).")
    for problem in consistency_problems(root, sections, updated):
        print("warning: " + problem, file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
