use std::fmt::{Debug, Display};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tryvial::try_fn;

#[cfg(feature = "rune")]
#[try_fn]
pub fn rune_module() -> Result<rune::Module, rune::ContextError> {
	let mut module = rune::Module::with_crate_item("hitman_commons", ["game"])?;

	module.ty::<GameVersion>()?;
	module.ty::<GamePlatform>()?;

	module
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::game))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DEBUG))]
#[cfg_attr(feature = "rune", rune(constructor))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum GameVersion {
	#[cfg_attr(feature = "rune", rune(constructor))]
	H1,

	#[cfg_attr(feature = "rune", rune(constructor))]
	H2,

	#[cfg_attr(feature = "rune", rune(constructor))]
	H3
}

impl Display for GameVersion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			GameVersion::H1 => write!(f, "HITMAN™"),
			GameVersion::H2 => write!(f, "HITMAN 2"),
			GameVersion::H3 => write!(f, "HITMAN 3")
		}
	}
}

#[cfg(feature = "rpkg-rs")]
impl From<rpkg_rs::WoaVersion> for GameVersion {
	fn from(value: rpkg_rs::WoaVersion) -> Self {
		match value {
			rpkg_rs::WoaVersion::HM2016 => GameVersion::H1,
			rpkg_rs::WoaVersion::HM2 => GameVersion::H2,
			rpkg_rs::WoaVersion::HM3 => GameVersion::H3
		}
	}
}

#[cfg(feature = "rpkg-rs")]
impl From<GameVersion> for rpkg_rs::WoaVersion {
	fn from(value: GameVersion) -> Self {
		match value {
			GameVersion::H1 => rpkg_rs::WoaVersion::HM2016,
			GameVersion::H2 => rpkg_rs::WoaVersion::HM2,
			GameVersion::H3 => rpkg_rs::WoaVersion::HM3
		}
	}
}

#[cfg(feature = "glacier-texture")]
impl From<glacier_texture::WoaVersion> for GameVersion {
	fn from(value: glacier_texture::WoaVersion) -> Self {
		match value {
			glacier_texture::WoaVersion::HM2016 => GameVersion::H1,
			glacier_texture::WoaVersion::HM2 => GameVersion::H2,
			glacier_texture::WoaVersion::HM3 => GameVersion::H3
		}
	}
}

#[cfg(feature = "glacier-texture")]
impl From<GameVersion> for glacier_texture::WoaVersion {
	fn from(value: GameVersion) -> Self {
		match value {
			GameVersion::H1 => glacier_texture::WoaVersion::HM2016,
			GameVersion::H2 => glacier_texture::WoaVersion::HM2,
			GameVersion::H3 => glacier_texture::WoaVersion::HM3
		}
	}
}

#[derive(Error, Debug)]
pub enum FromTonyToolsError {
	#[error("unknown game version")]
	UnknownGameVersion
}

#[cfg(feature = "tonytools")]
impl TryFrom<tonytools::Version> for GameVersion {
	type Error = FromTonyToolsError;

	#[try_fn]
	fn try_from(value: tonytools::Version) -> Result<Self, Self::Error> {
		match value {
			tonytools::Version::H2016 => GameVersion::H1,
			tonytools::Version::H2 => GameVersion::H2,
			tonytools::Version::H3 => GameVersion::H3,
			tonytools::Version::Unknown => return Err(FromTonyToolsError::UnknownGameVersion)
		}
	}
}

#[cfg(feature = "tonytools")]
impl From<GameVersion> for tonytools::Version {
	fn from(value: GameVersion) -> Self {
		match value {
			GameVersion::H1 => tonytools::Version::H2016,
			GameVersion::H2 => tonytools::Version::H2,
			GameVersion::H3 => tonytools::Version::H3
		}
	}
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::game))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
#[cfg_attr(feature = "rune", rune(constructor))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum GamePlatform {
	#[cfg_attr(feature = "rune", rune(constructor))]
	Steam,

	#[cfg_attr(feature = "rune", rune(constructor))]
	Epic,

	#[cfg_attr(feature = "rune", rune(constructor))]
	GOG,

	#[cfg_attr(feature = "rune", rune(constructor))]
	Microsoft
}

impl Display for GamePlatform {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			GamePlatform::Steam => write!(f, "Steam"),
			GamePlatform::Epic => write!(f, "Epic Games"),
			GamePlatform::GOG => write!(f, "GOG"),
			GamePlatform::Microsoft => write!(f, "Microsoft")
		}
	}
}
