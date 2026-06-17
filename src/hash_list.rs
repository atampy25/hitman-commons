use std::{
	collections::HashMap,
	hash::BuildHasherDefault,
	io::Read,
	sync::atomic::{AtomicU32, Ordering}
};

use arc_swap::ArcSwap;
use ecow::EcoString;
use identity_hash::IdentityHasher;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tryvial::try_fn;

use crate::metadata::{NON_PLATFORM_REGEX, PLATFORM_REGEX, ResourceID, ResourceType};

/// Because ResourceID is just a wrapper over a u64 derived from an MD5 hash, there is sufficient entropy to simply use it directly for improved performance.
type PassthroughHash = BuildHasherDefault<IdentityHasher<u64>>;

#[static_init::dynamic]
pub static HASH_LIST: HashList = HashList {
	version: AtomicU32::new(0),
	entries: ArcSwap::from_pointee(HashMap::default())
};

#[static_init::dynamic]
pub static CUSTOM_PATHS: papaya::HashMap<ResourceID, EcoString, PassthroughHash> = papaya::HashMap::default();

#[cfg(feature = "rune")]
#[try_fn]
pub fn rune_module() -> Result<rune::Module, rune::ContextError> {
	let mut module = rune::Module::with_crate_item("glacier_commons", ["hash_list"])?;

	module.function("hash_list", || HASH_LIST.clone()).build()?;

	module.ty::<HashList>()?;
	module.ty::<HashData>()?;

	#[cfg(feature = "hash-list")]
	module.ty::<DeserialisationError>()?;

	module
}

#[cfg(feature = "hash-list")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeserialisedHashList {
	pub version: u32,
	pub entries: Vec<DeserialisedEntry>
}

#[cfg(feature = "hash-list")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeserialisedEntry {
	pub resource_type: ResourceType,
	pub hash: ResourceID,
	pub path: EcoString,
	pub hint: EcoString,
	pub game_flags: u8
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::glacier_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, CLONE))]
#[cfg_attr(feature = "rune", rune_functions(Self::r_get))]
#[cfg_attr(
	all(feature = "rune", feature = "hash-list"),
	rune_functions(Self::from_compressed__meta)
)]
#[derive(Debug)]
pub struct HashList {
	pub version: AtomicU32,
	pub entries: ArcSwap<HashMap<ResourceID, HashData, PassthroughHash>>
}

impl Clone for HashList {
	fn clone(&self) -> Self {
		Self {
			version: AtomicU32::new(self.version.load(Ordering::SeqCst)),
			entries: ArcSwap::new(self.entries.load().clone())
		}
	}
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::glacier_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rune", rune(constructor_fn = Self::rune_construct))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct HashData {
	#[cfg_attr(feature = "rune", rune(get, set))]
	pub hash: ResourceID,

	#[cfg_attr(feature = "rune", rune(get, set))]
	pub resource_type: ResourceType,

	pub path: Option<EcoString>,
	pub hint: Option<EcoString>
}

#[cfg(feature = "rune")]
impl HashData {
	fn rune_construct(hash: ResourceID, resource_type: ResourceType, path: Option<String>, hint: Option<String>) -> Self {
		Self {
			hash,
			resource_type,
			path: path.map(|x| x.into()),
			hint: hint.map(|x| x.into())
		}
	}

	fn rune_install(module: &mut rune::Module) -> Result<(), rune::ContextError> {
		module.field_function(&rune::runtime::Protocol::GET, "path", |s: &Self| {
			s.path.as_ref().map(|x| String::from(x))
		})?;

		module.field_function(
			&rune::runtime::Protocol::SET,
			"path",
			|s: &mut Self, value: Option<String>| {
				s.path = value.map(|x| x.into());
			}
		)?;

		module.field_function(&rune::runtime::Protocol::GET, "hint", |s: &Self| {
			s.hint.as_ref().map(|x| String::from(x))
		})?;

		module.field_function(
			&rune::runtime::Protocol::SET,
			"hint",
			|s: &mut Self, value: Option<String>| {
				s.hint = value.map(|x| x.into());
			}
		)?;

		Ok(())
	}
}

