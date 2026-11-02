//! Binds the GIF Studio page to the engine's settings: which chips are checked, what a chip or slider does,
//! and the slider models. Pure functions, so the whole mapping is unit-tested without a window.

use crate::{
    ids,
    model::{Lang, StudioModel, StudioSlider},
};
use gxoled_studio::{Background, Dither, Fit, Param, Settings, Studio};

const ROTATIONS: [u16; 4] = [0, 90, 180, 270];

/// Action ids of every chip / toggle that is "on" for `s`.
pub fn checked_ids(s: &Settings) -> Vec<u32> {
    let mut v = vec![
        match s.fit {
            Fit::Contain => ids::STUDIO_FIT_CONTAIN,
            Fit::Cover => ids::STUDIO_FIT_COVER,
            Fit::Stretch => ids::STUDIO_FIT_STRETCH,
        },
        ids::STUDIO_ROTATE_BASE + ROTATIONS.iter().position(|r| *r == s.rotate).unwrap_or(0) as u32,
        match s.background {
            Background::Black => ids::STUDIO_BG_BLACK,
            Background::White => ids::STUDIO_BG_WHITE,
        },
        ids::STUDIO_DITHER_BASE + Dither::ALL.iter().position(|d| *d == s.dither).unwrap_or(0) as u32,
    ];
    for (on, id) in [
        (s.flip_h, ids::STUDIO_FLIP_H),
        (s.flip_v, ids::STUDIO_FLIP_V),
        (s.input_invert, ids::STUDIO_INPUT_INVERT),
        (s.normalize, ids::STUDIO_NORMALIZE),
        (s.invert, ids::STUDIO_INVERT),
        (s.looping, ids::STUDIO_LOOP),
    ] {
        if on {
            v.push(id);
        }
    }
    v
}

/// Applies a chip / toggle. Returns `false` for ids that are not settings (buttons, unknown ids).
pub fn apply_action(s: &mut Settings, id: u32) -> bool {
    match id {
        ids::STUDIO_FIT_CONTAIN => s.fit = Fit::Contain,
        ids::STUDIO_FIT_COVER => s.fit = Fit::Cover,
        ids::STUDIO_FIT_STRETCH => s.fit = Fit::Stretch,
        id if (ids::STUDIO_ROTATE_BASE..ids::STUDIO_ROTATE_BASE + 4).contains(&id) => s.rotate = ROTATIONS[(id - ids::STUDIO_ROTATE_BASE) as usize],
        ids::STUDIO_FLIP_H => s.flip_h = !s.flip_h,
        ids::STUDIO_FLIP_V => s.flip_v = !s.flip_v,
        ids::STUDIO_BG_BLACK => s.background = Background::Black,
        ids::STUDIO_BG_WHITE => s.background = Background::White,
        ids::STUDIO_INPUT_INVERT => s.input_invert = !s.input_invert,
        ids::STUDIO_NORMALIZE => s.normalize = !s.normalize,
        ids::STUDIO_INVERT => s.invert = !s.invert,
        id if (ids::STUDIO_DITHER_BASE..ids::STUDIO_DITHER_BASE + Dither::ALL.len() as u32).contains(&id) => {
            s.dither = Dither::ALL[(id - ids::STUDIO_DITHER_BASE) as usize]
        }
        ids::STUDIO_LOOP => s.looping = !s.looping,
        _ => return false,
    }
    true
}

/// Slider models in `ids::STUDIO_SLIDER_BASE` order (the order of `Param::ALL`).
pub fn slider_models(s: &Settings, frames: usize) -> Vec<StudioSlider> {
    Param::ALL
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let (min, max) = p.range(frames);
            let value = p.get(s, frames).clamp(min, max.max(min));
            StudioSlider { id: ids::STUDIO_SLIDER_BASE + i as u32, min, max: max.max(min), value, text: p.format(value) }
        })
        .collect()
}

/// Applies a slider move. Returns `false` when `id` is not a studio slider.
pub fn apply_slider(s: &mut Settings, id: u32, value: i32, frames: usize) -> bool {
    let Some(index) = id.checked_sub(ids::STUDIO_SLIDER_BASE).map(|i| i as usize) else { return false };
    let Some(param) = Param::ALL.get(index) else { return false };
    param.set(s, value, frames);
    true
}

