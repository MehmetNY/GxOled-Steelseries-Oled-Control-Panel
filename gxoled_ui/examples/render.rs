//! Renders every page to PNG for visual review:  cargo run -p gxoled_ui --example render -- OUT_DIR [width] [height] [dpi]
use gxoled_ui::{ids, model::*, Ui};

fn digit_rows(c: char) -> [u8; 7] {
    match c {
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        ':' => [0, 0b00100, 0b00100, 0, 0b00100, 0b00100, 0],
        _ => [0; 7],
    }
}

fn sample_oled() -> Vec<bool> {
    let mut px = vec![false; OLED_W * OLED_H];
    let text = "15:47:33";
    let scale = 2;
    let total_w = text.len() * 6 * scale - scale;
    let (x0, y0) = ((OLED_W - total_w) / 2, (OLED_H - 7 * scale) / 2);
    for (i, ch) in text.chars().enumerate() {
        for (ry, row) in digit_rows(ch).iter().enumerate() {
            for cx in 0..5 {
                if row & (1 << (4 - cx)) != 0 {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            px[(y0 + ry * scale + dy) * OLED_W + x0 + (i * 6 + cx) * scale + dx] = true;
                        }
                    }
                }
            }
        }
    }
    px
}

fn sample_qr() -> Image {
    let n = 29u32;
    let mut rgba = vec![255u8; (n * n * 4) as usize];
    for y in 0..n { for x in 0..n { if (x * 7 + y * 13 + x * y) % 3 == 0 { let i = ((y * n + x) * 4) as usize; rgba[i] = 0; rgba[i + 1] = 0; rgba[i + 2] = 0; } } }
    Image { w: n, h: n, rgba }
}

fn sample_studio(lang: Lang) -> StudioModel {
    let spec: [(i32, i32, i32, &str); 11] = [
        (20, 300, 100, "1.00×"), (-64, 64, 0, "+0 px"), (-64, 64, 0, "+0 px"), (-100, 100, 15, "+15"), (-100, 100, 0, "+0"),
        (10, 300, 100, "1.00"), (0, 255, 128, "128"), (10, 500, 100, "1.00×"), (10, 200, 30, "30 ms"), (0, 23, 0, "#1"), (0, 23, 23, "#24"),
    ];
    let (w, h) = (96u32, 64u32);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h { for x in 0..w {
        let i = ((y * w + x) * 4) as usize;
        let d = (((x as f32 - 48.0).powi(2) + (y as f32 - 32.0).powi(2)).sqrt() / 40.0).min(1.0);
        rgba[i] = (255.0 * (1.0 - d)) as u8; rgba[i + 1] = (180.0 * (1.0 - d)) as u8; rgba[i + 2] = (90.0 + 120.0 * d) as u8; rgba[i + 3] = 255;
    } }
    let mut mono = vec![false; 128 * 64];
    for y in 0..64usize { for x in 0..128usize {
        let d = (((x as f32 - 64.0).powi(2) + ((y as f32 - 32.0) * 1.3).powi(2)).sqrt()) / 38.0;
        mono[y * 128 + x] = d < 1.0 && (x + y) % 2 == 0 || d < 0.6;
    } }
    StudioModel {
        loaded: true,
        name: "nyan-cat-final-v2.gif".into(),
        info: if lang == Lang::Tr { "96×64 · 24 kare · 2,4 sn".into() } else { "96×64 · 24 frames · 2.4 s".into() },
        sliders: spec.iter().enumerate().map(|(i, (min, max, value, text))| StudioSlider { id: ids::STUDIO_SLIDER_BASE + i as u32, min: *min, max: *max, value: *value, text: text.to_string() }).collect(),
        source: Some(Image { w, h, rgba }),
        mono, mono_w: 128, mono_h: 64,
        status: if lang == Lang::Tr { "Dışa aktarıldı: nyan.gif".into() } else { "Exported nyan.gif".into() },
        status_ok: true,
    }
}

