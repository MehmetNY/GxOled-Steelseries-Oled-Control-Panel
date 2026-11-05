//! Behaviour of the modern UI toolkit without any window: rendering, hit testing, hover/press/focus,
//! keyboard navigation, scrolling, slider and profile-list handling.

use gxoled_ui::{
    ids,
    model::{Cursor, Image, Lang, Page, StudioModel, StudioSlider, UiEvent, UiModel, OLED_H, OLED_W},
    scene::{Hit, HitKind},
    Key, Ui,
};

const W: u32 = 1450;
const H: u32 = 974;

fn model(page: Page) -> UiModel {
    UiModel {
        page,
        checked: vec![ids::MODE_WINDOW, ids::LANG_EN, ids::THEME_DARK, ids::STARTUP, ids::VOLUME_HOLD_BASE + 3],
        brightness: Some(7),
        profiles: (0..14).map(|i| format!("Profile {i}")).collect(),
        profile_selected: 1,
        profile_running: Some(0),
        connected: true,
        version: "v0.5.3".into(),
        ..UiModel::default()
    }
}

fn ui(page: Page, w: u32, h: u32, dpi: f32) -> Ui {
    let mut ui = Ui::new();
    ui.set_size(w, h, dpi);
    ui.model = model(page);
    ui.invalidate();
    ui.frame().expect("frame");
    ui
}

/// Centre of a hit rect in physical pixels under the current scroll position.
fn centre(ui: &Ui, hit: &Hit, w: u32, dpi: f32) -> (f32, f32) {
    let s = Ui::scale_for(w, dpi);
    let y = if hit.scrolls { hit.rect.cy() - ui.scroll_offset() } else { hit.rect.cy() };
    (hit.rect.cx() * s, y * s)
}

fn find(ui: &Ui, id: u32) -> Hit {
    ui.hits().iter().find(|h| h.id == id).cloned().unwrap_or_else(|| panic!("no hit with id {id}"))
}

fn click(ui: &mut Ui, id: u32) {
    let hit = find(ui, id);
    let (x, y) = centre(ui, &hit, W, 1.0);
    ui.mouse_move(x, y);
    ui.mouse_down(x, y);
    ui.mouse_up(x, y);
}

#[test]
fn every_page_renders_in_every_variant() {
    for (w, h, dpi) in [(W, H, 1.0), (1240, 840, 1.0), (1000, 700, 1.0), (1860, 1260, 1.5), (2400, 1400, 2.0)] {
        let mut ui = Ui::new();
        ui.set_size(w, h, dpi);
        for lang in [Lang::En, Lang::Tr] {
            for dark in [true, false] {
                for page in Page::ALL {
                    let mut m = model(page);
                    m.lang = lang;
                    m.dark = dark;
                    ui.model = m;
                    ui.invalidate();
                    let f = ui.frame().expect("frame");
                    assert_eq!((f.width, f.height), (w, h));
                    assert_eq!(f.rgba.len(), (w * h * 4) as usize);
                    // Opaque and not blank.
                    assert!(f.rgba.chunks_exact(4).all(|p| p[3] == 255), "{page:?} {lang:?} dark={dark} has transparent pixels");
                    let first = &f.rgba[..4];
                    assert!(f.rgba.chunks_exact(4).any(|p| p != first), "{page:?} rendered flat");
                }
            }
        }
    }
}

#[test]
fn hit_rects_are_finite_and_inside_the_window_width() {
    for (w, h) in [(W, H), (1000, 700), (1240, 840)] {
        let mut ui = Ui::new();
        ui.set_size(w, h, 1.0);
        for page in Page::ALL {
            ui.model = model(page);
            ui.invalidate();
            ui.frame();
            let scale = Ui::scale_for(w, 1.0);
            let width_du = w as f32 / scale;
            for hit in ui.hits() {
                let r = hit.rect;
                assert!(r.x.is_finite() && r.y.is_finite() && r.w > 0.0 && r.h > 0.0, "{page:?} id {} bad rect {r:?}", hit.id);
                assert!(r.x >= 0.0 && r.right() <= width_du + 0.5, "{page:?} id {} outside window: {r:?} (width {width_du})", hit.id);
            }
        }
    }
}

