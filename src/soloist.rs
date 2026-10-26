//! Optional official playback process. Spotifast controls it as a Connect device.
//!
//! No Soloist binary is bundled. The user supplies Spotify's Windows build.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Deliberately not Debug: this contains the user's developer key.
#[derive(Serialize, Deserialize)]
pub struct Setup {
    pub executable: PathBuf,
    pub api_key: String,
    pub device_name: String,
}

impl Setup {
    pub fn validate(&self) -> Result<()> {
        if !self.executable.is_absolute() || !self.executable.is_file() {
            bail!("Choose the downloaded soloist.exe using its full path.");
        }
        if self.api_key.trim().is_empty() || self.device_name.trim().is_empty() {
            bail!("Enter your Soloist API key and a device name.");
        }
        Ok(())
    }
}

pub struct Process {
    child: Child,
}

impl Process {
    /// Runs on a background thread. Never capture or log Soloist's output:
    /// it may include account details. The key is required by Spotify's CLI.
    pub fn start(setup: &Setup, data_dir: &Path, cache_dir: &Path) -> Result<Self> {
        use std::os::windows::process::CommandExt;

        setup.validate()?;
        std::fs::create_dir_all(data_dir).context("Couldn't create Soloist's data folder")?;
        let child = command(setup, data_dir, cache_dir)
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
            .spawn()
            .context("Couldn't start Soloist. Check the executable and Windows permissions.")?;
        Ok(Self { child })
    }

    pub fn exited(&mut self) -> Result<Option<std::process::ExitStatus>> {
        self.child.try_wait().context("Couldn't check Soloist")
    }
}

fn command(setup: &Setup, data_dir: &Path, cache_dir: &Path) -> Command {
    let mut command = Command::new(&setup.executable);
    command
        .arg("--device-name")
        .arg(setup.device_name.trim())
        .arg("--api-key")
        .arg(setup.api_key.trim())
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--cache-dir")
        .arg(cache_dir)
        .arg("--cache-size")
        .arg("256")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

impl Drop for Process {
    fn drop(&mut self) {
        // Reap only the child this instance owns, never other Soloist processes.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct Form {
    pub open: bool,
    pub active: bool,
    executable: String,
    api_key: String,
    device_name: String,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            open: false,
            active: false,
            executable: String::new(),
            api_key: String::new(),
            device_name: "Spotifast Soloist".into(),
        }
    }
}

pub enum FormAction {
    Browse,
    Start(Option<Setup>),
    Forget,
}

impl Form {
    pub fn choose_executable(&mut self, path: PathBuf) {
        self.executable = path.to_string_lossy().into_owned();
    }
    /// Draw only; the caller applies the returned action after this view.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<FormAction> {
        let mut action = None;
        egui::Window::new("Soloist playback")
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_max_width(440.0);
                ui.label("Keep Spotifast's interface and play through Spotify's official Soloist engine.");
                ui.hyperlink_to("Download Soloist for Windows", "https://developer.spotify.com/documentation/soloist/reference/downloads-and-updates");
                ui.hyperlink_to("Generate your Soloist API key", "https://developer.spotify.com/dashboard/soloist");
                ui.label("Full path to soloist.exe (without quotation marks)");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.executable);
                    if ui.button("Browse...").clicked() {
                        action = Some(FormAction::Browse);
                    }
                });
                ui.label("Soloist API key");
                ui.add(egui::TextEdit::singleline(&mut self.api_key).password(true));
                ui.label("Device name");
                ui.text_edit_singleline(&mut self.device_name);
                ui.label("First use: select this device in an official Spotify app to pair it, then select it in Spotifast's device picker. Keep the same Spotify account in both apps.");
                ui.label("The equalizer, audio visualizers and local-file playback do not apply to Soloist. Soloist builds expire after 90 days; download a newer build when needed.");
                ui.horizontal(|ui| {
                    if ui.add_enabled(!self.executable.trim().is_empty() && !self.api_key.trim().is_empty() && !self.device_name.trim().is_empty(), egui::Button::new("Save and start")).clicked() {
                        action = Some(FormAction::Start(Some(Setup {
                            executable: PathBuf::from(self.executable.trim()),
                            api_key: std::mem::take(&mut self.api_key),
                            device_name: self.device_name.trim().into(),
                        })));
                    }
                    if ui.button("Start saved setup").clicked() {
                        action = Some(FormAction::Start(None));
                    }
                    if ui.button("Stop and forget setup").clicked() {
                        self.api_key.clear();
                        action = Some(FormAction::Forget);
                    }
                });
            });
        if !self.open {
            self.api_key.clear();
        }
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forgotten_setup_never_reads_an_old_key_from_credential_manager() {
        let state =
            std::env::temp_dir().join(format!("spotifast-soloist-revoked-{}", std::process::id()));
        std::fs::create_dir_all(&state).unwrap();
        let marker = state.join("soloist-disabled");
        std::fs::write(&marker, b"disabled").unwrap();
        assert!(crate::credentials::soloist_setup(&state).unwrap().is_none());
        std::fs::remove_file(marker).unwrap();
        std::fs::remove_dir(state).unwrap();
    }

    #[test]
    fn dropping_the_owned_process_stops_it() {
        use std::os::windows::io::{AsHandle, AsRawHandle};
        use std::os::windows::process::CommandExt;

        let child = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 60",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x0800_0000)
            .spawn()
            .unwrap();
        let observer = child.as_handle().try_clone_to_owned().unwrap();
        let process = Process { child };
        drop(process);
        // SAFETY: observer owns a valid process handle until after this wait.
        let result = unsafe {
            windows_sys::Win32::System::Threading::WaitForSingleObject(
                observer.as_raw_handle().cast(),
                2000,
            )
        };
        assert_eq!(result, windows_sys::Win32::Foundation::WAIT_OBJECT_0);
    }

    #[test]
    fn soloist_arguments_preserve_paths_and_names_without_a_shell() {
        let setup = Setup {
            executable: PathBuf::from(r"C:\Music apps\soloist.exe"),
            api_key: " dummy-key ".into(),
            device_name: " My PC & music ".into(),
        };
        let data = Path::new(r"C:\Profile folder\state\soloist");
        let cache = Path::new(r"C:\Profile folder\cache\soloist");
        let command = command(&setup, data, cache);
        assert_eq!(command.get_program(), setup.executable.as_os_str());
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_str().unwrap())
            .collect();
        assert_eq!(
            args,
            [
                "--device-name",
                "My PC & music",
                "--api-key",
                "dummy-key",
                "--data-dir",
                data.to_str().unwrap(),
                "--cache-dir",
                cache.to_str().unwrap(),
                "--cache-size",
                "256"
            ]
        );
        assert!(!args.contains(&"--verbose"));
        assert!(!args.contains(&"--ws"));
    }

    #[test]
    fn setup_rejects_relative_executables_and_empty_secrets() {
        let mut setup = Setup {
            executable: "soloist.exe".into(),
            api_key: "key".into(),
            device_name: "PC".into(),
        };
        assert!(setup.validate().is_err());
        setup.executable = std::env::current_exe().unwrap();
        assert!(setup.validate().is_ok());
        setup.api_key = "  ".into();
        assert!(setup.validate().is_err());
        setup.api_key = "key".into();
        setup.device_name.clear();
        assert!(setup.validate().is_err());
    }
}
