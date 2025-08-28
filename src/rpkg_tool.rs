use std::io::{Cursor, Read};

use thiserror::Error;
use tryvial::try_fn;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::metadata::{
	ExtendedResourceMetadata, FromU64Error, ResourceType, ResourceTypeError, RuntimeID, RuntimeIDFromHashError
};

#[cfg(feature = "rune")]
#[try_fn]
pub fn rune_module() -> Result<rune::Module, rune::ContextError> {
	let mut module = rune::Module::with_crate_item("hitman_commons", ["rpkg_tool"])?;

	module.ty::<RpkgResourceMeta>()?;
	module.ty::<RpkgResourceReference>()?;
	module.ty::<RpkgInteropError>()?;

	module
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "rune", serde_with::apply(_ => #[rune(get, set)]))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::rpkg_tool))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rune", rune_functions(Self::r_new))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RpkgResourceMeta {
	pub hash_offset: u64,
	pub hash_reference_data: Vec<RpkgResourceReference>,
	pub hash_reference_table_dummy: u32,
	pub hash_reference_table_size: u32,
	pub hash_resource_type: ResourceType,
	pub hash_size: u32,
	pub hash_size_final: u32,
	pub hash_size_in_memory: u32,
	pub hash_size_in_video_memory: u32,
	pub hash_value: RuntimeID,

	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub hash_path: Option<RuntimeID>
}

#[cfg(feature = "rune")]
impl RpkgResourceMeta {
	#[rune::function(path = Self::new)]
	pub fn r_new(id: RuntimeID, resource_type: ResourceType) -> Self {
		Self {
			hash_offset: 0,
			hash_reference_data: Vec::new(),
			hash_reference_table_dummy: 0,
			hash_reference_table_size: 0,
			hash_resource_type: resource_type,
			hash_size: 0,
			hash_size_final: 0,
			hash_size_in_memory: 0,
			hash_size_in_video_memory: 0,
			hash_value: id,
			hash_path: None
		}
	}
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::rpkg_tool))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rune", rune(constructor))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RpkgResourceReference {
	#[cfg_attr(feature = "rune", rune(get, set))]
	pub hash: RuntimeID,

	#[cfg_attr(feature = "rune", rune(get, set))]
	pub flag: String
}

