use crate::game::{GamePlatform, GameVersion};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, path::PathBuf};
use thiserror::Error;

#[cfg(feature = "rune")]
pub fn rune_module() -> Result<rune::Module, rune::ContextError> {
	let mut module = rune::Module::with_crate_item("hitman_commons", ["game_detection"])?;

	module.function_meta(detect_installs__meta)?;
	module.ty::<GameInstall>()?;
	module.ty::<GameDetectionError>()?;

	Ok(module)
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::game_detection))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum GameDetectionError {
	#[error("Couldn't get environment variable {0}: {1}")]
	EnvVar(String, std::env::VarError),

	#[error("IO error for {0}: {1}")]
	Io(String, std::io::Error),

	#[error("JSON deserialisation error for {0}: {1}")]
	JsonDeserialisation(String, serde_json::Error),

	#[error("VDF deserialisation error for {0}: {1}")]
	VdfDeserialisation(String, Box<keyvalues_serde::Error>),

	#[error("Missing field {0}")]
	MissingField(String),

	#[error("Value {0} was not type {1}")]
	IncorrectType(String, String)
}

#[derive(Deserialize)]
struct SteamLibraryFolder {
	path: String,
	apps: HashMap<String, String>
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::game_detection))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT))]
#[cfg_attr(feature = "rune", rune(install_with = Self::rune_install))]
#[cfg_attr(feature = "rune", rune(constructor_fn = Self::rune_construct))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct GameInstall {
	#[cfg_attr(feature = "rune", rune(get, set))]
	pub version: GameVersion,

	#[cfg_attr(feature = "rune", rune(get, set))]
	pub platform: GamePlatform,

	pub path: PathBuf
}

#[cfg(feature = "rune")]
impl GameInstall {
	fn rune_construct(version: GameVersion, platform: GamePlatform, path: String) -> Self {
		Self {
			version,
			platform,
			path: PathBuf::from(path)
		}
	}

	fn rune_install(module: &mut rune::Module) -> Result<(), rune::ContextError> {
		module.field_function(&rune::runtime::Protocol::GET, "path", |s: &Self| {
			s.path.to_string_lossy().to_string()
		})?;

		module.field_function(&rune::runtime::Protocol::SET, "path", |s: &mut Self, value: String| {
			s.path = PathBuf::from(value);
		})?;

		Ok(())
	}
}

#[cfg_attr(feature = "rune", rune::function(keep))]
pub fn detect_installs() -> Result<Vec<GameInstall>, GameDetectionError> {
	detection::detect_installs()
}

#[cfg(target_os = "windows")]
mod detection {
	use std::collections::HashMap;
	use std::os::windows::process::CommandExt;
	use std::{fs, path::PathBuf};
	use std::{path::Path, process::Command};

	use itertools::Itertools;
	use registry::{Data, Hive, Security};
	use serde_json::Value;
	use tryvial::try_fn;

	use crate::game::GameVersion;

	use super::{GameDetectionError, GameInstall, GamePlatform, SteamLibraryFolder};

