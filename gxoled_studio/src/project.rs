//! Project files (`.gxgif.json`): which GIF, and how it is processed. The picture itself is not embedded.

use crate::settings::Settings;
use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "gxoled-gif-project";
pub const VERSION: u32 = 1;
pub const FILE_SUFFIX: &str = "gxgif.json";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub format: String,
    pub version: u32,
    pub source: PathBuf,
    pub settings: Settings,
}

impl Project {
    pub fn new(source: PathBuf, settings: Settings) -> Self {
        Project { format: FORMAT.to_owned(), version: VERSION, source, settings }
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<Project> {
        let mut project: Project = serde_json::from_str(json).context("This is not a GxOled GIF project file")?;
        if project.format != FORMAT {
            bail!("This is not a GxOled GIF project file");
        }
        if project.version > VERSION {
            bail!("This project was saved by a newer GxOled version");
        }
        if is_network_or_device_path(&project.source) {
            bail!(
                "The picture of this project is on a network share or a device path, which GxOled does not open from a project file. Open the GIF itself and save the project again."
            );
        }
        project.settings = project.settings.clamped();
        Ok(project)
    }
}

/// A project file can come from anywhere. The picture it names must not make GxOled reach for a network location:
/// opening `\\host\share\x.gif` makes Windows try to sign in to that computer. UNC paths (`\\host\share`, also
/// written `//host/share`) and device paths (`\\.\...`) are refused; a drive letter, a mapped network drive and the
/// long local form `\\?\C:\...` are not.
fn is_network_or_device_path(path: &Path) -> bool {
    let text = path.to_string_lossy().replace('/', "\\");
    match text.strip_prefix(r"\\?\") {
        Some(rest) => rest.get(..4).is_some_and(|start| start.eq_ignore_ascii_case(r"UNC\")),
        None => text.starts_with(r"\\"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Dither, Fit};

    #[test]
    fn a_project_cannot_point_at_a_network_or_device_path() {
        for source in [r"\\host\share\x.gif", "//host/share/x.gif", r"\\.\pipe\x", r"\\?\UNC\host\share\x.gif", r"\\?\unc\host\share\x.gif"] {
            assert!(is_network_or_device_path(Path::new(source)), "{source} should be refused");
            let json = format!(r#"{{"format":"gxoled-gif-project","version":1,"source":{},"settings":{{}}}}"#, serde_json::to_string(source).unwrap());
            let error = Project::from_json(&json).unwrap_err().to_string();
            assert!(error.contains("network share"), "{source}: {error}");
        }
        // Local paths, mapped drives, relative names and the long local form are fine.
        for source in [r"C:\gifs\cat.gif", r"Z:\nas\cat.gif", "cat.gif", r"gifs\cat.gif", "/home/me/cat.gif", r"\\?\C:\gifs\cat.gif", ""] {
            assert!(!is_network_or_device_path(Path::new(source)), "{source} should be allowed");
        }
    }

    #[test]
    fn round_trip() {
        let s = Settings { dither: Dither::Atkinson, fit: Fit::Cover, rotate: 270, flip_h: true, zoom: 1.5, trim_end: Some(7), ..Settings::default() };
        let p = Project::new(PathBuf::from("C:/gifs/cat.gif"), s);
        assert_eq!(Project::from_json(&p.to_json().unwrap()).unwrap(), p);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults_and_values_are_clamped() {
        let json = r#"{"format":"gxoled-gif-project","version":1,"source":"a.gif","settings":{"threshold":90,"zoom":50.0,"rotate":91}}"#;
        let p = Project::from_json(json).unwrap();
        assert_eq!(p.settings.threshold, 90);
        assert_eq!(p.settings.zoom, 3.0);
        assert_eq!(p.settings.rotate, 90);
        assert_eq!(p.settings.dither, Dither::FloydSteinberg);
    }

    #[test]
    fn foreign_and_future_files_are_refused() {
        assert!(Project::from_json("{}").is_err());
        assert!(Project::from_json("not json").is_err());
        let other = r#"{"format":"ggif-project","version":1,"source":"a.gif","settings":{}}"#;
        assert!(Project::from_json(other).is_err());
        let future = r#"{"format":"gxoled-gif-project","version":99,"source":"a.gif","settings":{}}"#;
        assert!(Project::from_json(future).unwrap_err().to_string().contains("newer"));
    }
}
