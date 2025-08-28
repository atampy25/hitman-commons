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

use crate::metadata::{ResourceType, RuntimeID};

/// Because RuntimeID is just a wrapper over a u64 derived from an MD5 hash, there is sufficient entropy to simply use it directly for improved performance.
type PassthroughHash = BuildHasherDefault<IdentityHasher<u64>>;

#[static_init::dynamic]
pub static HASH_LIST: HashList = HashList {
	version: AtomicU32::new(0),
	entries: ArcSwap::from_pointee(HashMap::default())
};

#[static_init::dynamic]
pub static CUSTOM_PATHS: papaya::HashMap<RuntimeID, EcoString, PassthroughHash> = papaya::HashMap::default();

#[cfg(feature = "rune")]
#[try_fn]
pub fn rune_module() -> Result<rune::Module, rune::ContextError> {
	let mut module = rune::Module::with_crate_item("hitman_commons", ["hash_list"])?;

	module.function("hash_list", || HASH_LIST.clone()).build()?;

	module.ty::<HashList>()?;
	module.ty::<HashData>()?;

	#[cfg(feature = "hash_list")]
	module.ty::<DeserialisationError>()?;

	module
}

#[cfg(feature = "hash_list")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeserialisedHashList {
	pub version: u32,
	pub entries: Vec<DeserialisedEntry>
}

#[cfg(feature = "hash_list")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeserialisedEntry {
	pub resource_type: ResourceType,
	pub hash: RuntimeID,
	pub path: EcoString,
	pub hint: EcoString,
	pub game_flags: u8
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, CLONE))]
#[cfg_attr(feature = "rune", rune_functions(Self::r_get))]
#[cfg_attr(
	all(feature = "rune", feature = "hash_list"),
	rune_functions(Self::from_compressed__meta)
)]
#[derive(Debug)]
pub struct HashList {
	pub version: AtomicU32,
	pub entries: ArcSwap<HashMap<RuntimeID, HashData, PassthroughHash>>
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
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rune", rune(constructor_fn = Self::rune_construct))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct HashData {
	#[cfg_attr(feature = "rune", rune(get, set))]
	pub resource_type: ResourceType,

	pub path: Option<EcoString>,
	pub hint: Option<EcoString>
}

#[cfg(feature = "rune")]
impl HashData {
	fn rune_construct(resource_type: ResourceType, path: Option<String>, hint: Option<String>) -> Self {
		Self {
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

#[cfg(feature = "hash_list")]
#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum DeserialisationError {
	#[error("decompression failed: {0}")]
	DecompressionFailed(#[from] std::io::Error),

	#[error("deserialisation failed: {0}")]
	DeserialisationFailed(#[from] serde_smile::Error)
}

impl HashList {
	/// Load a hash list from the compressed Brotli/Smile format used by https://github.com/glacier-modding/Hitman-Hashes.
	#[cfg(feature = "hash_list")]
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
					.map(|entry| {
						(
							entry.hash,
							HashData {
								resource_type: entry.resource_type,
								path: (!entry.path.is_empty()).then_some(entry.path),
								hint: (!entry.hint.is_empty()).then_some(entry.hint)
							}
						)
					})
					.collect()
			)
		}
	}

	/// Replace the hash list with entries from the compressed Brotli/Smile format used by https://github.com/glacier-modding/Hitman-Hashes.
	#[cfg(feature = "hash_list")]
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
				.map(|entry| {
					(
						entry.hash,
						HashData {
							resource_type: entry.resource_type,
							path: (!entry.path.is_empty()).then_some(entry.path),
							hint: (!entry.hint.is_empty()).then_some(entry.hint)
						}
					)
				})
				.collect()
		));
	}
}

#[cfg(feature = "rune")]
impl HashList {
	#[rune::function(instance, path = Self::get)]
	fn r_get(&self, id: &RuntimeID) -> Option<HashData> {
		self.entries.load().get(id).cloned()
	}
}