/// "96×64 · 24 frames · 2.4 s"
pub fn describe(w: usize, h: usize, frames: usize, duration_ms: u64, lang: Lang) -> String {
    let secs = duration_ms as f32 / 1000.0;
    let secs = if lang == Lang::Tr { format!("{secs:.1}").replace('.', ",") } else { format!("{secs:.1}") };
    format!("{w}×{h} · {frames} {} · {secs} {}", lang.t("frames", "kare"), lang.t("s", "sn"))
}

/// Everything of the editor page except the animated previews (those are pushed separately and often).
pub fn model(studio: &Studio, lang: Lang, status: Option<(&str, bool)>) -> StudioModel {
    let s = studio.source();
    StudioModel {
        loaded: true,
        name: studio.path().file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        info: describe(s.w, s.h, studio.frame_count(), s.duration_ms(), lang),
        sliders: slider_models(studio.settings(), studio.frame_count()),
        status: status.map(|(t, _)| t.to_owned()).unwrap_or_default(),
        status_ok: status.is_none_or(|(_, ok)| ok),
        ..StudioModel::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slider_table_matches_the_engine() {
        assert_eq!(ids::STUDIO_SLIDER_COUNT, Param::ALL.len());
        let m = slider_models(&Settings::default(), 10);
        assert_eq!(m.len(), ids::STUDIO_SLIDER_COUNT);
        for (i, s) in m.iter().enumerate() {
            assert_eq!(s.id, ids::STUDIO_SLIDER_BASE + i as u32);
            assert!(s.min <= s.value && s.value <= s.max, "{i}: {s:?}");
            assert!(!s.text.is_empty());
        }
        // Defaults read as the neutral values.
        assert_eq!(m[0].text, "1.00×");
        assert_eq!(m[6].text, "128");
        assert_eq!(m[10].text, "#10", "the last frame of ten");
    }

    #[test]
    fn every_slider_id_reaches_its_setting() {
        let frames = 12;
        for (i, p) in Param::ALL.iter().enumerate() {
            let mut s = Settings::default();
            let (lo, hi) = p.range(frames);
            let id = ids::STUDIO_SLIDER_BASE + i as u32;
            assert!(apply_slider(&mut s, id, hi, frames));
            assert_eq!(p.get(&s, frames), hi, "{p:?}");
            assert!(apply_slider(&mut s, id, lo, frames));
            assert_eq!(p.get(&s, frames), lo, "{p:?}");
        }
        let mut s = Settings::default();
        assert!(!apply_slider(&mut s, ids::STUDIO_SLIDER_BASE + 99, 5, frames));
        assert!(!apply_slider(&mut s, ids::BRIGHTNESS, 5, frames));
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn radio_chips_set_exactly_one_value_and_show_as_checked() {
        let mut s = Settings::default();
        assert!(checked_ids(&s).contains(&ids::STUDIO_FIT_CONTAIN));
        for (id, check) in [
            (ids::STUDIO_FIT_COVER, ids::STUDIO_FIT_COVER),
            (ids::STUDIO_FIT_STRETCH, ids::STUDIO_FIT_STRETCH),
            (ids::STUDIO_ROTATE_BASE + 2, ids::STUDIO_ROTATE_BASE + 2),
            (ids::STUDIO_BG_WHITE, ids::STUDIO_BG_WHITE),
            (ids::STUDIO_DITHER_BASE + 4, ids::STUDIO_DITHER_BASE + 4),
        ] {
            assert!(apply_action(&mut s, id));
            let c = checked_ids(&s);
            assert!(c.contains(&check), "{id}");
            // Re-applying keeps the same state (radios are idempotent).
            apply_action(&mut s, id);
            assert_eq!(c, checked_ids(&s));
        }
        assert_eq!((s.fit, s.rotate, s.background, s.dither), (Fit::Stretch, 180, Background::White, Dither::None));
        // Exactly one chip per radio group is checked.
        let c = checked_ids(&s);
        assert_eq!(c.iter().filter(|i| (ids::STUDIO_FIT_CONTAIN..=ids::STUDIO_FIT_STRETCH).contains(i)).count(), 1);
        assert_eq!(c.iter().filter(|i| (ids::STUDIO_ROTATE_BASE..ids::STUDIO_ROTATE_BASE + 4).contains(i)).count(), 1);
        assert_eq!(c.iter().filter(|i| (ids::STUDIO_DITHER_BASE..ids::STUDIO_DITHER_BASE + 5).contains(i)).count(), 1);
    }

    #[test]
    fn toggles_flip_and_flip_back() {
        let mut s = Settings { looping: false, ..Settings::default() };
        for id in [ids::STUDIO_FLIP_H, ids::STUDIO_FLIP_V, ids::STUDIO_INPUT_INVERT, ids::STUDIO_NORMALIZE, ids::STUDIO_INVERT, ids::STUDIO_LOOP] {
            assert!(!checked_ids(&s).contains(&id), "{id} starts off");
            assert!(apply_action(&mut s, id));
            assert!(checked_ids(&s).contains(&id), "{id} on");
            assert!(apply_action(&mut s, id));
            assert!(!checked_ids(&s).contains(&id), "{id} off again");
        }
        assert_eq!(s, Settings { looping: false, ..Settings::default() });
    }

    #[test]
    fn buttons_and_foreign_ids_are_not_settings() {
        let mut s = Settings::default();
        for id in [ids::STUDIO_OPEN, ids::STUDIO_SAVE, ids::STUDIO_EXPORT, ids::STUDIO_RESET, ids::GIF_ADD, ids::BRIGHTNESS, 0] {
            assert!(!apply_action(&mut s, id), "{id}");
        }
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn every_chip_of_the_page_is_handled() {
        // The ids the page draws must all be accepted, otherwise a click would silently do nothing.
        let mut s = Settings::default();
        let chips = [
            ids::STUDIO_FIT_CONTAIN, ids::STUDIO_FIT_COVER, ids::STUDIO_FIT_STRETCH, ids::STUDIO_FLIP_H, ids::STUDIO_FLIP_V,
            ids::STUDIO_BG_BLACK, ids::STUDIO_BG_WHITE, ids::STUDIO_INPUT_INVERT, ids::STUDIO_NORMALIZE, ids::STUDIO_INVERT, ids::STUDIO_LOOP,
        ];
        for id in chips.into_iter().chain((0..4).map(|i| ids::STUDIO_ROTATE_BASE + i)).chain((0..5).map(|i| ids::STUDIO_DITHER_BASE + i)) {
            assert!(apply_action(&mut s, id), "{id}");
        }
    }

    #[test]
    fn the_description_is_localised() {
        assert_eq!(describe(96, 64, 24, 2400, Lang::En), "96×64 · 24 frames · 2.4 s");
        assert_eq!(describe(96, 64, 24, 2400, Lang::Tr), "96×64 · 24 kare · 2,4 sn");
    }

    #[test]
    fn the_page_model_describes_the_open_gif() {
        use gxoled_studio::{Source, Studio};
        let mut gif = Vec::new();
        {
            let mut enc = gif::Encoder::new(&mut gif, 4, 2, &[]).unwrap();
            for _ in 0..3 {
                let mut f = gif::Frame::from_palette_pixels(4, 2, vec![0u8; 8], vec![255, 255, 255], None);
                f.delay = 10;
                enc.write_frame(&f).unwrap();
            }
        }
        let studio = Studio::from_source("C:/gifs/cat.gif".into(), Source::decode(&gif).unwrap(), Settings::default(), (128, 64));
        let m = model(&studio, Lang::En, Some(("Saved", true)));
        assert!(m.loaded);
        assert_eq!(m.name, "cat.gif");
        assert_eq!(m.info, "4×2 · 3 frames · 0.3 s");
        assert_eq!(m.sliders.len(), ids::STUDIO_SLIDER_COUNT);
        assert_eq!((m.status.as_str(), m.status_ok), ("Saved", true));
        let bad = model(&studio, Lang::Tr, Some(("Hata", false)));
        assert!(!bad.status_ok);
        assert_eq!(model(&studio, Lang::En, None).status, "");
    }
}