type Result<T, E = RpkgInteropError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::rpkg_tool))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum RpkgInteropError {
	#[error("seek error: {0}")]
	Seek(#[from] std::io::Error),

	#[error("invalid number: {0}")]
	InvalidNumber(#[from] std::num::TryFromIntError),

	#[error("invalid hex value: {0}")]
	InvalidHex(#[from] std::num::ParseIntError),

	#[error("invalid RuntimeID: {0}")]
	InvalidHash(#[from] RuntimeIDFromHashError),

	#[error("invalid RuntimeID: {0}")]
	InvalidRuntimeID(#[from] FromU64Error),

	#[error("invalid resource type: {0}")]
	InvalidResourceType(#[from] ResourceTypeError)
}

impl RpkgResourceMeta {
	#[try_fn]
	pub fn from_binary(content: &[u8]) -> Result<Self> {
		let mut cursor = Cursor::new(content);

		let mut hash_value = [0; 8];
		cursor.read_exact(&mut hash_value)?;
		let hash_value = u64::from_le_bytes(hash_value);

		let mut hash_offset = [0; 8];
		cursor.read_exact(&mut hash_offset)?;
		let hash_offset = u64::from_le_bytes(hash_offset);

		let mut hash_size = [0; 4];
		cursor.read_exact(&mut hash_size)?;
		let hash_size = u32::from_le_bytes(hash_size);

		let mut hash_resource_type = [0; 4];
		cursor.read_exact(&mut hash_resource_type)?;
		let hash_resource_type = String::from_utf8_lossy(&hash_resource_type).to_string();

		let mut hash_reference_table_size = [0; 4];
		cursor.read_exact(&mut hash_reference_table_size)?;
		let hash_reference_table_size = u32::from_le_bytes(hash_reference_table_size);

		let mut hash_reference_table_dummy = [0; 4];
		cursor.read_exact(&mut hash_reference_table_dummy)?;
		let hash_reference_table_dummy = u32::from_le_bytes(hash_reference_table_dummy);

		let mut hash_size_final = [0; 4];
		cursor.read_exact(&mut hash_size_final)?;
		let hash_size_final = u32::from_le_bytes(hash_size_final);

		let mut hash_size_in_memory = [0; 4];
		cursor.read_exact(&mut hash_size_in_memory)?;
		let hash_size_in_memory = u32::from_le_bytes(hash_size_in_memory);

		let mut hash_size_in_video_memory = [0; 4];
		cursor.read_exact(&mut hash_size_in_video_memory)?;
		let hash_size_in_video_memory = u32::from_le_bytes(hash_size_in_video_memory);

		let mut dependencies: Vec<RpkgResourceReference> = vec![];

		if hash_reference_table_size != 0 {
			let mut hash_reference_count = [0; 4];
			cursor.read_exact(&mut hash_reference_count)?;
			let hash_reference_count = u32::from_le_bytes(hash_reference_count);
			let hash_reference_count = hash_reference_count & 0x3FFFFFFF;

			let mut flags = vec![];
			let mut references = vec![];

			for _ in 0..hash_reference_count {
				let mut flag = [0; 1];
				cursor.read_exact(&mut flag)?;
				flags.push(flag[0]);
			}

			for _ in 0..hash_reference_count {
				let mut reference = [0; 8];
				cursor.read_exact(&mut reference)?;
				references.push(u64::from_le_bytes(reference));
			}

			dependencies.extend(
				flags
					.iter()
					.zip(references)
					.map(|(flag, reference)| {
						Ok(RpkgResourceReference {
							hash: reference.try_into()?,
							flag: format!("{:X}", flag)
						})
					})
					.collect::<Result<Vec<_>>>()?
			)
		}

		RpkgResourceMeta {
			hash_offset,
			hash_reference_data: dependencies,
			hash_reference_table_dummy,
			hash_reference_table_size,
			hash_resource_type: hash_resource_type.try_into()?,
			hash_size,
			hash_size_final,
			hash_size_in_memory,
			hash_size_in_video_memory,
			hash_value: hash_value.try_into()?,
			hash_path: None
		}
	}

	#[try_fn]
	pub fn to_binary(&self) -> Result<Vec<u8>> {
		let mut data = Vec::with_capacity(44);

		// Note: hash_path is not considered here; this is in line with RPKG Tool's behaviour
		data.extend(self.hash_value.as_u64().to_le_bytes());
		data.extend(self.hash_offset.to_le_bytes());
		data.extend(self.hash_size.to_le_bytes());
		data.extend(self.hash_resource_type.as_ref().as_bytes());

		data.extend(if self.hash_reference_data.is_empty() {
			[0; 4]
		} else {
			u32::try_from(self.hash_reference_data.len() * 9 + 4)?.to_le_bytes()
		}); // Recalculate hash_reference_table_size

		data.extend(self.hash_reference_table_dummy.to_le_bytes());
		data.extend(self.hash_size_final.to_le_bytes());
		data.extend(self.hash_size_in_memory.to_le_bytes());
		data.extend(self.hash_size_in_video_memory.to_le_bytes());

		if !self.hash_reference_data.is_empty() {
			data.extend((u32::try_from(self.hash_reference_data.len())? | 0xC0000000).to_le_bytes());

			for reference in &self.hash_reference_data {
				data.push(u8::from_str_radix(&reference.flag, 16)?);
			}

			for reference in &self.hash_reference_data {
				data.extend(reference.hash.as_u64().to_le_bytes());
			}
		}

		data
	}

	pub fn from_resource_metadata(metadata: ExtendedResourceMetadata, use_legacy_flags: bool) -> Self {
		RpkgResourceMeta {
			hash_offset: 0,
			hash_size: if metadata.core_info.compressed { 1000 } else { 0 }
				| if metadata.core_info.scrambled { 0x80000000 } else { 0x0 },
			hash_size_final: 1000,
			hash_value: metadata.core_info.id,
			hash_path: None,
			hash_size_in_memory: metadata.system_memory_requirement,
			hash_size_in_video_memory: metadata.video_memory_requirement,
			hash_resource_type: metadata.core_info.resource_type,
			hash_reference_data: metadata
				.core_info
				.references
				.iter()
				.map(|reference| RpkgResourceReference {
					flag: format!(
						"{:02X}",
						if use_legacy_flags {
							reference.flags.as_legacy()
						} else {
							reference.flags.as_modern()
						}
					),
					hash: reference.resource
				})
				.collect(),
			hash_reference_table_size: match &metadata.core_info.references.len() {
				0 => 0x0,
				n => 0x4 + (*n as u32 * 0x9)
			},
			hash_reference_table_dummy: 0
		}
	}
}

#[cfg(feature = "rpkg-rs")]
use rpkg_rs::resource::{resource_info::ResourceInfo, resource_package::ResourceReferenceFlags};

#[cfg(feature = "rpkg-rs")]
impl TryFrom<ResourceInfo> for RpkgResourceMeta {
	type Error = RpkgInteropError;

	#[try_fn]
	fn try_from(info: ResourceInfo) -> Result<RpkgResourceMeta> {
		RpkgResourceMeta {
			hash_offset: info.data_offset(),
			hash_size: info.compressed_size().unwrap_or(0) | (if info.is_scrambled() { 0x80000000 } else { 0x0 }),
			hash_size_final: info.size(),
			hash_value: info.rrid().try_into()?,
			hash_path: None,
			hash_size_in_memory: info.system_memory_requirement(),
			hash_size_in_video_memory: info.video_memory_requirement(),
			hash_resource_type: info.data_type().try_into()?,
			hash_reference_data: info
				.references()
				.iter()
				.map(|(hash, flag)| {
					Ok(RpkgResourceReference {
						flag: format!(
							"{:02X}",
							match flag {
								ResourceReferenceFlags::Legacy(x) => x.into_bits(),
								ResourceReferenceFlags::Standard(x) => x.into_bits()
							}
						),
						hash: hash.try_into()?
					})
				})
				.collect::<Result<_>>()?,
			hash_reference_table_size: info.reference_chunk_size() as u32,
			hash_reference_table_dummy: info.states_chunk_size() as u32
		}
	}
}

#[cfg(feature = "rpkg-rs")]
impl TryFrom<&ResourceInfo> for RpkgResourceMeta {
	type Error = RpkgInteropError;

	#[try_fn]
	fn try_from(info: &ResourceInfo) -> Result<RpkgResourceMeta> {
		RpkgResourceMeta {
			hash_offset: info.data_offset(),
			hash_size: info.compressed_size().unwrap_or(0) | (if info.is_scrambled() { 0x80000000 } else { 0x0 }),
			hash_size_final: info.size(),
			hash_value: info.rrid().try_into()?,
			hash_path: None,
			hash_size_in_memory: info.system_memory_requirement(),
			hash_size_in_video_memory: info.video_memory_requirement(),
			hash_resource_type: info.data_type().try_into()?,
			hash_reference_data: info
				.references()
				.iter()
				.map(|(hash, flag)| {
					Ok(RpkgResourceReference {
						flag: format!(
							"{:02X}",
							match flag {
								ResourceReferenceFlags::Legacy(x) => x.into_bits(),
								ResourceReferenceFlags::Standard(x) => x.into_bits()
							}
						),
						hash: hash.try_into()?
					})
				})
				.collect::<Result<_>>()?,
			hash_reference_table_size: info.reference_chunk_size() as u32,
			hash_reference_table_dummy: info.states_chunk_size() as u32
		}
	}
}