#[cfg(feature = "hash-list")]
#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::glacier_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum DeserialisationError {
	#[error("decompression failed: {0}")]
	DecompressionFailed(#[from] std::io::Error),

	#[error("deserialisation failed: {0}")]
	DeserialisationFailed(#[from] serde_smile::Error)
}

#[cfg(feature = "hash-list-download")]
#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::glacier_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum DownloadError {
	#[error("couldn't get data directory")]
	NoDataDir,

	#[error("online hash list version wasn't a number: {0}")]
	VersionParseFailed(#[from] std::num::ParseIntError),

	#[error("file system error: {0}")]
	FileSystemError(#[from] std::io::Error),

	#[error("deserialisation error: {0}")]
	DeserialisationError(#[from] DeserialisationError)
}

impl HashList {
	/// Load a hash list from the compressed Brotli/Smile format used by https://github.com/glacier-modding/Hitman-Hashes.
	#[cfg(feature = "hash-list")]
	#[try_fn]
	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::from_compressed))]
	pub fn from_compressed(data: &[u8]) -> Result<Self, DeserialisationError> {
		let mut decompressed = vec![];

		brotli_decompressor::Decompressor::new(data, 4096)
			.read_to_end(&mut decompressed)
			.map_err(DeserialisationError::DecompressionFailed)?;

		let hash_list: DeserialisedHashList =
			serde_smile::from_slice(&decompressed).map_err(DeserialisationError::DeserialisationFailed)?;

		Self {
			version: hash_list.version.into(),
			entries: ArcSwap::from_pointee(
				hash_list
					.entries
					.into_iter()
					.flat_map(|entry| {
						let path = PLATFORM_REGEX.replace(&entry.path, "].$2");
						let hash = ResourceID::from_u64(ResourceID::md5(&path)).unwrap();
						std::iter::once((
							hash,
							HashData {
								hash,
								resource_type: entry.resource_type,
								path: (!path.is_empty()).then(|| path.clone().into()),
								hint: (!entry.hint.is_empty()).then(|| entry.hint.to_owned())
							}
						))
						.chain((!path.is_empty()).then(|| {
							(
								ResourceID::from_u64(ResourceID::md5(&NON_PLATFORM_REGEX.replace(&path, "].pc_$1")))
									.unwrap(),
								HashData {
									hash,
									resource_type: entry.resource_type,
									path: Some(path.into()),
									hint: (!entry.hint.is_empty()).then_some(entry.hint)
								}
							)
						}))
					})
					.collect()
			)
		}
	}

	/// Replace the hash list with entries from the compressed Brotli/Smile format used by https://github.com/glacier-modding/Hitman-Hashes.
	#[cfg(feature = "hash-list")]
	#[try_fn]
	pub fn load_compressed(&self, data: &[u8]) -> Result<(), DeserialisationError> {
		use std::sync::Arc;

		let mut decompressed = vec![];

		brotli_decompressor::Decompressor::new(data, 4096)
			.read_to_end(&mut decompressed)
			.map_err(DeserialisationError::DecompressionFailed)?;

		let hash_list: DeserialisedHashList =
			serde_smile::from_slice(&decompressed).map_err(DeserialisationError::DeserialisationFailed)?;

		self.version.store(hash_list.version, Ordering::SeqCst);
		self.entries.store(Arc::new(
			hash_list
				.entries
				.into_iter()
				.flat_map(|entry| {
					let path = PLATFORM_REGEX.replace(&entry.path, "].$2");
					let hash = ResourceID::from_u64(ResourceID::md5(&path)).unwrap();
					std::iter::once((
						hash,
						HashData {
							hash,
							resource_type: entry.resource_type,
							path: (!path.is_empty()).then(|| path.clone().into()),
							hint: (!entry.hint.is_empty()).then(|| entry.hint.to_owned())
						}
					))
					.chain((!path.is_empty()).then(|| {
						(
							ResourceID::from_u64(ResourceID::md5(&NON_PLATFORM_REGEX.replace(&path, "].pc_$1")))
								.unwrap(),
							HashData {
								hash,
								resource_type: entry.resource_type,
								path: Some(path.into()),
								hint: (!entry.hint.is_empty()).then_some(entry.hint)
							}
						)
					}))
				})
				.collect()
		));
	}

	pub const VERSION_ENDPOINT: &str =
		"https://github.com/glacier-modding/Hitman-Hashes/releases/latest/download/version";

	pub const DOWNLOAD_ENDPOINT: &str =
		"https://github.com/glacier-modding/Hitman-Hashes/releases/latest/download/hash_list.sml";

	/// Download, parse and cache the latest hash list version to the user's local data directory.
	/// Will make a network request to check the latest version, and another to download it if the cached version is outdated or missing.
	/// If the network requests fail, no error will be returned and the existing cached version (if any) will remain in use.
	#[cfg(feature = "hash-list-download")]
	#[try_fn]
	pub async fn load_latest(&self) -> Result<(), DownloadError> {
		use std::fs;

		let data_dir = dirs::data_local_dir()
			.ok_or(DownloadError::NoDataDir)?
			.join("glacier-commons");

		let hash_list_path = data_dir.join("hash_list.sml");

		let _ = fs::read(&hash_list_path)
			.ok()
			.and_then(|x| self.load_compressed(&x).ok());

		let current_version = self.version.load(Ordering::SeqCst);

		if let Ok(data) = reqwest::get(Self::VERSION_ENDPOINT).await
			&& let Ok(data) = data.error_for_status()
			&& let Ok(data) = data.text().await
		{
			let new_version = data.trim().parse::<u32>()?;

			if current_version < new_version
				&& let Ok(data) = reqwest::get(Self::DOWNLOAD_ENDPOINT).await
				&& let Ok(data) = data.error_for_status()
				&& let Ok(data) = data.bytes().await
			{
				self.load_compressed(&data)?;

				fs::create_dir_all(data_dir)?;
				fs::write(hash_list_path, data)?;
			}
		}
	}
}

#[cfg(feature = "rune")]
impl HashList {
	#[rune::function(instance, path = Self::get)]
	fn r_get(&self, id: &ResourceID) -> Option<HashData> {
		self.entries.load().get(id).cloned()
	}
}