/// Resizing the window zooms the interface; nothing may move relative to anything else. The layout is computed in
/// design units, so every control must have exactly the same rectangle at every window size, in both languages.
#[test]
fn layout_is_identical_at_every_window_size() {
    fn layout(page: Page, lang: Lang, w: u32, h: u32, dpi: f32) -> Vec<(u32, [f32; 4])> {
        let mut ui = Ui::new();
        ui.set_size(w, h, dpi);
        let mut m = model(page);
        m.lang = lang;
        ui.model = m;
        ui.invalidate();
        ui.frame().expect("frame");
        ui.hits().iter().map(|hit| (hit.id, [hit.rect.x, hit.rect.y, hit.rect.w, hit.rect.h])).collect()
    }
    for lang in [Lang::En, Lang::Tr] {
        for page in Page::ALL {
            let reference = layout(page, lang, W, H, 1.0);
            assert!(!reference.is_empty(), "{page:?} has no controls");
            for (w, h, dpi) in [(960, 620, 1.0), (1000, 700, 1.0), (1240, 840, 1.0), (1700, 1000, 1.0), (2175, 1461, 1.5), (3000, 1800, 2.0)] {
                let other = layout(page, lang, w, h, dpi);
                assert_eq!(reference.len(), other.len(), "{page:?} {lang:?} {w}x{h}: different number of controls");
                for ((id_a, a), (id_b, b)) in reference.iter().zip(&other) {
                    assert_eq!(id_a, id_b);
                    for k in 0..4 {
                        assert!((a[k] - b[k]).abs() < 0.05, "{page:?} {lang:?} {w}x{h}: control {id_a} moved: {a:?} vs {b:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn the_scale_follows_the_window_width_without_limits() {
    assert!((Ui::scale_for(1450, 1.0) - 1.0).abs() < 1e-6);
    assert!((Ui::scale_for(725, 1.0) - 0.5).abs() < 1e-6);
    assert!((Ui::scale_for(2900, 2.0) - 2.0).abs() < 1e-6);
    assert!((Ui::scale_for(4350, 1.0) - 3.0).abs() < 1e-6);
}

#[test]
fn action_ids_are_unique_on_every_page() {
    let mut ui = Ui::new();
    ui.set_size(W, H, 1.0);
    for page in Page::ALL {
        ui.model = model(page);
        ui.invalidate();
        ui.frame();
        let mut seen: Vec<u32> = Vec::new();
        for hit in ui.hits() {
            assert!(!seen.contains(&hit.id), "duplicate hit id {} on {page:?}", hit.id);
            seen.push(hit.id);
        }
    }
}

#[test]
fn clicking_a_radio_reports_its_action() {
    let mut ui = ui(Page::General, W, H, 1.0);
    click(&mut ui, ids::THEME_LIGHT);
    assert_eq!(ui.take_events(), vec![UiEvent::Action(ids::THEME_LIGHT)]);
}

#[test]
fn press_and_release_on_different_targets_does_nothing() {
    let mut ui = ui(Page::General, W, H, 1.0);
    let a = find(&ui, ids::THEME_LIGHT);
    let b = find(&ui, ids::THEME_DARK);
    let (ax, ay) = centre(&ui, &a, W, 1.0);
    let (bx, by) = centre(&ui, &b, W, 1.0);
    ui.mouse_down(ax, ay);
    ui.mouse_up(bx, by);
    assert!(ui.take_events().is_empty());
}

#[test]
fn hover_marks_dirty_and_changes_the_cursor() {
    let mut ui = ui(Page::General, W, H, 1.0);
    let hit = find(&ui, ids::LANG_TR);
    let (x, y) = centre(&ui, &hit, W, 1.0);
    assert_eq!(ui.cursor_at(2.0, 2.0), Cursor::Arrow);
    assert!(!ui.needs_render());
    ui.mouse_move(x, y);
    assert!(ui.needs_render(), "hover must schedule a repaint");
    assert_eq!(ui.cursor_at(x, y), Cursor::Hand);
    ui.frame();
    ui.mouse_leave();
    assert!(ui.needs_render());
}

#[test]
fn sidebar_navigation_switches_pages_and_resets_scroll() {
    let mut ui = ui(Page::Display, W, 600, 1.0);
    ui.scroll_by(200.0);
    assert!(ui.scroll_offset() > 0.0);
    let nav = Page::Media.nav_id();
    let hit = find(&ui, nav);
    let (x, y) = centre(&ui, &hit, W, 1.0);
    ui.mouse_down(x, y);
    ui.mouse_up(x, y);
    assert_eq!(ui.model.page, Page::Media);
    assert_eq!(ui.scroll_offset(), 0.0);
    assert!(ui.take_events().contains(&UiEvent::PageChanged(Page::Media)));
}

#[test]
fn escape_asks_the_host_to_close() {
    let mut ui = ui(Page::General, W, H, 1.0);
    ui.key_down(Key::Escape);
    assert_eq!(ui.take_events(), vec![UiEvent::Close]);
}

#[test]
fn tab_walks_the_focus_ring_and_enter_activates() {
    let mut ui = ui(Page::General, W, H, 1.0);
    assert_eq!(ui.focused(), None);
    ui.key_down(Key::Tab { shift: false });
    let first = ui.focused().expect("focus after Tab");
    ui.key_down(Key::Tab { shift: false });
    let second = ui.focused().unwrap();
    assert_ne!(first, second);
    ui.key_down(Key::Tab { shift: true });
    assert_eq!(ui.focused(), Some(first));
    for _ in 0..80 {
        if ui.focused() == Some(ids::THEME_LIGHT) {
            break;
        }
        ui.key_down(Key::Tab { shift: false });
    }
    assert_eq!(ui.focused(), Some(ids::THEME_LIGHT), "Tab must reach every control");
    ui.take_events();
    ui.key_down(Key::Enter);
    assert_eq!(ui.take_events(), vec![UiEvent::Action(ids::THEME_LIGHT)]);
    ui.key_down(Key::Space);
    assert_eq!(ui.take_events(), vec![UiEvent::Action(ids::THEME_LIGHT)]);
}

#[test]
fn shift_tab_from_nothing_goes_to_the_last_control() {
    let mut ui = ui(Page::General, W, H, 1.0);
    ui.key_down(Key::Tab { shift: true });
    let last = ui.hits().iter().rev().find(|h| h.focusable).map(|h| h.id);
    assert_eq!(ui.focused(), last);
}

#[test]
fn focusing_an_offscreen_control_scrolls_it_into_view() {
    let mut ui = ui(Page::Display, W, 560, 1.0);
    assert_eq!(ui.scroll_offset(), 0.0);
    for _ in 0..40 {
        ui.key_down(Key::Tab { shift: false });
    }
    ui.frame();
    assert!(ui.scroll_offset() > 0.0, "tabbing deep into the page must scroll");
}

#[test]
fn wheel_scrolls_the_page_and_clamps() {
    let mut ui = ui(Page::Display, W, 500, 1.0);
    ui.wheel(700.0, 300.0, -3.0);
    let a = ui.scroll_offset();
    assert!(a > 0.0);
    for _ in 0..100 {
        ui.wheel(700.0, 300.0, -3.0);
    }
    ui.frame();
    let end = ui.scroll_offset();
    ui.wheel(700.0, 300.0, -3.0);
    assert_eq!(ui.scroll_offset(), end, "scroll must clamp at the bottom");
    for _ in 0..200 {
        ui.wheel(700.0, 300.0, 3.0);
    }
    assert_eq!(ui.scroll_offset(), 0.0);
}

#[test]
fn page_that_fits_does_not_scroll() {
    let mut ui = ui(Page::Media, W, 2000, 1.0);
    ui.wheel(700.0, 300.0, -3.0);
    assert_eq!(ui.scroll_offset(), 0.0);
}

#[test]
fn brightness_slider_maps_the_track_to_1_through_10() {
    let mut ui = ui(Page::Display, W, 1400, 1.0);
    let hit = find(&ui, ids::BRIGHTNESS);
    let HitKind::Slider { track, min, max } = hit.kind else { panic!("brightness is not a slider") };
    assert_eq!((min, max), (1, 10));
    let s = Ui::scale_for(W, 1.0);
    let y = (track.cy() - ui.scroll_offset()) * s;
    ui.mouse_down((track.x + 1.0) * s, y);
    assert!(matches!(ui.take_events().last(), Some(UiEvent::Brightness(1))));
    // Dragging far to the left of the track stays clamped at the minimum.
    ui.mouse_move(0.0, y);
    assert_eq!(ui.model.brightness, Some(1));
    ui.mouse_move((track.right() + 60.0) * s, y);
    assert!(matches!(ui.take_events().last(), Some(UiEvent::Brightness(10))));
    ui.mouse_move((track.x + track.w * 0.5) * s, y);
    let mid = ui.take_events();
    let Some(UiEvent::Brightness(v)) = mid.last() else { panic!("no brightness event: {mid:?}") };
    assert!((5..=6).contains(v), "midpoint gave {v}");
    ui.mouse_up(0.0, 0.0);
    assert!(ui.take_events().is_empty(), "releasing a drag is not a click");
    assert_eq!(ui.model.brightness, Some(*v));
}

#[test]
fn slider_arrow_keys_step_by_one() {
    let mut ui = ui(Page::Display, W, 1400, 1.0);
    let hit = find(&ui, ids::BRIGHTNESS);
    let (x, y) = centre(&ui, &hit, W, 1.0);
    ui.mouse_down(x, y);
    ui.mouse_up(x, y);
    ui.take_events();
    ui.model.brightness = Some(4);
    ui.key_down(Key::Right);
    assert_eq!(ui.take_events(), vec![UiEvent::Brightness(5)]);
    ui.key_down(Key::Left);
    ui.key_down(Key::Left);
    assert_eq!(ui.model.brightness, Some(3));
}

#[test]
fn profile_list_selection_and_double_click() {
    let mut ui = ui(Page::Animations, W, 1100, 1.0);
    let row = ids::LIST_BASE + 3;
    click(&mut ui, row);
    assert_eq!(ui.model.profile_selected, 3);
    assert_eq!(ui.take_events(), vec![UiEvent::ProfileSelected(3)]);
    // Second click within the double-click window selects-and-runs.
    click(&mut ui, row);
    assert_eq!(ui.take_events(), vec![UiEvent::Action(ids::PROFILE_SELECT)]);
}

#[test]
fn profile_list_wheel_scrolls_the_list_not_the_page() {
    let mut ui = ui(Page::Animations, W, 600, 1.0);
    assert!(ui.model.profiles.len() > gxoled_ui::pages::list_visible_rows());
    let first = find(&ui, ids::LIST_BASE);
    let (x, y) = centre(&ui, &first, W, 1.0);
    let before = ui.hits().iter().filter(|h| h.id >= ids::LIST_BASE && h.id < ids::LIST_BASE + 100).count();
    ui.wheel(x, y, -1.0);
    ui.frame();
    assert_eq!(ui.scroll_offset(), 0.0, "wheel over the list must not scroll the page");
    let after = ui.hits().iter().filter(|h| h.id >= ids::LIST_BASE && h.id < ids::LIST_BASE + 100).count();
    assert_eq!(before, after, "the list keeps showing a fixed number of rows");
    // The first visible row is now row 1.
    assert!(ui.hits().iter().any(|h| h.id == ids::LIST_BASE + 1));
    assert!(!ui.hits().iter().any(|h| h.id == ids::LIST_BASE));
}

#[test]
fn text_with_forbidden_xml_characters_does_not_blank_the_window() {
    // Profile names, GIF names and OS error messages come from outside; one control character in them used to
    // make the SVG unparseable, which left the window empty.
    let mut ui = ui(Page::Animations, W, 700, 1.0);
    ui.model.profiles[0] = "bad\u{1}name\u{FFFF}".to_owned();
    ui.model.studio.name = "gif\u{8}.gif".to_owned();
    ui.invalidate();
    let frame = ui.frame().expect("the page must still render");
    let first = &frame.rgba[..4];
    assert!(frame.rgba.chunks_exact(4).any(|p| p != first));
}

#[test]
fn a_scrolled_profile_list_that_shrinks_shows_its_first_rows_again() {
    let mut ui = ui(Page::Animations, W, 600, 1.0);
    let visible = gxoled_ui::pages::list_visible_rows();
    let first = find(&ui, ids::LIST_BASE);
    let (x, y) = centre(&ui, &first, W, 1.0);
    ui.wheel(x, y, -1.0);
    ui.wheel(x, y, -1.0);
    ui.frame();
    assert!(!ui.hits().iter().any(|h| h.id == ids::LIST_BASE), "scrolled down: row 0 is out of view");
    // Delete profiles until everything fits: nothing can be scrolled any more, so row 0 has to be back.
    ui.model.profiles.truncate(visible);
    ui.invalidate();
    ui.frame();
    for row in 0..visible as u32 {
        assert!(ui.hits().iter().any(|h| h.id == ids::LIST_BASE + row), "row {row} is missing after the list shrank");
    }
    // One row more than fits: the list scrolls again, but never beyond the last full page.
    ui.model.profiles.push("one more".into());
    ui.invalidate();
    ui.frame();
    assert!(ui.hits().iter().any(|h| h.id == ids::LIST_BASE));
}

#[test]
fn arrow_keys_move_the_profile_selection() {
    let mut ui = ui(Page::Animations, W, 1100, 1.0);
    click(&mut ui, ids::LIST_BASE + 1);
    ui.take_events();
    ui.key_down(Key::Down);
    ui.key_down(Key::Down);
    assert_eq!(ui.model.profile_selected, 3);
    ui.key_down(Key::Up);
    assert_eq!(ui.model.profile_selected, 2);
    assert_eq!(ui.take_events().last(), Some(&UiEvent::ProfileSelected(2)));
    for _ in 0..30 {
        ui.key_down(Key::Down);
    }
    assert_eq!(ui.model.profile_selected, ui.model.profiles.len() - 1);
}

#[test]
fn scroll_position_survives_a_model_refresh() {
    let mut ui = ui(Page::Display, W, 500, 1.0);
    ui.scroll_by(150.0);
    ui.frame();
    let before = ui.scroll_offset();
    ui.model.checked.push(ids::SHIFT_HIGH);
    ui.invalidate();
    ui.frame();
    assert_eq!(ui.scroll_offset(), before);
}

#[test]
fn shrinking_the_window_clamps_the_scroll_offset() {
    let mut ui = ui(Page::Display, W, 500, 1.0);
    ui.scroll_by(10_000.0);
    ui.frame();
    assert!(ui.scroll_offset() > 0.0);
    ui.set_size(W, 2400, 1.0);
    ui.frame();
    assert_eq!(ui.scroll_offset(), 0.0, "a taller window leaves nothing to scroll");
}

#[test]
fn oled_mirror_pixels_reach_the_frame_without_a_full_rerender() {
    let mut ui = ui(Page::General, W, H, 1.0);
    let (x, y, w, h) = ui.oled_rect_px().expect("General page shows the live OLED");
    assert!(w > 100 && h > 50);
    let probe = |ui: &mut Ui| {
        let f = ui.frame().unwrap();
        let i = (((y + h / 2) as u32 * f.width + (x + w / 2) as u32) * 4) as usize;
        f.rgba[i]
    };
    assert_eq!(probe(&mut ui), 0, "unlit OLED is black");
    ui.model.oled = vec![true; OLED_W * OLED_H];
    ui.refresh_overlays();
    assert!(!ui.needs_render(), "overlay refresh must not trigger a full render");
    assert_eq!(probe(&mut ui), 255, "lit OLED is white");
}

#[test]
fn volume_chips_cover_the_seven_choices() {
    let ui = ui(Page::Display, W, 3000, 1.0);
    for i in 0..7 {
        let _ = find(&ui, ids::VOLUME_HOLD_BASE + i);
    }
}

#[test]
fn dpi_scaling_keeps_hit_testing_aligned() {
    let (w, h, dpi) = (1860, 1260, 1.5);
    let mut ui = ui(Page::General, w, h, dpi);
    let hit = find(&ui, ids::LANG_TR);
    let (x, y) = centre(&ui, &hit, w, dpi);
    ui.mouse_down(x, y);
    ui.mouse_up(x, y);
    assert_eq!(ui.take_events(), vec![UiEvent::Action(ids::LANG_TR)]);
}

// ───────────────────────── profile editor surface ─────────────────────────

fn editor(w: u32, h: u32, dpi: f32) -> Ui {
    let mut ui = Ui::new();
    ui.set_size(w, h, dpi);
    ui.model = UiModel {
        surface: gxoled_ui::model::Surface::Editor,
        editor_title: "matrix-001.json — matrix".into(),
        editor_status: "JSON is valid".into(),
        ..UiModel::default()
    };
    ui.invalidate();
    ui.frame().expect("frame");
    ui
}

#[test]
fn editor_reserves_a_text_area_and_five_actions() {
    for (w, h, dpi) in [(930, 680, 1.0), (860, 540, 1.0), (1395, 1020, 1.5), (1860, 1360, 2.0)] {
        let ui = editor(w, h, dpi);
        let (x, y, tw, th) = ui.editor_text_rect_px().expect("text rect");
        assert!(x > 0 && y > 0 && tw > 300 && th > 120, "text rect {x},{y} {tw}x{th} at {w}x{h}");
        assert!(x + tw < w as i32 && y + th < h as i32);
        for id in [ids::EDITOR_SAVE, ids::EDITOR_FORMAT, ids::EDITOR_IMPORT, ids::EDITOR_EXPORT, ids::EDITOR_RESET] {
            let hit = find(&ui, id);
            let s = dpi;
            assert!(hit.rect.right() * s <= w as f32 + 0.5, "button {id} sticks out at {w}x{h}");
            assert!(hit.rect.y * s >= (y + th) as f32, "button {id} overlaps the text area");
        }
    }
}

#[test]
fn editor_buttons_report_actions_and_escape_closes() {
    let mut ui = editor(930, 680, 1.0);
    for id in [ids::EDITOR_SAVE, ids::EDITOR_RESET, ids::EDITOR_FORMAT] {
        // The editor is not width-proportional: one design unit is one pixel at 100 % DPI.
        let hit = find(&ui, id);
        let (x, y) = (hit.rect.cx(), hit.rect.cy());
        ui.mouse_down(x, y);
        ui.mouse_up(x, y);
        assert_eq!(ui.take_events(), vec![UiEvent::Action(id)]);
    }
    ui.key_down(Key::Escape);
    assert_eq!(ui.take_events(), vec![UiEvent::Close]);
}

#[test]
fn editor_renders_in_both_languages_and_themes_with_long_status() {
    let mut ui = Ui::new();
    ui.set_size(930, 680, 1.0);
    for lang in [Lang::En, Lang::Tr] {
        for dark in [true, false] {
            ui.model = UiModel {
                surface: gxoled_ui::model::Surface::Editor,
                lang,
                dark,
                editor_title: "a-very-long-profile-name-that-needs-to-be-shortened-with-an-ellipsis-because-it-is-long.json".into(),
                editor_status: "Invalid JSON: ".to_owned() + &"x".repeat(400),
                editor_status_ok: false,
                ..UiModel::default()
            };
            ui.invalidate();
            let f = ui.frame().expect("frame");
            assert_eq!((f.width, f.height), (930, 680));
        }
    }
}

// ───────────────────────── GIF Studio ─────────────────────────

fn studio_model(lang: Lang, dark: bool) -> UiModel {
    // (min, max, value) in the order of `gxoled_studio::Param::ALL`.
    let spec: [(i32, i32, i32); ids::STUDIO_SLIDER_COUNT] =
        [(20, 300, 100), (-64, 64, 0), (-64, 64, 0), (-100, 100, 0), (-100, 100, 0), (10, 300, 100), (0, 255, 128), (10, 500, 100), (10, 200, 30), (0, 23, 0), (0, 23, 23)];
    let studio = StudioModel {
        loaded: true,
        name: "a-very-long-file-name-that-would-not-fit-on-one-line-in-any-reasonable-window-at-all.gif".into(),
        info: "96×64 · 24 frames · 2.4 s".into(),
        sliders: spec
            .iter()
            .enumerate()
            .map(|(i, (min, max, value))| StudioSlider { id: ids::STUDIO_SLIDER_BASE + i as u32, min: *min, max: *max, value: *value, text: value.to_string() })
            .collect(),
        source: Some(Image { w: 2, h: 1, rgba: vec![255, 0, 0, 255, 0, 0, 255, 255] }),
        mono: (0..128 * 64).map(|i| i % 2 == 0).collect(),
        mono_w: 128,
        mono_h: 64,
        status: "Exported a file with a long name to a folder with an even longer name than that".into(),
        playing: true,
        status_ok: false,
    };
    UiModel {
        lang,
        dark,
        studio,
        checked: vec![ids::STUDIO_FIT_CONTAIN, ids::STUDIO_ROTATE_BASE, ids::STUDIO_BG_BLACK, ids::STUDIO_DITHER_BASE, ids::STUDIO_LOOP],
        ..model(Page::GifStudio)
    }
}

fn studio_ui(w: u32, h: u32, dpi: f32) -> Ui {
    let mut ui = Ui::new();
    ui.set_size(w, h, dpi);
    ui.model = studio_model(Lang::En, true);
    ui.invalidate();
    ui.frame().expect("frame");
    ui
}

#[test]
fn studio_controls_appear_only_with_a_gif() {
    let empty = ui(Page::GifStudio, W, H, 1.0);
    let ids_without: Vec<u32> = empty.hits().iter().map(|h| h.id).collect();
    assert!(ids_without.contains(&ids::GIF_ADD) && ids_without.contains(&ids::STUDIO_OPEN));
    assert!(!ids_without.iter().any(|id| (ids::STUDIO_SLIDER_BASE..ids::STUDIO_SLIDER_BASE + 50).contains(id)));
    assert!(!ids_without.contains(&ids::STUDIO_RESET));

    let full = studio_ui(W, H, 1.0);
    let present: Vec<u32> = full.hits().iter().map(|h| h.id).collect();
    for i in 0..ids::STUDIO_SLIDER_COUNT as u32 {
        assert!(present.contains(&(ids::STUDIO_SLIDER_BASE + i)), "slider {i}");
    }
    for id in [
        ids::STUDIO_FIT_CONTAIN, ids::STUDIO_FIT_COVER, ids::STUDIO_FIT_STRETCH, ids::STUDIO_FLIP_H, ids::STUDIO_FLIP_V,
        ids::STUDIO_BG_BLACK, ids::STUDIO_BG_WHITE, ids::STUDIO_INPUT_INVERT, ids::STUDIO_NORMALIZE, ids::STUDIO_INVERT,
        ids::STUDIO_LOOP, ids::STUDIO_RESET, ids::STUDIO_SAVE, ids::STUDIO_EXPORT, ids::STUDIO_OPEN, ids::GIF_TOGGLE,
        ids::STUDIO_PREVIEW_TOGGLE,
    ] {
        assert!(present.contains(&id), "control {id}");
    }
    for i in 0..4 {
        assert!(present.contains(&(ids::STUDIO_ROTATE_BASE + i)), "rotate {i}");
    }
    for i in 0..5 {
        assert!(present.contains(&(ids::STUDIO_DITHER_BASE + i)), "dither {i}");
    }
}

#[test]
fn studio_ids_are_unique_and_do_not_collide_with_other_pages() {
    let ui = studio_ui(W, H, 1.0);
    let mut seen = std::collections::HashSet::new();
    for h in ui.hits() {
        assert!(seen.insert(h.id), "duplicate id {}", h.id);
    }
}

#[test]
fn studio_page_renders_everywhere_and_stays_inside_the_window() {
    for (w, h, dpi) in [(W, H, 1.0), (1015, 682, 1.0), (800, 538, 1.0), (1860, 1260, 1.5)] {
        for lang in [Lang::En, Lang::Tr] {
            for dark in [true, false] {
                let mut ui = Ui::new();
                ui.set_size(w, h, dpi);
                ui.model = studio_model(lang, dark);
                ui.invalidate();
                assert!(ui.frame().is_some());
                let design_w = w as f32 / Ui::scale_for(w, dpi);
                for hit in ui.hits() {
                    let r = hit.rect;
                    assert!(r.x.is_finite() && r.y.is_finite() && r.w > 0.0 && r.h > 0.0, "{w}x{h} {}", hit.id);
                    assert!(r.x >= -0.5 && r.right() <= design_w + 0.5, "{w}x{h} id {} spills out: {r:?}", hit.id);
                }
            }
        }
    }
}

#[test]
fn studio_slider_drag_reports_the_value_and_clamps_to_its_range() {
    let mut ui = studio_ui(W, 2600, 1.0);
    let id = ids::STUDIO_SLIDER_BASE + 3; // brightness -100..=100
    let hit = find(&ui, id);
    let HitKind::Slider { track, min, max } = hit.kind else { panic!("not a slider") };
    assert_eq!((min, max), (-100, 100));
    let s = Ui::scale_for(W, 1.0);
    let y = (track.cy() - ui.scroll_offset()) * s;
    ui.mouse_down((track.x + track.w * 0.75) * s, y);
    let events = ui.take_events();
    let Some(UiEvent::Slider(got_id, v)) = events.last() else { panic!("no slider event: {events:?}") };
    assert_eq!(*got_id, id);
    assert!((48..=52).contains(v), "75% of -100..100 is about 50, got {v}");
    assert_eq!(ui.model.studio.slider(id).unwrap().value, *v);
    ui.mouse_move(0.0, y);
    assert!(matches!(ui.take_events().last(), Some(UiEvent::Slider(i, -100)) if *i == id));
    ui.mouse_move(W as f32 * s, y);
    assert!(matches!(ui.take_events().last(), Some(UiEvent::Slider(i, 100)) if *i == id));
    ui.mouse_up(0.0, 0.0);
    assert!(ui.take_events().is_empty(), "releasing a drag is not a click");
    // The brightness slider of the Display page is a different control and must not have moved.
    assert_eq!(ui.model.brightness, Some(7));
}

#[test]
fn studio_slider_keys_step_and_percent_ranges_move_faster() {
    let mut ui = studio_ui(W, 2600, 1.0);
    for (index, expect_step) in [(1usize, 1), (0usize, 2)] {
        let id = ids::STUDIO_SLIDER_BASE + index as u32;
        let hit = find(&ui, id);
        let (x, y) = centre(&ui, &hit, W, 1.0);
        ui.mouse_down(x, y);
        ui.mouse_up(x, y);
        ui.take_events();
        let before = ui.model.studio.slider(id).unwrap().value;
        ui.key_down(Key::Right);
        assert_eq!(ui.take_events(), vec![UiEvent::Slider(id, before + expect_step)], "slider {index}");
        ui.key_down(Key::Left);
        assert_eq!(ui.model.studio.slider(id).unwrap().value, before, "slider {index}");
    }
}

#[test]
fn studio_chips_report_actions() {
    let mut ui = studio_ui(W, 2600, 1.0);
    click(&mut ui, ids::STUDIO_DITHER_BASE + 1);
    assert_eq!(ui.take_events().last(), Some(&UiEvent::Action(ids::STUDIO_DITHER_BASE + 1)));
    click(&mut ui, ids::STUDIO_RESET);
    assert_eq!(ui.take_events().last(), Some(&UiEvent::Action(ids::STUDIO_RESET)));
}

#[test]
fn studio_preview_pixels_reach_the_frame_and_refresh_without_a_full_render() {
    let mut ui = studio_ui(W, 2600, 1.0);
    let before = ui.frame().unwrap().rgba.to_vec();
    // Flip every OLED pixel of the result pane; only the overlay may change.
    ui.model.studio.mono = (0..128 * 64).map(|i| i % 2 != 0).collect();
    ui.refresh_overlays();
    let after = ui.frame().unwrap().rgba.to_vec();
    assert_ne!(before, after, "the 1-bit pane did not update");
    assert_eq!(before.len(), after.len());
    // The source pane shows the two-colour sample (red | blue) somewhere in the frame.
    let has = |rgba: &[u8], px: [u8; 3]| rgba.chunks_exact(4).any(|c| c[..3] == px);
    assert!(has(&after, [255, 0, 0]) && has(&after, [0, 0, 255]));
}

#[test]
fn studio_preview_button_reports_the_action_and_names_what_it_will_do() {
    let mut ui = studio_ui(W, 2600, 1.0);
    // Playing: the button offers to pause; the action id is the studio's own (no menu event).
    assert!(ids::is_studio_action(ids::STUDIO_PREVIEW_TOGGLE));
    let playing = ui.frame().unwrap().rgba.to_vec();
    click(&mut ui, ids::STUDIO_PREVIEW_TOGGLE);
    assert_eq!(ui.take_events().last(), Some(&UiEvent::Action(ids::STUDIO_PREVIEW_TOGGLE)));
    // Paused: the same button now offers to play (its icon and caption change, so the picture differs).
    ui.model.studio.playing = false;
    ui.invalidate();
    let paused = ui.frame().unwrap().rgba.to_vec();
    assert_ne!(playing, paused, "the button looks the same while paused");
    // It exists in both languages and both themes and keeps inside the card on every size.
    for (w, h) in [(W, 2600), (800, 2800)] {
        for lang in [Lang::En, Lang::Tr] {
            for playing in [true, false] {
                let mut ui = Ui::new();
                ui.set_size(w, h, 1.0);
                ui.model = studio_model(lang, true);
                ui.model.studio.playing = playing;
                ui.invalidate();
                ui.frame().expect("frame");
                let hit = find(&ui, ids::STUDIO_PREVIEW_TOGGLE);
                assert!(hit.rect.h >= 30.0 && hit.rect.w >= 100.0, "{w} {lang:?} {playing}: {:?}", hit.rect);
            }
        }
    }
}

#[test]
fn releasing_the_buffers_frees_the_frame_and_the_next_frame_is_identical() {
    let mut ui = studio_ui(W, H, 1.0);
    let before = ui.frame().unwrap().rgba.to_vec();
    assert!(ui.current_frame().is_some());
    ui.release_buffers();
    assert!(ui.current_frame().is_none(), "the composited frame must be gone");
    assert!(ui.needs_render(), "the next paint has to render again");
    // Nothing may touch the missing buffers while the window is hidden.
    ui.refresh_overlays();
    assert!(ui.current_frame().is_none());
    let after = ui.frame().unwrap().rgba.to_vec();
    assert_eq!(before, after, "the page looks the same after it was rendered again");
}
