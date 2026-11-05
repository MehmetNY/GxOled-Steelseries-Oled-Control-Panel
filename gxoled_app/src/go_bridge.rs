//! The Go code is an internal renderer and never opens the SteelSeries HID device.
//! The child is launched only for a selected SteelClock animation and is killed on Drop.
#![cfg(target_os = "windows")]

use anyhow::{Context, Result};
use gxoled_lib::Bitmap;
use std::{io::Read, process::{Child, Command, Stdio}, sync::{Arc, Mutex, mpsc::{self, Receiver}}, thread::{self, JoinHandle}, path::PathBuf};
use std::os::windows::process::CommandExt;

const RENDERER: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../go_animations/gxoled-animations.exe"));
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct Runner {
    child: Child,
    reader: Option<JoinHandle<()>>,
    /// Drains the helper's diagnostics (a full pipe would block it) and keeps the first line.
    diagnostics: Option<JoinHandle<()>>,
    first_diagnostic: Arc<Mutex<String>>,
    pub frames: Receiver<Arc<Bitmap>>,
}
fn executable() -> Result<PathBuf> {
    let base = directories::BaseDirs::new().context("GxOled settings directory unavailable")?;
    let folder = base.cache_dir().join("GxOled").join("renderer");
    std::fs::create_dir_all(&folder)?;
    let dest = folder.join("gxoled-animations.exe");
    let outdated = std::fs::read(&dest).map(|old| old.as_slice() != RENDERER).unwrap_or(true);
    if outdated {
        // A selected renderer is always stopped before another one is launched.
        let temporary = folder.join("gxoled-animations.new");
        std::fs::write(&temporary, RENDERER)?;
        if dest.exists() { std::fs::remove_file(&dest)?; }
        std::fs::rename(&temporary, &dest)?;
    }
    Ok(dest)
}
impl Runner {
    pub fn start(animation: &str, filename: Option<&str>) -> Result<Self> {
        let config = directories::BaseDirs::new().map(|base| {
            let profiles = base.config_dir().join("GxOled").join("profiles");
            // Never let filenames supplied by a profile file escape its dedicated folder.
            match filename.filter(|s| !s.is_empty() && s.ends_with(".json")
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
                && !s.contains("..")) {
                Some(name) => profiles.join("custom").join(name),
                None => profiles.join(format!("{animation}.json")),
            }
        });
        let mut command = Command::new(executable()?);
        command.args(["--animation", animation]);
        if let Some(path) = &config {if path.is_file() {command.arg("--profile-file").arg(path);}}
        let mut child = command
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn().with_context(||format!("Couldn't start Go animation '{animation}'"))?;
        let mut stdout = child.stdout.take().context("renderer stdout unavailable")?;
        let first_diagnostic = Arc::new(Mutex::new(String::new()));
        let diagnostics = child.stderr.take().map(|mut stderr| {
            let first = first_diagnostic.clone();
            thread::spawn(move || {
                let mut buf = [0u8; 512];
                // Read until the helper is gone; only the first non-empty line is kept ("panic: ..." or the
                // error of a rejected profile), the rest of a Go stack trace is of no use to the user.
                while let Ok(n) = stderr.read(&mut buf) {
                    if n == 0 { break; }
                    if let Ok(mut line) = first.lock() {
                        if line.is_empty() {
                            let text = String::from_utf8_lossy(&buf[..n]);
                            if let Some(found) = text.lines().map(str::trim).find(|l| !l.is_empty()) {
                                *line = found.chars().take(240).collect();
                            }
                        }
                    }
                }
            })
        });
        let (sender, frames)=mpsc::sync_channel(2);
        let reader=thread::spawn(move || {
            let mut buf=[0u8;1024];
            while stdout.read_exact(&mut buf).is_ok() {
                let mut bitmap=Bitmap::new(128,64,false);
                // The original SteelClock ImageToBytes uses MSB-first packed rows.
                for idx in 0..8192usize {
                    bitmap.data.set(idx, buf[idx/8] & (0x80 >> (idx%8)) != 0);
                }
                match sender.try_send(Arc::new(bitmap)) {
                    Ok(()) | Err(mpsc::TrySendError::Full(_)) => {},
                    Err(mpsc::TrySendError::Disconnected(_)) => break,
                }
            }
        });
        Ok(Self { child, reader:Some(reader), diagnostics, first_diagnostic, frames })
    }

    /// `false` once the helper has exited by itself: a profile it rejects or cannot handle, a crash, or a virus
    /// scanner that killed it. Nothing then produces frames any more, so the caller has to react.
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// The first line the helper wrote to its error output, if any (why it stopped).
    pub fn diagnostic(&self) -> Option<String> {
        self.first_diagnostic.lock().ok().map(|line| line.clone()).filter(|line| !line.is_empty())
    }
}
impl Drop for Runner {
    fn drop(&mut self) {
        let _=self.child.kill();
        let _=self.child.wait();
        if let Some(handle)=self.reader.take() { let _=handle.join(); }
        if let Some(handle)=self.diagnostics.take() { let _=handle.join(); }
    }
}
