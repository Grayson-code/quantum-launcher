use std::{
    path::PathBuf,
    process::Child,
    sync::{mpsc::Receiver, Arc},
};

use quantum_launcher_backend::{
    error::LauncherResult, io_err, json_structs::json_instance_config::InstanceConfigJson,
    DownloadProgress, FabricVersion, GameLaunchResult, JavaInstallMessage,
};

use crate::config::LauncherConfig;

#[derive(Debug, Clone)]
pub enum Message {
    OpenDir(PathBuf),
    InstallFabricEnd(Result<(), String>),
    InstallFabricVersionSelected(String),
    InstallFabricVersionsLoaded(Result<Vec<FabricVersion>, String>),
    LaunchInstanceSelected(String),
    LaunchUsernameSet(String),
    LaunchStart,
    DeleteInstanceMenu,
    DeleteInstance,
    LaunchScreenOpen,
    LaunchEnd(GameLaunchResult),
    LaunchJavaInstallProgressUpdate,
    CreateInstanceScreenOpen,
    CreateInstanceVersionsLoaded(Result<Arc<Vec<String>>, String>),
    CreateInstanceVersionSelected(String),
    CreateInstanceNameInput(String),
    CreateInstanceStart,
    CreateInstanceEnd(Result<(), String>),
    CreateInstanceChangeAssetToggle(bool),
    CreateInstanceProgressUpdate,
    EditInstance,
    EditInstanceJavaOverride(String),
    EditInstanceMemoryChanged(f32),
    EditInstanceSave,
    ManageModsScreenOpen,
    InstallFabricClicked,
    InstallFabricScreenOpen,
    ErrorCopy,
}

#[derive(Default)]
pub struct MenuLaunch {
    pub selected_instance: Option<String>,
    pub java_install_progress: Option<JavaInstallProgress>,
}

pub struct JavaInstallProgress {
    pub num: f32,
    pub recv: Receiver<JavaInstallMessage>,
    pub message: String,
}

pub struct MenuEditInstance {
    pub selected_instance: String,
    pub config: InstanceConfigJson,
    pub slider_value: f32,
    pub slider_text: String,
}

pub struct MenuEditMods {
    pub selected_instance: String,
    pub config: InstanceConfigJson,
}

pub struct MenuCreateInstance {
    pub instance_name: String,
    pub selected_version: Option<String>,
    pub versions: Vec<String>,
    pub progress_receiver: Option<Receiver<DownloadProgress>>,
    pub progress_number: Option<f32>,
    pub progress_text: Option<String>,
    pub download_assets: bool,
}

pub struct MenuDeleteInstance {
    pub selected_instance: String,
}

pub struct MenuInstallFabric {
    pub selected_instance: String,
    pub fabric_version: Option<String>,
    pub fabric_versions: Vec<String>,
}

pub enum State {
    Launch(MenuLaunch),
    EditInstance(MenuEditInstance),
    EditMods(MenuEditMods),
    Create(MenuCreateInstance),
    Error { error: String },
    DeleteInstance(MenuDeleteInstance),
    InstallFabric(MenuInstallFabric),
}

pub struct Launcher {
    pub state: State,
    pub instances: Option<Vec<String>>,
    pub config: Option<LauncherConfig>,
    pub spawned_process: Option<Arc<std::sync::Mutex<Child>>>,
}

impl Launcher {
    pub fn new() -> LauncherResult<Self> {
        // .config/QuantumLauncher/ OR AppData/Roaming/QuantumLauncher/
        let dir_path = quantum_launcher_backend::file_utils::get_launcher_dir()?;
        std::fs::create_dir_all(&dir_path).map_err(io_err!(dir_path))?;

        // QuantumLauncher/instances/
        let dir_path = dir_path.join("instances");
        std::fs::create_dir_all(&dir_path).map_err(io_err!(dir_path))?;

        let dir = std::fs::read_dir(&dir_path).map_err(io_err!(dir_path))?;

        let subdirectories: Vec<String> = dir
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            return Some(file_name.to_owned());
                        }
                    }
                }
                None
            })
            .collect();

        Ok(Self {
            instances: Some(subdirectories),
            state: State::Launch(MenuLaunch::default()),
            spawned_process: None,
            config: Some(LauncherConfig::load()?),
        })
    }

    pub fn with_error(error: String) -> Self {
        Self {
            state: State::Error {
                error: format!("Error: {error}"),
            },
            instances: None,
            config: LauncherConfig::load().ok(),
            spawned_process: None,
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.state = State::Error { error }
    }

    pub fn go_to_launch_screen(&mut self) {
        self.state = State::Launch(MenuLaunch::default())
    }

    pub fn edit_instance_wrapped(&mut self) {
        if let State::Launch(menu_launch) = &self.state {
            match self.edit_instance(menu_launch.selected_instance.clone().unwrap()) {
                Ok(_) => {}
                Err(err) => self.set_error(err.to_string()),
            }
        }
    }
}
