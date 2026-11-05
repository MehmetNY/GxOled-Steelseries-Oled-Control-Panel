# Screenshot gallery

The README shows a 3 × 3 gallery. Its nine slots are the files in this folder, read left to right, top to bottom:

| | | |
| --- | --- | --- |
| `01.png` | `02.png` | `03.png` |
| `04.png` | `05.png` | `06.png` |
| `07.png` | `08.png` | `09.png` |

They are **placeholders** ("Your screenshot goes here") until you replace them.

## Filling a slot

1. Save your picture as a **PNG** with exactly the slot's name, for example `docs/images/gallery/03.png`.
2. Commit it. The README needs no edit: it shows `docs/images/gallery/NN.png`, so a replaced file simply appears.

Tips:

- Any size works (the README scales it to about 270 px wide and a click opens the full picture). **16 : 10** (for example
  1600 × 1000) fills the slot best; the generated placeholders are 960 × 600.
- Keep each file under about 1 MB (a window screenshot as an optimised PNG is usually 100 – 400 KB).
- A JPEG renamed to `.png` is not a PNG; convert it. A different extension would need the README line changed.
- Re-running `python3 tools/make_branding.py` never overwrites a slot that already holds your picture; it only refreshes empty
  slots and the untouched placeholders.

A possible order (only a suggestion): 01 General · 02 Clock gallery · 03 Animations · 04 GIF Studio · 05 Display · 06 Media ·
07 the base station's OLED itself (a photo) · 08 tray menu · 09 About.

## Earlier screenshots

The six pictures the README showed before v0.6.2 are still hosted by GitHub and can be re-used (download them from the links and
save them under the slot names above):

| # | Address |
| --- | --- |
| 1 | https://github.com/user-attachments/assets/08087e53-ac59-4d96-aae2-86bff951c585 |
| 2 | https://github.com/user-attachments/assets/fb2c1cee-2b0d-46e3-b5f1-37fb7fe9b70e |
| 3 | https://github.com/user-attachments/assets/e9859914-d039-4977-921e-a2932d05d144 |
| 4 | https://github.com/user-attachments/assets/6509c9bb-7f34-4ff1-be66-3aa4269433f9 |
| 5 | https://github.com/user-attachments/assets/00c89698-fb6e-40c8-aa4c-09924e6991c4 |
| 6 | https://github.com/user-attachments/assets/3a29fcb3-6db1-45e6-bc81-ceef59a82684 |

The main picture at the top of the README is `docs/images/gxoled-advanced-main.png` (replace it the same way).
