use std::fmt::{Debug, Display};

use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "rune")]
#[tryvial::try_fn]
pub fn rune_module() -> Result<rune::Module, rune::ContextError> {
	let mut module = rune::Module::with_crate_item("glacier_commons", ["game"])?;

	module.ty::<GlacierGame>()?;
	module.ty::<GamePlatform>()?;
	module.ty::<StorePlatform>()?;

	module
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::glacier_commons::game))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum GlacierGame {
	#[cfg_attr(feature = "rune", rune(constructor))]
	H1,

	#[cfg_attr(feature = "rune", rune(constructor))]
	H2,

	#[cfg_attr(feature = "rune", rune(constructor))]
	H3,

	#[cfg_attr(feature = "rune", rune(constructor))]
	FL
}

impl Display for GlacierGame {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			GlacierGame::H1 => f.write_str("HITMAN™"),
			GlacierGame::H2 => f.write_str("HITMAN 2"),
			GlacierGame::H3 => f.write_str("HITMAN 3"),
			GlacierGame::FL => f.write_str("007 First Light")
		}
	}
}

#[cfg(feature = "rpkg-rs")]
impl From<rpkg_rs::WoaVersion> for GlacierGame {
	fn from(value: rpkg_rs::WoaVersion) -> Self {
		match value {
			rpkg_rs::WoaVersion::HM2016 => GlacierGame::H1,
			rpkg_rs::WoaVersion::HM2 => GlacierGame::H2,
			rpkg_rs::WoaVersion::HM3 => GlacierGame::H3
		}
	}
}

#[cfg(feature = "rpkg-rs")]
impl From<GlacierGame> for Option<rpkg_rs::WoaVersion> {
	fn from(value: GlacierGame) -> Self {
		match value {
			GlacierGame::H1 => Some(rpkg_rs::WoaVersion::HM2016),
			GlacierGame::H2 => Some(rpkg_rs::WoaVersion::HM2),
			GlacierGame::H3 => Some(rpkg_rs::WoaVersion::HM3),
			_ => None
		}
	}
}

#[derive(Error, Debug)]
pub enum FromGlacierTextureError {
	#[error("unknown game version")]
	UnknownGlacierGame
}

#[cfg(feature = "glacier-texture")]
impl TryFrom<glacier_texture::GlacierGame> for GlacierGame {
	type Error = FromGlacierTextureError;

	#[tryvial::try_fn]
	fn try_from(value: glacier_texture::GlacierGame) -> Result<Self, Self::Error> {
		match value {
			glacier_texture::GlacierGame::HM2016 => GlacierGame::H1,
			glacier_texture::GlacierGame::HM2 => GlacierGame::H2,
			glacier_texture::GlacierGame::HM3 => GlacierGame::H3,
			glacier_texture::GlacierGame::KNT => GlacierGame::FL,
			_ => return Err(FromGlacierTextureError::UnknownGlacierGame)
		}
	}
}

#[cfg(feature = "glacier-texture")]
impl From<GlacierGame> for glacier_texture::GlacierGame {
	fn from(value: GlacierGame) -> Self {
		match value {
			GlacierGame::H1 => glacier_texture::GlacierGame::HM2016,
			GlacierGame::H2 => glacier_texture::GlacierGame::HM2,
			GlacierGame::H3 => glacier_texture::GlacierGame::HM3,
			GlacierGame::FL => glacier_texture::GlacierGame::KNT
		}
	}
}

#[derive(Error, Debug)]
pub enum FromTonyToolsError {
	#[error("unknown game version")]
	UnknownGlacierGame
}

#[cfg(feature = "tonytools")]
impl TryFrom<tonytools::Version> for GlacierGame {
	type Error = FromTonyToolsError;

	#[tryvial::try_fn]
	fn try_from(value: tonytools::Version) -> Result<Self, Self::Error> {
		match value {
			tonytools::Version::H2016 => GlacierGame::H1,
			tonytools::Version::H2 => GlacierGame::H2,
			tonytools::Version::H3 => GlacierGame::H3,
			tonytools::Version::KNT => GlacierGame::FL,
			tonytools::Version::Unknown => return Err(FromTonyToolsError::UnknownGlacierGame)
		}
	}
}

