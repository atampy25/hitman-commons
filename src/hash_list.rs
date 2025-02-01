use std::io::Read;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tryvial::try_fn;

use crate::metadata::{PathedID, ResourceType, RuntimeID};

#[cfg(feature = "rune")]
#[try_fn]
pub fn rune_module() -> Result<rune::Module, rune::ContextError> {
	let mut module = rune::Module::with_crate_item("hitman_commons", ["hash_list"])?;

	module.ty::<HashList>()?;
	module.ty::<HashData>()?;
	module.ty::<DeserialisationError>()?;

	module
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeserialisedHashList {
	pub version: u32,
	pub entries: Vec<DeserialisedEntry>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeserialisedEntry {
	pub resource_type: ResourceType,
	pub hash: RuntimeID,
	pub path: String,
	pub hint: String,
	pub game_flags: u8
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DEBUG))]
#[cfg_attr(
	feature = "rune",
	rune_functions(
		Self::from_compressed__meta,
		Self::to_path__meta,
		Self::to_pathed_id__meta,
		Self::r_get_entry,
		Self::r_insert_entry,
		Self::r_remove_entry
	)
)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HashList {
	#[cfg_attr(feature = "rune", rune(get, set))]
	pub version: u32,

	pub entries: HashMap<RuntimeID, HashData>
}

#[cfg(feature = "rune")]
impl HashList {
	#[rune::function(instance, path = Self::get_entry)]
	fn r_get_entry(&self, hash: &RuntimeID) -> Option<HashData> {
		self.entries.get(hash).cloned()
	}

	#[rune::function(instance, path = Self::insert_entry)]
	fn r_insert_entry(&mut self, hash: RuntimeID, data: HashData) {
		self.entries.insert(hash, data);
	}

	#[rune::function(instance, path = Self::remove_entry)]
	fn r_remove_entry(&mut self, hash: &RuntimeID) -> Option<HashData> {
		self.entries.remove(hash)
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "rune", serde_with::apply(_ => #[rune(get, set)]))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DEBUG))]
#[cfg_attr(feature = "rune", rune(constructor))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct HashData {
	pub resource_type: ResourceType,
	pub path: Option<String>,
	pub hint: Option<String>
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::hash_list))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
#[cfg_attr(feature = "rune", rune(constructor))]
pub enum DeserialisationError {
	#[error("decompression failed: {0}")]
	DecompressionFailed(#[from] std::io::Error),

	#[error("deserialisation failed: {0}")]
	DeserialisationFailed(#[from] serde_smile::Error)
}

impl HashList {
	#[try_fn]
	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::from_compressed))]
	pub fn from_compressed(slice: &[u8]) -> Result<Self, DeserialisationError> {
		let mut decompressed = vec![];

		brotli_decompressor::Decompressor::new(slice, 4096)
			.read_to_end(&mut decompressed)
			.map_err(DeserialisationError::DecompressionFailed)?;

		let hash_list: DeserialisedHashList =
			serde_smile::from_slice(&decompressed).map_err(DeserialisationError::DeserialisationFailed)?;

		HashList {
			version: hash_list.version,
			entries: hash_list
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
		}
	}

	/// Gets the path of a resource if possible; otherwise just returns the hash.
	#[cfg_attr(feature = "rune", rune::function(keep))]
	pub fn to_path(&self, hash: &RuntimeID) -> String {
		if let Some(entry) = self.entries.get(hash) {
			if let Some(path) = entry.path.as_ref() {
				return path.to_owned();
			}
		}

		hash.to_string()
	}

	/// Converts the hash to a PathedID.
	#[cfg_attr(feature = "rune", rune::function(keep))]
	pub fn to_pathed_id(&self, hash: &RuntimeID) -> PathedID {
		if let Some(entry) = self.entries.get(hash) {
			if let Some(path) = entry.path.as_ref() {
				return PathedID::Path(path.to_owned());
			}
		}

		PathedID::Unknown(*hash)
	}
}
