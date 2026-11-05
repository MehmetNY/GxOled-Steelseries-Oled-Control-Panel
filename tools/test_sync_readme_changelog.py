"""Tests for tools/sync_readme_changelog.py:  python3 -m unittest discover -s tools -p "test_*.py" """
from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path

import sync_readme_changelog as tool

CHANGELOG = """## GxOled Advanced v1.2.0 — second `thing` & <more>

- **New:** one.
- **Fix:** two.

## GxOled Advanced v1.1.0-beta.2 — a beta

### A heading inside
- detail

```
## not a section
```

## 2026-01-01 – Old notes

### Details
- very old
"""

README = """# Title

## Changelogs

<!-- CHANGELOGS:START - generated -->
stale text
<!-- CHANGELOGS:END -->

## After
[v1.2.0](https://example.invalid/releases/tag/v1.2.0) GxOled.Advanced.v1.2.0.exe
"""

CARGO = '[workspace]\nmembers = []\n\n[workspace.package]\nversion = "1.2.0"\n'


def run(*argv: str) -> tuple[int, str, str]:
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = tool.main(list(argv))
    return code, out.getvalue(), err.getvalue()


class Repo(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)
        (self.root / "CHANGELOG.md").write_text(CHANGELOG, encoding="utf-8")
        (self.root / "README.md").write_text(README, encoding="utf-8")
        (self.root / "Cargo.toml").write_text(CARGO, encoding="utf-8")
        (self.root / "release-notes").mkdir()
        (self.root / "release-notes" / "v1.2.0.md").write_text("### New\n- One short heading\n", encoding="utf-8")

    def readme(self) -> str:
        return (self.root / "README.md").read_text(encoding="utf-8")

    def args(self, *extra: str) -> list[str]:
        return ["--root", str(self.root), *extra]


class Parsing(unittest.TestCase):
    def test_sections_and_versions(self) -> None:
        sections = tool.parse(CHANGELOG)
        self.assertEqual([s.version for s in sections], ["1.2.0", "1.1.0-beta.2", None])
        self.assertEqual(sections[0].title, "second `thing` & <more>")
        # "## not a section" sits in a code fence and belongs to the beta entry
        self.assertTrue(any("not a section" in line for line in sections[1].lines))

    def test_render_has_one_block_per_version_and_the_old_notes_last(self) -> None:
        text = tool.render(tool.parse(CHANGELOG))
        self.assertEqual(text.count("<details>"), 3)
        self.assertEqual(text.count("</details>"), 3)
        self.assertLess(text.index("v1.2.0"), text.index("v1.1.0-beta.2"))
        self.assertLess(text.index("v1.1.0-beta.2"), text.index("Earlier history"))
        # escaped and with code spans in the summary
        self.assertIn("<summary><b>v1.2.0</b> — second <code>thing</code> &amp; &lt;more&gt;</summary>", text)
        # headings never reach the README outline from a version block; old entries move down two levels
        self.assertIn("**A heading inside**", text)
        self.assertNotIn("\n### A heading inside", text)
        self.assertIn("#### 2026-01-01 – Old notes", text)
        self.assertIn("##### Details", text)
        # a blank line after <summary> and before </details> keeps GitHub rendering the Markdown inside
        self.assertIn("</summary>\n\n- **New:** one.", text)
        self.assertIn("- **Fix:** two.\n\n</details>", text)

    def test_apply_is_idempotent_and_keeps_the_rest(self) -> None:
        block = tool.render(tool.parse(CHANGELOG))
        once = tool.apply(README, block)
        self.assertEqual(tool.apply(once, block), once)
        self.assertTrue(once.startswith("# Title\n\n## Changelogs\n\n<!-- CHANGELOGS:START - generated -->\n<details>"))
        self.assertTrue(once.endswith("<!-- CHANGELOGS:END -->\n\n## After\n[v1.2.0](https://example.invalid/releases/tag/v1.2.0) GxOled.Advanced.v1.2.0.exe\n"))
        self.assertNotIn("stale text", once)

    def test_missing_or_reversed_markers_are_an_error(self) -> None:
        with self.assertRaises(SystemExit):
            tool.apply("# no markers", "x")
        with self.assertRaises(SystemExit):
            tool.apply("<!-- CHANGELOGS:END -->\n<!-- CHANGELOGS:START -->", "x")


class Command(Repo):
    def test_check_fails_on_a_stale_block_and_write_fixes_it(self) -> None:
        code, _, err = run(*self.args("--check"))
        self.assertEqual(code, 1)
        self.assertIn("out of date", err)
        code, out, _ = run(*self.args())
        self.assertEqual(code, 0)
        self.assertIn("updated", out)
        self.assertEqual(run(*self.args("--check"))[0], 0)
        self.assertIn("already up to date", run(*self.args())[1])

    def test_a_changelog_edit_makes_the_check_fail_again(self) -> None:
        run(*self.args())
        path = self.root / "CHANGELOG.md"
        path.write_text(path.read_text(encoding="utf-8").replace("- **Fix:** two.", "- **Fix:** two, changed."), encoding="utf-8")
        self.assertEqual(run(*self.args("--check"))[0], 1)
        run(*self.args())
        self.assertEqual(run(*self.args("--check"))[0], 0)
        self.assertIn("two, changed.", self.readme())

    def test_crlf_checkouts_are_not_a_difference(self) -> None:
        run(*self.args())
        (self.root / "README.md").write_text(self.readme().replace("\n", "\r\n"), encoding="utf-8", newline="")
        self.assertEqual(run(*self.args("--check"))[0], 0)

    def test_release_consistency(self) -> None:
        run(*self.args())
        self.assertEqual(run(*self.args("--check"))[0], 0)

        (self.root / "Cargo.toml").write_text(CARGO.replace("1.2.0", "1.3.0"), encoding="utf-8")
        code, _, err = run(*self.args("--check"))
        self.assertEqual(code, 1)
        self.assertIn("Cargo.toml says v1.3.0", err)
        self.assertIn("releases/tag/v1.3.0", err)
        self.assertIn("release-notes/v1.3.0.md is missing", err)
        (self.root / "Cargo.toml").write_text(CARGO, encoding="utf-8")

        notes = self.root / "release-notes" / "v1.2.0.md"
        notes.write_text("\n".join(f"- heading {i}" for i in range(tool.MAX_NOTE_LINES + 1)), encoding="utf-8")
        self.assertIn("at most", run(*self.args("--check"))[2])
        notes.write_text("- " + "x" * tool.MAX_NOTE_LINE_LENGTH, encoding="utf-8")
        self.assertIn("characters", run(*self.args("--check"))[2])
        notes.write_text("", encoding="utf-8")
        self.assertIn("is empty", run(*self.args("--check"))[2])


if __name__ == "__main__":
    unittest.main()