pub fn sample(lang: Lang, dark: bool, page: Page) -> UiModel {
    UiModel {
        studio: sample_studio(lang),
        lang, dark, page,
        checked: vec![ids::MODE_WINDOW, ids::LANG_EN, if dark { ids::THEME_DARK } else { ids::THEME_LIGHT }, ids::STARTUP, ids::QUIT_SCREENSAVER,
            ids::UI_MODERN, ids::STYLE_ANALOG_ROUND, ids::TIME_24, ids::SHIFT_LOW, ids::BURN_OFF, ids::NOTIFY_ALWAYS,
            ids::VOLUME_HOLD_BASE + 3, ids::MEDIA, ids::IDLE,
            ids::STUDIO_FIT_CONTAIN, ids::STUDIO_ROTATE_BASE, ids::STUDIO_BG_BLACK, ids::STUDIO_DITHER_BASE, ids::STUDIO_LOOP, ids::STUDIO_NORMALIZE],
        brightness: Some(7),
        profiles: ["Matrix Rain — Default", "Hyperspace — Default", "Star Wars Intro — Default", "Game of Life — Default", "matrix-001.json — matrix", "life-002.json — life"].iter().map(|s| s.to_string()).collect(),
        profile_selected: 1,
        profile_running: Some(0),
        oled: sample_oled(),
        connected: true,
        version: "v0.5.3".into(),
        qr: Some(sample_qr()),
        ..UiModel::default()
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| ".".into());
    let w: u32 = a.get(2).and_then(|v| v.parse().ok()).unwrap_or(1450);
    let h: u32 = a.get(3).and_then(|v| v.parse().ok()).unwrap_or(974);
    let dpi: f32 = a.get(4).and_then(|v| v.parse().ok()).unwrap_or(1.0);
    let only = a.get(5).cloned();
    std::fs::create_dir_all(&out).unwrap();
    let mut ui = Ui::new();
    ui.set_size(w, h, dpi);
    for (lang, lname) in [(Lang::En, "en"), (Lang::Tr, "tr")] {
        for (dark, tname) in [(true, "dark"), (false, "light")] {
            for page in Page::ALL {
                let tag = format!("{}_{}_{}", page.index(), lname, tname);
                if let Some(o) = &only { if !tag.contains(o.as_str()) { continue; } }
                ui.model = sample(lang, dark, page);
                ui.invalidate();
                let t = std::time::Instant::now();
                let f = ui.frame().unwrap();
                let ms = t.elapsed().as_secs_f32() * 1000.0;
                let (fw, fh, rgba) = (f.width, f.height, f.rgba.to_vec());
                image_save(&format!("{out}/{tag}.png"), fw, fh, &rgba);
                println!("{tag}: {fw}x{fh} {ms:.1} ms, {} hits", ui.hits().len());
            }
        }
    }
    // Profile editor window (930 x 680 logical px).
    let mut ed = Ui::new();
    ed.set_size((930.0 * dpi) as u32, (680.0 * dpi) as u32, dpi);
    for (lang, lname) in [(Lang::En, "en"), (Lang::Tr, "tr")] {
        for (dark, tname) in [(true, "dark"), (false, "light")] {
            for (ok, status) in [(true, "JSON is valid - 128x64 widget, refresh 50 ms."), (false, "Invalid JSON: expected `,` or `}` at line 12 column 5")] {
                let tag = format!("editor_{}_{}_{}", lname, tname, if ok { "ok" } else { "err" });
                if let Some(o) = &only { if !tag.contains(o.as_str()) { continue; } }
                ed.model = UiModel {
                    surface: Surface::Editor, lang, dark,
                    editor_title: "matrix-001.json - matrix".into(),
                    editor_status: status.into(), editor_status_ok: ok,
                    ..UiModel::default()
                };
                ed.invalidate();
                let f = ed.frame().unwrap();
                let (fw, fh, rgba) = (f.width, f.height, f.rgba.to_vec());
                image_save(&format!("{out}/{tag}.png"), fw, fh, &rgba);
                println!("{tag}: {fw}x{fh}, text rect {:?}", ed.editor_text_rect_px());
            }
        }
    }
}

fn image_save(path: &str, w: u32, h: u32, rgba: &[u8]) {
    let pm = resvg_pixmap(w, h, rgba);
    pm.save_png(path).unwrap();
}

fn resvg_pixmap(w: u32, h: u32, rgba: &[u8]) -> gxoled_ui::engine::Pixmap {
    let mut pm = gxoled_ui::engine::Pixmap::new(w, h).unwrap();
    pm.data_mut().copy_from_slice(rgba);
    pm
}
