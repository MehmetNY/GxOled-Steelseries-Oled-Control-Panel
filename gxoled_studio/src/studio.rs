//! A loaded GIF plus the settings it is processed with, and a cache of the 1-bit pictures.

use crate::{
    export::encode_gif,
    plan::{Step, playback_plan},
    process::render_frame,
    project::Project,
    settings::Settings,
    source::Source,
};
use gxoled_lib::Bitmap;
use std::{path::PathBuf, sync::Arc};

pub struct Studio {
    path: PathBuf,
    source: Source,
    settings: Settings,
    target: (usize, usize),
    cache: Vec<Option<Arc<Bitmap>>>,
}

impl Studio {
    pub fn open(path: PathBuf, settings: Settings, target: (usize, usize)) -> anyhow::Result<Studio> {
        let source = Source::load(&path)?;
        Ok(Studio::from_source(path, source, settings, target))
    }

    pub fn from_source(path: PathBuf, source: Source, settings: Settings, target: (usize, usize)) -> Studio {
        let cache = vec![None; source.frames.len()];
        Studio { path, source, settings: settings.clamped(), target, cache }
    }

    pub fn from_project(project: &Project, target: (usize, usize)) -> anyhow::Result<Studio> {
        Studio::open(project.source.clone(), project.settings, target)
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn frame_count(&self) -> usize {
        self.source.frames.len()
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn target(&self) -> (usize, usize) {
        self.target
    }

    pub fn project(&self) -> Project {
        Project::new(self.path.clone(), self.settings)
    }

    pub fn set_target(&mut self, target: (usize, usize)) {
        if self.target != target {
            self.target = target;
            self.cache.iter_mut().for_each(|c| *c = None);
        }
    }

    /// Changes the settings; the picture cache is dropped only when the look (not just the playback) changes.
    pub fn update(&mut self, change: impl FnOnce(&mut Settings)) {
        let before = self.settings;
        let mut next = before;
        change(&mut next);
        let next = next.clamped();
        if !before.same_look(&next) {
            self.cache.iter_mut().for_each(|c| *c = None);
        }
        self.settings = next;
    }

    pub fn replace_settings(&mut self, settings: Settings) {
        self.update(|s| *s = settings);
    }

    /// 1-bit picture of source frame `index` with the current settings.
    pub fn frame(&mut self, index: usize) -> Option<Arc<Bitmap>> {
        let slot = self.cache.get_mut(index)?;
        if let Some(b) = slot {
            return Some(b.clone());
        }
        let f = &self.source.frames[index];
        let bitmap = Arc::new(render_frame(&f.rgba, self.source.w, self.source.h, &self.settings, self.target.0, self.target.1));
        *slot = Some(bitmap.clone());
        Some(bitmap)
    }

    pub fn plan(&self) -> Vec<Step> {
        let delays: Vec<u32> = self.source.frames.iter().map(|f| f.delay_ms).collect();
        playback_plan(&delays, &self.settings)
    }

    /// Every frame that plays, in order, with its on-screen time.
    pub fn render_all(&mut self) -> Vec<(Arc<Bitmap>, u32)> {
        let plan = self.plan();
        plan.into_iter().filter_map(|step| self.frame(step.index).map(|b| (b, step.delay_ms))).collect()
    }

    /// The finished animation as a two-colour GIF.
    pub fn export_gif(&mut self) -> anyhow::Result<Vec<u8>> {
        let frames = self.render_all();
        let refs: Vec<(&Bitmap, u32)> = frames.iter().map(|(b, d)| (b.as_ref(), *d)).collect();
        encode_gif(&refs, self.settings.looping)
    }

    /// Which step of the plan is on screen `elapsed_ms` after the start (wraps when looping, holds the last
    /// frame otherwise). `None` for an empty plan.
    pub fn step_at(plan: &[Step], elapsed_ms: u64, looping: bool) -> Option<usize> {
        let total: u64 = plan.iter().map(|s| s.delay_ms as u64).sum();
        if plan.is_empty() || total == 0 {
            return None;
        }
        let t = if looping { elapsed_ms % total } else { elapsed_ms.min(total - 1) };
        let mut acc = 0u64;
        for (i, s) in plan.iter().enumerate() {
            acc += s.delay_ms as u64;
            if t < acc {
                return Some(i);
            }
        }
        Some(plan.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::Dither, source::testing::gif_bytes};

    fn studio(settings: Settings) -> Studio {
        let bytes = gif_bytes(16, 8, &[([255, 255, 255], 5), ([0, 0, 0], 10), ([255, 255, 255], 20)]);
        Studio::from_source(PathBuf::from("x.gif"), Source::decode(&bytes).unwrap(), settings, (16, 8))
    }

    fn lit(b: &Bitmap) -> usize {
        b.data.iter().filter(|v| *v).count()
    }

    #[test]
    fn renders_each_frame_with_the_current_settings() {
        let mut s = studio(Settings::default());
        assert_eq!(lit(&s.frame(0).unwrap()), 128);
        assert_eq!(lit(&s.frame(1).unwrap()), 0);
        assert!(s.frame(3).is_none());
        s.update(|x| x.invert = true);
        assert_eq!(lit(&s.frame(0).unwrap()), 0);
        assert_eq!(lit(&s.frame(1).unwrap()), 128);
    }

    #[test]
    fn frames_are_cached_until_the_look_changes() {
        let mut s = studio(Settings::default());
        let a = s.frame(0).unwrap();
        assert!(Arc::ptr_eq(&a, &s.frame(0).unwrap()));
        s.update(|x| x.speed = 2.0);
        assert!(Arc::ptr_eq(&a, &s.frame(0).unwrap()), "speed does not change the picture");
        s.update(|x| x.threshold = 10);
        assert!(!Arc::ptr_eq(&a, &s.frame(0).unwrap()), "threshold does");
        let b = s.frame(0).unwrap();
        s.set_target((32, 16));
        assert_eq!(s.frame(0).unwrap().w, 32);
        assert!(!Arc::ptr_eq(&b, &s.frame(0).unwrap()));
    }

    #[test]
    fn update_clamps_its_input() {
        let mut s = studio(Settings::default());
        s.update(|x| {
            x.zoom = 99.0;
            x.rotate = 135;
        });
        assert_eq!((s.settings().zoom, s.settings().rotate), (3.0, 90));
    }

    #[test]
    fn plan_and_render_all_follow_trim_and_speed() {
        let mut s = studio(Settings { trim_start: 1, speed: 2.0, ..Settings::default() });
        let all = s.render_all();
        assert_eq!(all.iter().map(|(_, d)| *d).collect::<Vec<_>>(), [50, 100]);
        assert_eq!(lit(&all[0].0), 0);
        assert_eq!(lit(&all[1].0), 128);
    }

    #[test]
    fn export_produces_a_gif_that_loads_back() {
        let mut s = studio(Settings { dither: Dither::Threshold, ..Settings::default() });
        let bytes = s.export_gif().unwrap();
        let back = Source::decode(&bytes).unwrap();
        assert_eq!((back.w, back.h, back.frames.len()), (16, 8, 3));
        assert_eq!(back.frames.iter().map(|f| f.delay_ms).collect::<Vec<_>>(), [50, 100, 200]);
    }

    #[test]
    fn project_round_trip_keeps_path_and_settings() {
        let s = studio(Settings { threshold: 77, ..Settings::default() });
        let p = s.project();
        assert_eq!(p.source, PathBuf::from("x.gif"));
        assert_eq!(p.settings.threshold, 77);
    }

    #[test]
    fn step_at_wraps_or_holds() {
        let plan = vec![Step { index: 0, delay_ms: 100 }, Step { index: 1, delay_ms: 50 }];
        assert_eq!(Studio::step_at(&plan, 0, true), Some(0));
        assert_eq!(Studio::step_at(&plan, 99, true), Some(0));
        assert_eq!(Studio::step_at(&plan, 100, true), Some(1));
        assert_eq!(Studio::step_at(&plan, 150, true), Some(0), "wraps");
        assert_eq!(Studio::step_at(&plan, 10_000, false), Some(1), "holds the last frame");
        assert_eq!(Studio::step_at(&[], 5, true), None);
    }

    #[test]
    fn opens_files_from_disk() {
        let dir = std::env::temp_dir().join(format!("gxoled_studio_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.gif");
        std::fs::write(&path, gif_bytes(4, 4, &[([255, 255, 255], 5)])).unwrap();
        let mut s = Studio::open(path.clone(), Settings::default(), (8, 8)).unwrap();
        assert_eq!(s.frame_count(), 1);
        assert_eq!(s.path(), path);
        assert_eq!(lit(&s.frame(0).unwrap()), 64);
        assert!(Studio::open(dir.join("nope.gif"), Settings::default(), (8, 8)).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