#[cfg(feature = "tonytools")]
impl From<GlacierGame> for tonytools::Version {
	fn from(value: GlacierGame) -> Self {
		match value {
			GlacierGame::H1 => tonytools::Version::H2016,
			GlacierGame::H2 => tonytools::Version::H2,
			GlacierGame::H3 => tonytools::Version::H3,
			GlacierGame::FL => tonytools::Version::KNT
		}
	}
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::glacier_commons::game))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum GamePlatform {
	#[cfg_attr(feature = "serde", serde(rename = "pc"))]
	PC,

	#[cfg_attr(feature = "serde", serde(rename = "macos"))]
	MacOS,

	#[allow(non_camel_case_types)]
	#[cfg_attr(feature = "serde", serde(rename = "ios"))]
	iOS,

	#[cfg_attr(feature = "serde", serde(rename = "ps4"))]
	PS4,

	#[cfg_attr(feature = "serde", serde(rename = "ps5"))]
	PS5,

	#[cfg_attr(feature = "serde", serde(rename = "xboxOne"))]
	XboxOne,

	#[cfg_attr(feature = "serde", serde(rename = "xboxSeries"))]
	XboxSeries,

	#[cfg_attr(feature = "serde", serde(rename = "nintendoSwitch"))]
	NintendoSwitch,

	#[cfg_attr(feature = "serde", serde(rename = "nintendoSwitch2"))]
	NintendoSwitch2
}

impl Display for GamePlatform {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			GamePlatform::PC => f.write_str("PC"),
			GamePlatform::MacOS => f.write_str("macOS"),
			GamePlatform::iOS => f.write_str("iOS"),
			GamePlatform::PS4 => f.write_str("PlayStation 4"),
			GamePlatform::PS5 => f.write_str("PlayStation 5"),
			GamePlatform::XboxOne => f.write_str("Xbox One"),
			GamePlatform::XboxSeries => f.write_str("Xbox Series X/S"),
			GamePlatform::NintendoSwitch => f.write_str("Nintendo Switch"),
			GamePlatform::NintendoSwitch2 => f.write_str("Nintendo Switch 2")
		}
	}
}

impl GamePlatform {
	pub fn from_codename(codename: &str) -> Option<Self> {
		match codename {
			"pc" => Some(GamePlatform::PC),
			"macos" => Some(GamePlatform::MacOS),
			"ios" => Some(GamePlatform::iOS),
			"orbis" => Some(GamePlatform::PS4),
			"ps5" => Some(GamePlatform::PS5),
			"durango" => Some(GamePlatform::XboxOne),
			"scarlett" => Some(GamePlatform::XboxSeries),
			"izumo" => Some(GamePlatform::NintendoSwitch),
			"ounce" => Some(GamePlatform::NintendoSwitch2),
			_ => None
		}
	}

	pub fn codename(&self) -> &'static str {
		match self {
			GamePlatform::PC => "pc",
			GamePlatform::MacOS => "macos",
			GamePlatform::iOS => "ios",
			GamePlatform::PS4 => "orbis",
			GamePlatform::PS5 => "ps5",
			GamePlatform::XboxOne => "durango",
			GamePlatform::XboxSeries => "scarlett",
			GamePlatform::NintendoSwitch => "izumo",
			GamePlatform::NintendoSwitch2 => "ounce"
		}
	}

	pub fn from_tag(tag: u8) -> Option<Self> {
		match tag {
			1 => Some(GamePlatform::PC),
			2 => Some(GamePlatform::PS5),
			3 => Some(GamePlatform::XboxSeries),
			4 => Some(GamePlatform::NintendoSwitch2),
			_ => None
		}
	}

	pub fn tag(&self) -> Option<u8> {
		match self {
			GamePlatform::PC => Some(1),
			GamePlatform::PS5 => Some(2),
			GamePlatform::XboxSeries => Some(3),
			GamePlatform::NintendoSwitch2 => Some(4),
			_ => None
		}
	}
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::glacier_commons::game))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum StorePlatform {
	#[cfg_attr(feature = "rune", rune(constructor))]
	Steam,

	#[cfg_attr(feature = "rune", rune(constructor))]
	Epic,

	#[cfg_attr(feature = "rune", rune(constructor))]
	GOG,

	#[cfg_attr(feature = "rune", rune(constructor))]
	Microsoft
}

impl Display for StorePlatform {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			StorePlatform::Steam => f.write_str("Steam"),
			StorePlatform::Epic => f.write_str("Epic Games"),
			StorePlatform::GOG => f.write_str("GOG"),
			StorePlatform::Microsoft => f.write_str("Microsoft")
		}
	}
}
