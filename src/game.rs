#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tryvial::try_fn;

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum GameVersion {
	H1,
	H2,
	H3
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

#[cfg(feature = "tex-rs")]
impl From<tex_rs::WoaVersion> for GameVersion {
	fn from(value: tex_rs::WoaVersion) -> Self {
		match value {
			tex_rs::WoaVersion::HM2016 => GameVersion::H1,
			tex_rs::WoaVersion::HM2 => GameVersion::H2,
			tex_rs::WoaVersion::HM3 => GameVersion::H3
		}
	}
}

#[cfg(feature = "tex-rs")]
impl From<GameVersion> for tex_rs::WoaVersion {
	fn from(value: GameVersion) -> Self {
		match value {
			GameVersion::H1 => tex_rs::WoaVersion::HM2016,
			GameVersion::H2 => tex_rs::WoaVersion::HM2,
			GameVersion::H3 => tex_rs::WoaVersion::HM3
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