	#[try_fn]
	pub fn detect_installs() -> Result<Vec<GameInstall>, GameDetectionError> {
		let legendary_installed_paths = [
			Path::new(&std::env::var("USERPROFILE").map_err(|x| GameDetectionError::EnvVar("USERPROFILE".into(), x))?)
				.join(".config")
				.join("legendary")
				.join("installed.json"),
			Path::new(&std::env::var("APPDATA").map_err(|x| GameDetectionError::EnvVar("APPDATA".into(), x))?)
				.join("heroic")
				.join("legendaryConfig")
				.join("legendary")
				.join("installed.json")
		];

		let mut check_paths = vec![];

		// Legendary installs
		for legendary_installed_path in legendary_installed_paths {
			if legendary_installed_path.exists() {
				let legendary_installed_data: Value = serde_json::from_slice(
					&fs::read(&legendary_installed_path)
						.map_err(|x| GameDetectionError::Io(legendary_installed_path.to_string_lossy().into(), x))?
				)
				.map_err(|x| {
					GameDetectionError::JsonDeserialisation(legendary_installed_path.to_string_lossy().into(), x)
				})?;

				// H3
				if let Some(data) = legendary_installed_data.get("Eider") {
					check_paths.push((
						PathBuf::from(
							data.get("install_path")
								.ok_or_else(|| GameDetectionError::MissingField("install_path".into()))?
								.as_str()
								.ok_or_else(|| {
									GameDetectionError::IncorrectType("install_path".into(), "string".into())
								})?
						),
						GamePlatform::Epic
					));
				}

				// H1
				if let Some(data) = legendary_installed_data.get("Barbet") {
					check_paths.push((
						PathBuf::from(
							data.get("install_path")
								.ok_or_else(|| GameDetectionError::MissingField("install_path".into()))?
								.as_str()
								.ok_or_else(|| {
									GameDetectionError::IncorrectType("install_path".into(), "string".into())
								})?
						),
						GamePlatform::Epic
					));
				}
			}
		}

		// EGL installs
		if let Ok(hive) = Hive::CurrentUser.open(r#"Software\Epic Games\EOS"#, Security::Read) {
			match hive.value("ModSdkMetadataDir") {
				Ok(Data::String(d)) => {
					if let Ok(entries) = fs::read_dir(d.to_string_lossy()) {
						for entry in entries
							.filter_map(|x| x.ok())
							.filter(|x| x.file_type().ok().map(|x| x.is_file()).unwrap_or(false))
						{
							if let Ok(manifest_data) = serde_json::from_slice::<Value>(
								&fs::read(entry.path())
									.map_err(|x| GameDetectionError::Io(entry.path().to_string_lossy().into(), x))?
							) {
								// H3
								if manifest_data
									.get("AppName")
									.ok_or_else(|| GameDetectionError::MissingField("AppName".into()))?
									.as_str()
									.ok_or_else(|| {
										GameDetectionError::IncorrectType("AppName".into(), "string".into())
									})? == "Eider"
								{
									check_paths.push((
										PathBuf::from(
											manifest_data
												.get("InstallLocation")
												.ok_or_else(|| {
													GameDetectionError::MissingField("InstallLocation".into())
												})?
												.as_str()
												.ok_or_else(|| {
													GameDetectionError::IncorrectType(
														"InstallLocation".into(),
														"string".into()
													)
												})?
										),
										GamePlatform::Epic
									));
								}

								// H1
								if manifest_data
									.get("AppName")
									.ok_or_else(|| GameDetectionError::MissingField("AppName".into()))?
									.as_str()
									.ok_or_else(|| {
										GameDetectionError::IncorrectType("AppName".into(), "string".into())
									})? == "Barbet"
								{
									check_paths.push((
										PathBuf::from(
											manifest_data
												.get("InstallLocation")
												.ok_or_else(|| {
													GameDetectionError::MissingField("InstallLocation".into())
												})?
												.as_str()
												.ok_or_else(|| {
													GameDetectionError::IncorrectType(
														"InstallLocation".into(),
														"string".into()
													)
												})?
										),
										GamePlatform::Epic
									));
								}
							}
						}
					}
				}

				Ok(_) => Err(GameDetectionError::IncorrectType(
					"ModSdkMetadataDir".into(),
					"string".into()
				))?,

				Err(_) => {}
			}
		}

		// 	Steam installs
		if let Ok(hive) = Hive::CurrentUser.open(r#"Software\Valve\Steam"#, Security::Read) {
			match hive.value("SteamPath") {
				Ok(Data::String(d)) => {
					let libraryfolders_path = if Path::new(&d.to_string_lossy())
						.join("config")
						.join("libraryfolders.vdf")
						.exists()
					{
						Path::new(&d.to_string_lossy())
							.join("config")
							.join("libraryfolders.vdf")
					} else {
						Path::new(&d.to_string_lossy())
							.join("steamapps")
							.join("libraryfolders.vdf")
					};

					if let Ok(s) = fs::read_to_string(&libraryfolders_path) {
						let folders: HashMap<String, SteamLibraryFolder> =
							keyvalues_serde::from_str(&s).map_err(|x| {
								GameDetectionError::VdfDeserialisation(
									libraryfolders_path.to_string_lossy().into(),
									x.into()
								)
							})?;

						for folder in folders.values() {
							// H1, H1 free trial
							if folder.apps.contains_key("236870") || folder.apps.contains_key("649780") {
								check_paths.push((
									PathBuf::from(&folder.path)
										.join("steamapps")
										.join("common")
										.join("HITMAN™"),
									GamePlatform::Steam
								));
							}

							// H2
							if folder.apps.contains_key("863550") {
								check_paths.push((
									PathBuf::from(&folder.path)
										.join("steamapps")
										.join("common")
										.join("HITMAN2"),
									GamePlatform::Steam
								));
							}

							// H3, H3 demo
							if folder.apps.contains_key("1659040") || folder.apps.contains_key("1847520") {
								check_paths.push((
									PathBuf::from(&folder.path)
										.join("steamapps")
										.join("common")
										.join("HITMAN 3"),
									GamePlatform::Steam
								));
							}
						}
					};
				}

				Ok(_) => Err(GameDetectionError::IncorrectType("SteamPath".into(), "string".into()))?,

				Err(_) => {}
			}
		}

		// Microsoft install of H3
		if let Ok(proc_out) = Command::new("powershell")
			.args(["-Command", "Get-AppxPackage -Name IOInteractiveAS.PC-HITMAN3-BaseGame"])
			.creation_flags(0x08000000) // CREATE_NO_WINDOW
			.output()
		{
			if let Some(line) = String::from_utf8_lossy(&proc_out.stdout)
				.lines()
				.find(|x| x.starts_with("InstallLocation"))
			{
				let path = line.split(':').skip(1).collect::<Vec<_>>().join(":");

				check_paths.push((
					fs::read_link(path.trim()).map_err(|x| GameDetectionError::Io(path.trim().into(), x))?,
					GamePlatform::Microsoft
				));
			}
		}

		// GOG install of H1
		if let Ok(hive) = Hive::LocalMachine.open(r#"Software\WOW6432Node\GOG.com\Games\1545448592"#, Security::Read) {
			match hive.value("path") {
				Ok(Data::String(d)) => {
					check_paths.push((PathBuf::from(&d.to_string_lossy()), GamePlatform::GOG));
				}

				_ => Err(GameDetectionError::IncorrectType("path".into(), "string".into()))?
			}
		}

		let mut game_installs = vec![];

		for (path, platform) in check_paths {
			// Game folder has Retail
			let subfolder_retail = path.join("Retail").is_dir();

			if subfolder_retail {
				game_installs.push(GameInstall {
					path: path.join("Retail"),
					platform,
					version: if path.join("Retail").join("HITMAN3.exe").is_file() {
						GameVersion::H3
					} else if path.join("Retail").join("HITMAN2.exe").is_file() {
						GameVersion::H2
					} else if path.join("Retail").join("HITMAN.exe").is_file() {
						GameVersion::H1
					} else {
						panic!("Unknown game added to check paths");
					}
				});
			}
		}

		game_installs
			.into_iter()
			.unique_by(|x| x.path.to_owned())
			.sorted_unstable_by_key(|x| x.version)
			.collect()
	}
}

