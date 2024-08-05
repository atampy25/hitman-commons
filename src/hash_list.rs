use std::io::Read;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tryvial::try_fn;

use crate::metadata::{ResourceType, RuntimeID};

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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HashList {
	pub version: u32,
	pub entries: HashMap<RuntimeID, HashData>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct HashData {
	pub resource_type: ResourceType,
	pub path: Option<String>,
	pub hint: Option<String>
}

#[derive(Error, Debug)]
pub enum DeserialisationError {
	#[error("decompression failed: {0}")]
	DecompressionFailed(#[from] std::io::Error),

	#[error("deserialisation failed: {0}")]
	DeserialisationFailed(#[from] serde_smile::Error)
}

impl HashList {
	#[try_fn]
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
	pub fn to_path(&self, hash: &RuntimeID) -> String {
		if let Some(entry) = self.entries.get(hash) {
			if let Some(path) = entry.path.as_ref() {
				return path.to_owned();
			}
		}

		hash.to_string()
	}
}