#[cfg(target_os = "linux")]
mod detection {
	use std::collections::HashMap;
	use std::{fs, path::PathBuf};

	use itertools::Itertools;
	use serde_json::Value;
	use tryvial::try_fn;

	use crate::game::GameVersion;

	use super::{GameDetectionError, GameInstall, GamePlatform, SteamLibraryFolder};

	#[try_fn]
	pub fn detect_installs() -> Result<Vec<GameInstall>, GameDetectionError> {
		let mut check_paths = vec![];

		// Legendary installs
		if let Some(home_dir) = home::home_dir() {
			let legendary_installed_path = home_dir
				.join(".config/legendary/installed.json")
				.exists()
				.then_some(home_dir.join(".config/legendary/installed.json"));

			if let Some(legendary_installed_path) = legendary_installed_path {
				let legendary_installed_data: Value = serde_json::from_slice(
					&fs::read(&legendary_installed_path)
						.map_err(|x| GameDetectionError::Io(legendary_installed_path.to_string_lossy().into(), x))?
				)
				.map_err(|x| {
					GameDetectionError::JsonDeserialisation(legendary_installed_path.to_string_lossy().into(), x)
				})?;

				// H3
				if let Some(data) = legendary_installed_data.get("Eider") {
					check_paths.push((
						PathBuf::from(
							data.get("install_path")
								.ok_or_else(|| GameDetectionError::MissingField("install_path".into()))?
								.as_str()
								.ok_or_else(|| {
									GameDetectionError::IncorrectType("install_path".into(), "string".into())
								})?
						),
						GamePlatform::Epic
					));
				}

				// H1
				if let Some(data) = legendary_installed_data.get("Barbet") {
					check_paths.push((
						PathBuf::from(
							data.get("install_path")
								.ok_or_else(|| GameDetectionError::MissingField("install_path".into()))?
								.as_str()
								.ok_or_else(|| {
									GameDetectionError::IncorrectType("install_path".into(), "string".into())
								})?
						),
						GamePlatform::Epic
					));
				}
			}
		}

		// Steam installs
		if let Some(home_dir) = home::home_dir() {
			let steam_path = match home_dir {
				home if home_dir.join(".local/share/Steam").exists() => Some(home.join(".local/share/Steam")),
				home if home_dir.join(".steam/steam").exists() => Some(home.join(".steam/steam")),
				_ => None
			};

			if let Some(steam_path) = steam_path {
				let libraryfolders_path = if steam_path.join("config").join("libraryfolders.vdf").exists() {
					steam_path.join("config").join("libraryfolders.vdf")
				} else {
					steam_path.join("steamapps").join("libraryfolders.vdf")
				};

				if let Ok(s) = fs::read_to_string(&libraryfolders_path) {
					let folders: HashMap<String, SteamLibraryFolder> = keyvalues_serde::from_str(&s).map_err(|x| {
						GameDetectionError::VdfDeserialisation(libraryfolders_path.to_string_lossy().into(), x.into())
					})?;

					for folder in folders.values() {
						// H1, H1 free trial
						if folder.apps.contains_key("236870") || folder.apps.contains_key("649780") {
							check_paths.push((
								PathBuf::from(&folder.path)
									.join("steamapps")
									.join("common")
									.join("HITMAN™"),
								GamePlatform::Steam
							));

							check_paths.push((
								PathBuf::from(&folder.path)
									.join("steamapps")
									.join("common")
									.join("Hitman™"),
								GamePlatform::Steam
							));

							check_paths.push((
								PathBuf::from(&folder.path)
									.join("steamapps")
									.join("common")
									.join("Hitman™")
									.join("share")
									.join("data"),
								GamePlatform::Steam
							));
						}

						// H2
						if folder.apps.contains_key("863550") {
							check_paths.push((
								PathBuf::from(&folder.path)
									.join("steamapps")
									.join("common")
									.join("HITMAN2"),
								GamePlatform::Steam
							));
						}

						// H3, H3 demo
						if folder.apps.contains_key("1659040") || folder.apps.contains_key("1847520") {
							check_paths.push((
								PathBuf::from(&folder.path)
									.join("steamapps")
									.join("common")
									.join("HITMAN 3"),
								GamePlatform::Steam
							));
						}
					}
				};
			}
		}

		let mut game_installs = vec![];

		for (path, platform) in check_paths {
			let retail_folder = ["Retail", "retail"]
				.iter()
				.map(|folder| path.join(folder))
				.find(|joined_path| joined_path.exists());

			if let Some(retail_folder) = retail_folder {
				let version = if retail_folder.join("HITMAN3.exe").is_file() {
					GameVersion::H3
				} else if retail_folder.join("HITMAN2.exe").is_file() {
					GameVersion::H2
				} else if retail_folder.join("HITMAN.exe").is_file() || retail_folder.join("hitman.dll").is_file() {
					GameVersion::H1
				} else {
					panic!("Unknown game added to check paths");
				};

				game_installs.push(GameInstall {
					path: retail_folder,
					platform,
					version
				});
			}
		}

		game_installs
			.into_iter()
			.unique_by(|x| x.path.to_owned())
			.sorted_unstable_by_key(|x| x.version)
			.collect()
	}
}
