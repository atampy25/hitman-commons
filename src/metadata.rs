use std::{
	fmt::{Debug, Display},
	io::{Cursor, Read, Seek, SeekFrom},
	str::{self, FromStr}
};

use ecow::EcoString;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use thiserror::Error;
use tryvial::try_fn;

use crate::{
	game::GameVersion,
	hash_list::{CUSTOM_PATHS, HASH_LIST},
	rpkg_tool::RpkgResourceMeta
};

#[cfg(feature = "rune")]
#[try_fn]
pub fn rune_module() -> Result<rune::Module, rune::ContextError> {
	let mut module = rune::Module::with_crate_item("hitman_commons", ["metadata"])?;

	module.ty::<RuntimeID>()?;
	module.ty::<FromU64Error>()?;
	module.ty::<RuntimeIDFromHashError>()?;
	module.ty::<ResourceReference>()?;
	module.ty::<ReferenceFlags>()?;
	module.ty::<ReferenceType>()?;
	module.ty::<ResourceType>()?;
	module.ty::<ResourceTypeError>()?;
	module.ty::<ResourceMetadata>()?;
	module.ty::<ExtendedResourceMetadata>()?;
	module.ty::<MetadataCalculationError>()?;
	module.ty::<FromRpkgResourceMetaError>()?;

	#[cfg(feature = "rpkg-rs")]
	module.ty::<FromResourceInfoError>()?;

	module
}

#[cfg_attr(
	feature = "serde",
	derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(
	feature = "rune",
	rune_functions(
		Self::from_hash__meta,
		Self::from_path__meta,
		Self::as_u64__meta,
		Self::r_get_path,
		Self::r_from_str,
		Self::r_from_u64
	)
)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", rkyv(derive(Hash, PartialEq, Eq)))]
#[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct RuntimeID(u64);

#[cfg(feature = "specta")]
impl specta::Type for RuntimeID {
	fn inline(_: &mut specta::TypeMap, _: specta::Generics<'_>) -> specta::DataType {
		specta::DataType::Primitive(specta::datatype::PrimitiveType::String)
	}
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for RuntimeID {
	fn schema_name() -> std::borrow::Cow<'static, str> {
		std::borrow::Cow::Borrowed("RuntimeID")
	}

	fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
		schemars::json_schema!({
			"type": "string"
		})
	}
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum FromU64Error {
	#[error("value too high; must be less than 00FFFFFFFFFFFFFF")]
	TooHigh
}

impl TryFrom<u64> for RuntimeID {
	type Error = FromU64Error;

	#[try_fn]
	fn try_from(value: u64) -> Result<Self, Self::Error> {
		if value < 0x00FFFFFFFFFFFFFF {
			Self(value)
		} else {
			return Err(FromU64Error::TooHigh);
		}
	}
}

impl From<RuntimeID> for u64 {
	fn from(val: RuntimeID) -> Self {
		val.0
	}
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum RuntimeIDFromHashError {
	#[error("invalid u64: {0}")]
	InvalidNumber(#[from] std::num::ParseIntError),

	#[error("invalid length")]
	InvalidLength,

	#[error("invalid RuntimeID: {0}")]
	InvalidID(#[from] FromU64Error)
}

impl FromStr for RuntimeID {
	type Err = RuntimeIDFromHashError;

	#[try_fn]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.starts_with('[') {
			RuntimeID::from_path(s)
		} else {
			RuntimeID::from_hash(s)?
		}
	}
}

impl Display for RuntimeID {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		if let Some(data) = HASH_LIST.entries.load().get(self)
			&& let Some(path) = data.path.as_ref()
		{
			write!(f, "{path}")
		} else if let Some(path) = CUSTOM_PATHS.pin().get(self) {
			write!(f, "{path}")
		} else {
			write!(f, "{:016X}", self.0)
		}
	}
}

impl Debug for RuntimeID {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{self}")
	}
}

impl RuntimeID {
	#[try_fn]
	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::from_hash))]
	pub fn from_hash(hash: &str) -> Result<Self, RuntimeIDFromHashError> {
		if hash.len() != 16 {
			return Err(RuntimeIDFromHashError::InvalidLength);
		}

		let val = u64::from_str_radix(hash, 16).map_err(RuntimeIDFromHashError::InvalidNumber)?;
		val.try_into().map_err(RuntimeIDFromHashError::InvalidID)?
	}

	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::from_path))]
	pub fn from_path(path: &str) -> Self {
		let digest = md5::compute(path.to_ascii_lowercase());

		let mut val = 0u64;
		for i in 1..8 {
			val |= u64::from(digest[i]) << (8 * (7 - i));
		}

		let id = Self(val);

		if !HASH_LIST.entries.load().contains_key(&id) {
			CUSTOM_PATHS.pin().get_or_insert_with(id, || path.into());
		}

		id
	}

	pub fn get_path(&self) -> Option<EcoString> {
		HASH_LIST
			.entries
			.load()
			.get(self)
			.and_then(|data| data.path.to_owned())
			.or_else(|| CUSTOM_PATHS.pin().get(self).cloned())
	}

	pub fn to_hash(&self) -> String {
		format!("{:016X}", self.0)
	}

	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::as_u64))]
	pub fn as_u64(self) -> u64 {
		self.0
	}
}

#[cfg(feature = "rune")]
impl RuntimeID {
	#[rune::function(path = Self::from_str)]
	fn r_from_str(s: &str) -> Result<Self, RuntimeIDFromHashError> {
		Self::from_str(s)
	}

	#[rune::function(path = Self::from_u64)]
	fn r_from_u64(val: u64) -> Result<Self, FromU64Error> {
		Self::try_from(val)
	}

	#[rune::function(path = Self::get_path)]
	fn r_get_path(&self) -> Option<String> {
		self.get_path().map(|x| x.into())
	}
}

#[cfg(feature = "rpkg-rs")]
impl TryFrom<rpkg_rs::resource::runtime_resource_id::RuntimeResourceID> for RuntimeID {
	type Error = RuntimeIDFromHashError;

	#[try_fn]
	fn try_from(val: rpkg_rs::resource::runtime_resource_id::RuntimeResourceID) -> Result<Self, Self::Error> {
		// TODO: We should be able to use the u64 directly instead of having to convert to/from a string.
		RuntimeID::from_hash(&val.to_hex_string())?
	}
}

#[cfg(feature = "rpkg-rs")]
impl TryFrom<&rpkg_rs::resource::runtime_resource_id::RuntimeResourceID> for RuntimeID {
	type Error = RuntimeIDFromHashError;

	#[try_fn]
	fn try_from(val: &rpkg_rs::resource::runtime_resource_id::RuntimeResourceID) -> Result<Self, Self::Error> {
		// TODO: We should be able to use the u64 directly instead of having to convert to/from a string.
		RuntimeID::from_hash(&val.to_hex_string())?
	}
}

#[cfg(feature = "rpkg-rs")]
impl From<RuntimeID> for rpkg_rs::resource::runtime_resource_id::RuntimeResourceID {
	fn from(val: RuntimeID) -> rpkg_rs::resource::runtime_resource_id::RuntimeResourceID {
		rpkg_rs::resource::runtime_resource_id::RuntimeResourceID::from(u64::from(val))
	}
}

#[cfg(feature = "rpkg-rs")]
impl From<&RuntimeID> for rpkg_rs::resource::runtime_resource_id::RuntimeResourceID {
	fn from(val: &RuntimeID) -> rpkg_rs::resource::runtime_resource_id::RuntimeResourceID {
		rpkg_rs::resource::runtime_resource_id::RuntimeResourceID::from(u64::from(*val))
	}
}

#[cfg_attr(feature = "rune", serde_with::apply(_ => #[rune(get, set)]))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rune", rune(constructor))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResourceReference {
	pub resource: RuntimeID,
	pub flags: ReferenceFlags
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for ResourceReference {
	fn schema_name() -> std::borrow::Cow<'static, str> {
		"ResourceReference".into()
	}

	fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
		generator.subschema_for::<ResourceReferenceProxy>()
	}
}

#[cfg(feature = "specta")]
impl specta::Type for ResourceReference {
	fn inline(type_map: &mut specta::TypeMap, generics: specta::Generics<'_>) -> specta::DataType {
		ResourceReferenceProxy::inline(type_map, generics)
	}
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
enum ResourceReferenceProxy {
	RuntimeID(RuntimeID),
	ReferenceWithFlags {
		resource: RuntimeID,

		#[cfg_attr(feature = "serde", serde(flatten))]
		flags: ReferenceFlags
	}
}

#[cfg(feature = "serde")]
impl Serialize for ResourceReference {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		if is_default_flags(&self.flags) {
			ResourceReferenceProxy::RuntimeID(self.resource.to_owned()).serialize(serializer)
		} else {
			ResourceReferenceProxy::ReferenceWithFlags {
				resource: self.resource.to_owned(),
				flags: self.flags.to_owned()
			}
			.serialize(serializer)
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ResourceReference {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		ResourceReferenceProxy::deserialize(deserializer).map(|x| match x {
			ResourceReferenceProxy::RuntimeID(resource) => Self {
				resource,
				flags: Default::default()
			},

			ResourceReferenceProxy::ReferenceWithFlags { resource, flags } => Self { resource, flags }
		})
	}
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rune", rune(constructor))]
#[cfg_attr(
	feature = "rune",
	rune_functions(
		Self::r_default,
		Self::from_any__meta,
		Self::from_legacy__meta,
		Self::from_modern__meta,
		Self::as_legacy__meta,
		Self::as_modern__meta
	)
)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ReferenceFlags {
	#[cfg_attr(feature = "serde", serde(default))]
	#[cfg_attr(feature = "serde", serde(rename = "type"))]
	#[cfg_attr(feature = "rune", rune(get, set))]
	pub reference_type: ReferenceType,

	#[cfg_attr(feature = "serde", serde(default))]
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
	#[cfg_attr(feature = "rune", rune(get, set))]
	pub acquired: bool,

	#[cfg_attr(feature = "serde", serde(default = "all_ones"))]
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_all_ones"))]
	#[cfg_attr(feature = "rune", rune(get, set))]
	pub language_code: u8
}

#[cfg(feature = "serde")]
fn is_default_flags(val: &ReferenceFlags) -> bool {
	val.reference_type == ReferenceType::Install && !val.acquired && val.language_code == 0b0001_1111
}

#[cfg(feature = "serde")]
fn is_false(val: &bool) -> bool {
	!val
}

#[cfg(feature = "serde")]
fn all_ones() -> u8 {
	0b0001_1111
}

#[cfg(feature = "serde")]
fn is_all_ones(val: &u8) -> bool {
	*val == 0b0001_1111
}

impl Default for ReferenceFlags {
	fn default() -> Self {
		Self {
			reference_type: ReferenceType::Install,
			acquired: false,
			language_code: 0b0001_1111
		}
	}
}

#[cfg(feature = "rune")]
impl ReferenceFlags {
	#[rune::function(path = Self::default)]
	fn r_default() -> Self {
		Self::default()
	}
}

impl ReferenceFlags {
	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::from_any))]
	pub fn from_any(flag: u8) -> Self {
		// First and fourth bits are padding in the legacy format
		if flag & 0b0000_1001 != 0 {
			Self::from_modern(flag)
		} else {
			let install_dependency = flag & 0b1000_0000 == 0b1000_0000;
			let media_streamed = flag & 0b0100_0000 == 0b0100_0000;
			let state_streamed = flag & 0b0010_0000 == 0b0010_0000;
			let type_of_streaming_entity = flag & 0b0001_0000 == 0b0001_0000;
			let weak_reference = flag & 0b0000_0100 == 0b0000_0100;

			if install_dependency && weak_reference
				|| media_streamed && state_streamed
				|| state_streamed && type_of_streaming_entity
				|| media_streamed && type_of_streaming_entity
			{
				Self::from_modern(flag)
			} else {
				Self::from_legacy(flag)
			}
		}
	}

	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::from_legacy))]
	pub fn from_legacy(flag: u8) -> Self {
		let install_dependency = flag & 0b1000_0000 == 0b1000_0000;
		let media_streamed = flag & 0b0100_0000 == 0b0100_0000;
		let state_streamed = flag & 0b0010_0000 == 0b0010_0000;
		let type_of_streaming_entity = flag & 0b0001_0000 == 0b0001_0000;
		let weak_reference = flag & 0b0000_0100 == 0b0000_0100;
		let runtime_acquired = flag & 0b0000_0010 == 0b0000_0010;

		Self {
			reference_type: if type_of_streaming_entity {
				ReferenceType::EntityType
			} else if install_dependency {
				ReferenceType::Install
			} else if weak_reference {
				ReferenceType::Weak
			} else if media_streamed {
				ReferenceType::Media
			} else if state_streamed {
				ReferenceType::State
			} else {
				ReferenceType::Normal
			},
			acquired: runtime_acquired,
			language_code: 0x1F
		}
	}

	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::from_modern))]
	pub fn from_modern(flag: u8) -> Self {
		Self {
			reference_type: match flag & 0b1100_0000 {
				0b0000_0000 => ReferenceType::Install,
				0b0100_0000 => ReferenceType::Normal,
				0b1000_0000 => ReferenceType::Weak,
				_ => ReferenceType::Normal
			},
			acquired: (flag & 0b0010_0000) != 0,
			language_code: flag & 0b0001_1111
		}
	}

	#[cfg_attr(feature = "rune", rune::function(keep))]
	pub fn as_legacy(&self) -> u8 {
		let mut flag = match self.reference_type {
			ReferenceType::Install => 0b1000_0000,
			ReferenceType::Normal => 0b0000_0000,
			ReferenceType::Weak => 0b0000_0100,
			ReferenceType::Media => 0b0100_0000,
			ReferenceType::State => 0b0010_0000,
			ReferenceType::EntityType => 0b1001_0000
		};

		if self.acquired {
			flag |= 0b0000_0010;
		}

		flag
	}

	#[cfg_attr(feature = "rune", rune::function(keep))]
	pub fn as_modern(&self) -> u8 {
		self.language_code
			| ((self.acquired as u8) << 0x5)
			| ((match self.reference_type {
				ReferenceType::Install => 0,
				ReferenceType::Normal => 1,
				ReferenceType::Weak => 2,
				ReferenceType::Media => 2,
				ReferenceType::State => 1,
				ReferenceType::EntityType => 0
			}) << 0x6)
	}
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum ReferenceType {
	#[default]
	#[cfg_attr(feature = "rune", rune(constructor))]
	Install,

	#[cfg_attr(feature = "rune", rune(constructor))]
	Normal,

	#[cfg_attr(feature = "rune", rune(constructor))]
	Weak,

	#[cfg_attr(feature = "rune", rune(constructor))]
	Media, // same as Weak in modern format

	#[cfg_attr(feature = "rune", rune(constructor))]
	State, // same as Normal in modern format

	#[cfg_attr(feature = "rune", rune(constructor))]
	EntityType // same as Install in modern format
}

/// Core information about a resource.
#[cfg_attr(feature = "rune", serde_with::apply(_ => #[rune(get, set)]))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rune", rune(constructor))]
#[cfg_attr(
	feature = "rune",
	rune_functions(
		Self::infer_scrambled__meta,
		Self::infer_compressed__meta,
		Self::calculate_system_memory_requirement__meta,
		Self::calculate_video_memory_requirement__meta,
		Self::system_memory_requirement__meta,
		Self::video_memory_requirement__meta,
		Self::to_extended__meta
	)
)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResourceMetadata {
	pub id: RuntimeID,
	pub resource_type: ResourceType,
	pub compressed: bool,
	pub scrambled: bool,
	pub references: Vec<ResourceReference>
}

impl ResourceMetadata {
	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::infer_scrambled))]
	pub fn infer_scrambled(resource_type: ResourceType) -> bool {
		match resource_type.as_ref() {
			// Only these types are not scrambled
			"GFXV" | "TEXD" => false,

			_ => true
		}
	}

	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::infer_compressed))]
	pub fn infer_compressed(resource_type: ResourceType) -> bool {
		match resource_type.as_ref() {
			// Always compressed
			"REPO" | "ASVA" | "VIDB" | "GFXF" | "UICB" | "PRIM" | "WSWB" | "DLGE" | "MATI" | "BMSK" | "TBLU"
			| "AIBB" | "YSHP" | "MATE" | "MRTN" | "CRMD" | "WSGB" | "LOCR" | "MJBA" | "NAVP" | "BORG" | "ENUM"
			| "BOXC" | "CPPT" | "ECPB" | "FXAC" | "WBNK" | "ATMD" | "ORES" | "FXAS" | "MRTR" | "RTLV" | "AIBZ"
			| "GIDX" | "AIRG" | "DITL" | "SDEF" | "CBLU" | "TEMP" | "DSWB" | "GFXI" => true,

			// Usually compressed
			"MATB" | "SCDA" | "JSON" | "ALOC" | "MATT" | "VTXD" | "PREL" | "WWEV" => true,

			// Never compressed
			"LINE" | "WWES" | "WWEM" | "TEXT" | "ERES" | "GFXV" | "TEXD" | "WSGT" | "ASET" | "CLNG" | "ECPT"
			| "UICT" | "WSWT" | "AIBX" | "ASEB" => false,

			_ => true
		}
	}

	#[try_fn]
	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::calculate_system_memory_requirement))]
	pub fn calculate_system_memory_requirement(
		resource_type: ResourceType,
		data: &[u8]
	) -> Result<u32, MetadataCalculationError> {
		match resource_type.as_ref() {
			"AIBX" | "AIBZ" | "AIRG" | "ASEB" | "ASET" | "ASVA" | "ATMD" | "BLOB" | "BMSK" | "BORG" | "BOXC"
			| "CRMD" | "DITL" | "DLGE" | "ECPT" | "ENUM" | "ERES" | "GFXF" | "GFXI" | "GFXV" | "JSON" | "LINE"
			| "LOCR" | "MATB" | "MATE" | "MATI" | "MATT" | "NAVP" | "ORES" | "PRIM" | "REPO" | "RTLV" | "SDEF"
			| "TEXD" | "TEXT" | "UICT" | "VIDB" | "VTXD" | "WBNK" | "WSGT" | "WSWT" | "WWEM" | "WWES" | "WWEV"
			| "TELI" | "CLNG" => 0xFFFFFFFF,

			"AIBB" | "CBLU" | "CPPT" | "DSWB" | "ECPB" | "GIDX" | "TEMP" | "TBLU" | "UICB" | "WSGB" | "WSWB" => {
				let mut cur = Cursor::new(data);
				cur.seek(SeekFrom::Start(0x8))?;

				let mut x = [0; 4];
				cur.read_exact(&mut x)?;
				u32::from_be_bytes(x)
			}

			"ALOC" => ((data.len() as f64) * 1.75) as u32,

			"FXAS" | "MJBA" | "MRTN" | "MRTR" | "SCDA" => data.len() as u32,

			"PREL" => (data.len() - 0x10) as u32,

			"YSHP" => ((data.len() as f64) * 1.5) as u32,

			"FXAC" | "HIKC" | "IMAP" | "SLMX" => todo!(),

			_ => return Err(MetadataCalculationError::UnknownResourceType(resource_type))
		}
	}

	#[try_fn]
	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::calculate_video_memory_requirement))]
	pub fn calculate_video_memory_requirement(
		resource_type: ResourceType,
		data: &[u8],
		game_version: GameVersion
	) -> Result<u32, MetadataCalculationError> {
		match resource_type.as_ref() {
			"AIBB" | "AIBX" | "AIBZ" | "AIRG" | "ASEB" | "ASET" | "ASVA" | "ATMD" | "BLOB" | "BMSK" | "BORG"
			| "CBLU" | "CLNG" | "CPPT" | "CRMD" | "DITL" | "DLGE" | "DSWB" | "ECPB" | "ECPT" | "ENUM" | "ERES"
			| "GFXF" | "GFXI" | "GFXV" | "JSON" | "LINE" | "LOCR" | "MATB" | "MATE" | "MATI" | "MATT" | "GIDX"
			| "NAVP" | "ORES" | "PRIM" | "REPO" | "RTLV" | "SDEF" | "TBLU" | "TELI" | "TEMP" | "UICB" | "UICT"
			| "VIDB" | "VTXD" | "WBNK" | "WSGB" | "WSGT" | "WSWB" | "WSWT" | "WWEM" | "WWES" | "WWEV" => 0xFFFFFFFF,

			"ALOC" | "FXAC" | "FXAS" | "MJBA" | "MRTN" | "MRTR" | "PREL" | "SCDA" | "YSHP" => 0,

			"TEXT" => {
				#[cfg(feature = "glacier-texture")]
				{
					// NOTE: This is not fully accurate. In HITMAN 3, the TEXD data is required to calculate this, so it will just return 0.
					glacier_texture::texture_map::TextureMap::from_memory(data, game_version.into())
						.map_err(MetadataCalculationError::TextureMapError)?
						.video_memory_requirement() as u32
				}

				#[cfg(not(feature = "glacier-texture"))]
				{
					// TODO: Calculate a proper estimate
					0
				}
			}

			"TEXD" => {
				#[cfg(feature = "glacier-texture")]
				{
					// This is accurate across game versions.
					glacier_texture::mipblock::MipblockData::from_memory(data, game_version.into())
						.map_err(MetadataCalculationError::TextureMapError)?
						.video_memory_requirement() as u32
				}

				#[cfg(not(feature = "glacier-texture"))]
				{
					// TODO: Calculate a proper estimate
					0
				}
			}

			"BOXC" | "HIKC" | "IMAP" | "SLMX" => todo!(),

			_ => return Err(MetadataCalculationError::UnknownResourceType(resource_type))
		}
	}
}

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct ResourceMetadataProxy {
	id: RuntimeID,

	#[serde(rename = "type")]
	resource_type: ResourceType,

	#[serde(skip_serializing_if = "Option::is_none")]
	compressed: Option<bool>,

	#[serde(skip_serializing_if = "Option::is_none")]
	scrambled: Option<bool>,

	references: Vec<ResourceReference>
}

#[cfg(feature = "serde")]
impl Serialize for ResourceMetadata {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		ResourceMetadataProxy {
			id: self.id.to_owned(),
			resource_type: self.resource_type,
			compressed: (self.compressed != Self::infer_compressed(self.resource_type)).then_some(self.compressed),
			scrambled: (self.scrambled != Self::infer_scrambled(self.resource_type)).then_some(self.scrambled),
			references: self.references.to_owned()
		}
		.serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ResourceMetadata {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		ResourceMetadataProxy::deserialize(deserializer).map(|x| Self {
			id: x.id,
			resource_type: x.resource_type,
			compressed: x.compressed.unwrap_or_else(|| Self::infer_compressed(x.resource_type)),
			scrambled: x.scrambled.unwrap_or_else(|| Self::infer_scrambled(x.resource_type)),
			references: x.references
		})
	}
}

/// Extended information about a resource.
///
/// Where necessary, this information can be computed from the core information and the resource data itself.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "rune", serde_with::apply(_ => #[rune(get, set)]))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rune", rune(constructor))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ExtendedResourceMetadata {
	#[cfg_attr(feature = "serde", serde(flatten))]
	pub core_info: ResourceMetadata,

	pub system_memory_requirement: u32,
	pub video_memory_requirement: u32
}

#[cfg_attr(
	feature = "serde",
	derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_functions(Self::r_from_str, Self::r_as_string))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT, PARTIAL_EQ, EQ, CLONE))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct ResourceType([u8; 4]);

#[cfg(feature = "specta")]
impl specta::Type for ResourceType {
	fn inline(_: &mut specta::TypeMap, _: specta::Generics<'_>) -> specta::DataType {
		specta::DataType::Primitive(specta::datatype::PrimitiveType::String)
	}
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for ResourceType {
	fn schema_name() -> std::borrow::Cow<'static, str> {
		"ResourceType".into()
	}

	fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
		schemars::json_schema!({
			"type": "string",
			"pattern": "^[A-Z]{4}$"
		})
	}
}

impl From<ResourceType> for [u8; 4] {
	fn from(val: ResourceType) -> Self {
		val.0
	}
}

impl From<ResourceType> for Vec<u8> {
	fn from(val: ResourceType) -> Self {
		val.0.to_vec()
	}
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum ResourceTypeError {
	#[error("invalid length")]
	InvalidLength,

	#[error("invalid UTF-8: {0}")]
	InvalidString(#[from] std::string::FromUtf8Error)
}

impl FromStr for ResourceType {
	type Err = ResourceTypeError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		ResourceType::try_from(s)
	}
}

impl TryFrom<String> for ResourceType {
	type Error = ResourceTypeError;

	#[try_fn]
	fn try_from(value: String) -> Result<Self, Self::Error> {
		ResourceType(
			value
				.into_bytes()
				.try_into()
				.map_err(|_| ResourceTypeError::InvalidLength)?
		)
	}
}

impl TryFrom<&str> for ResourceType {
	type Error = ResourceTypeError;

	#[try_fn]
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		ResourceType(
			value
				.as_bytes()
				.try_into()
				.map_err(|_| ResourceTypeError::InvalidLength)?
		)
	}
}

impl From<ResourceType> for String {
	fn from(val: ResourceType) -> Self {
		unsafe { String::from_utf8_unchecked(val.0.into()) }
	}
}

impl TryFrom<[u8; 4]> for ResourceType {
	type Error = ResourceTypeError;

	#[try_fn]
	fn try_from(val: [u8; 4]) -> Result<Self, Self::Error> {
		String::from_utf8(val.to_vec()).map_err(ResourceTypeError::InvalidString)?;

		ResourceType(val)
	}
}

impl AsRef<str> for ResourceType {
	fn as_ref(&self) -> &str {
		unsafe { str::from_utf8_unchecked(&self.0) }
	}
}

impl Debug for ResourceType {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", unsafe { str::from_utf8_unchecked(&self.0) })
	}
}

impl Display for ResourceType {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", unsafe { str::from_utf8_unchecked(&self.0) })
	}
}

impl PartialEq<str> for ResourceType {
	fn eq(&self, other: &str) -> bool {
		self.0 == other.as_bytes()
	}
}

impl PartialEq<&str> for ResourceType {
	fn eq(&self, other: &&str) -> bool {
		self.0 == other.as_bytes()
	}
}

impl PartialEq<String> for ResourceType {
	fn eq(&self, other: &String) -> bool {
		self.0 == other.as_bytes()
	}
}

#[cfg(feature = "rune")]
impl ResourceType {
	#[rune::function(path = Self::from_str)]
	fn r_from_str(s: &str) -> Result<Self, ResourceTypeError> {
		Self::try_from(s)
	}

	#[rune::function(instance, path = Self::as_string)]
	fn r_as_string(&self) -> String {
		(*self).into()
	}
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum MetadataCalculationError {
	#[error("seek error: {0}")]
	Seek(#[from] std::io::Error),

	#[error("unknown resource type {0}")]
	UnknownResourceType(ResourceType),

	#[cfg(feature = "glacier-texture")]
	#[error("texture parsing error: {0}")]
	TextureMapError(glacier_texture::texture_map::TextureMapError)
}

impl ResourceMetadata {
	#[cfg_attr(feature = "rune", rune::function(keep))]
	pub fn system_memory_requirement(&self, data: &[u8]) -> Result<u32, MetadataCalculationError> {
		Self::calculate_system_memory_requirement(self.resource_type, data)
	}

	#[cfg_attr(feature = "rune", rune::function(keep))]
	pub fn video_memory_requirement(
		&self,
		data: &[u8],
		game_version: GameVersion
	) -> Result<u32, MetadataCalculationError> {
		Self::calculate_video_memory_requirement(self.resource_type, data, game_version)
	}

	#[try_fn]
	#[cfg_attr(feature = "rune", rune::function(keep))]
	pub fn to_extended(
		self,
		data: &[u8],
		game_version: GameVersion
	) -> Result<ExtendedResourceMetadata, MetadataCalculationError> {
		ExtendedResourceMetadata {
			system_memory_requirement: self.system_memory_requirement(data)?,
			video_memory_requirement: self.video_memory_requirement(data, game_version)?,
			core_info: self
		}
	}
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum FromRpkgResourceMetaError {
	#[error("invalid flag: {0}")]
	InvalidFlag(#[from] std::num::ParseIntError)
}

impl TryFrom<RpkgResourceMeta> for ResourceMetadata {
	type Error = FromRpkgResourceMetaError;

	#[try_fn]
	fn try_from(meta: RpkgResourceMeta) -> Result<Self, Self::Error> {
		Self {
			id: meta.hash_value,
			resource_type: meta.hash_resource_type,
			compressed: meta.hash_size & 0x7FFFFFFF != 0,
			scrambled: meta.hash_size & 0x80000000 == 0x80000000,
			references: meta
				.hash_reference_data
				.into_iter()
				.map(|x| {
					Ok(ResourceReference {
						resource: x.hash,
						flags: ReferenceFlags::from_any(
							u8::from_str_radix(&x.flag, 16).map_err(FromRpkgResourceMetaError::InvalidFlag)?
						)
					})
				})
				.collect::<Result<_, Self::Error>>()?
		}
	}
}

impl TryFrom<RpkgResourceMeta> for ExtendedResourceMetadata {
	type Error = FromRpkgResourceMetaError;

	#[try_fn]
	fn try_from(meta: RpkgResourceMeta) -> Result<Self, Self::Error> {
		Self {
			core_info: ResourceMetadata {
				id: meta.hash_value,
				resource_type: meta.hash_resource_type,
				compressed: meta.hash_size & 0x7FFFFFFF != 0,
				scrambled: meta.hash_size & 0x80000000 == 0x80000000,
				references: meta
					.hash_reference_data
					.into_iter()
					.map(|x| {
						Ok(ResourceReference {
							resource: x.hash,
							flags: ReferenceFlags::from_any(
								u8::from_str_radix(&x.flag, 16).map_err(FromRpkgResourceMetaError::InvalidFlag)?
							)
						})
					})
					.collect::<Result<_, Self::Error>>()?
			},
			system_memory_requirement: meta.hash_size_in_memory,
			video_memory_requirement: meta.hash_size_in_video_memory
		}
	}
}

#[cfg(feature = "rpkg-rs")]
use rpkg_rs::resource::resource_info::ResourceInfo;

#[cfg(feature = "rpkg-rs")]
#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(DISPLAY_FMT, DEBUG_FMT))]
pub enum FromResourceInfoError {
	#[error("invalid RuntimeID: {0}")]
	InvalidID(#[from] RuntimeIDFromHashError),

	#[error("invalid resource type")]
	InvalidResourceType
}

#[cfg(feature = "rpkg-rs")]
impl TryFrom<&ResourceInfo> for ExtendedResourceMetadata {
	type Error = FromResourceInfoError;

	#[try_fn]
	fn try_from(info: &ResourceInfo) -> Result<ExtendedResourceMetadata, Self::Error> {
		ExtendedResourceMetadata {
			core_info: ResourceMetadata {
				id: RuntimeID::try_from(*info.rrid()).map_err(FromResourceInfoError::InvalidID)?,
				resource_type: info.data_type().try_into().unwrap(),
				references: info
					.references()
					.iter()
					.map(|(id, flags)| {
						Ok::<_, Self::Error>(ResourceReference {
							resource: RuntimeID::try_from(*id).map_err(FromResourceInfoError::InvalidID)?,
							flags: ReferenceFlags {
								reference_type: match flags.reference_type() {
									rpkg_rs::resource::resource_package::ReferenceType::INSTALL => {
										ReferenceType::Install
									}
									rpkg_rs::resource::resource_package::ReferenceType::NORMAL => ReferenceType::Normal,
									rpkg_rs::resource::resource_package::ReferenceType::WEAK => ReferenceType::Weak
								},
								acquired: flags.is_acquired(),
								language_code: flags.language_code()
							}
						})
					})
					.collect::<Result<_, _>>()?,
				compressed: info.is_compressed(),
				scrambled: info.is_scrambled()
			},
			system_memory_requirement: info.system_memory_requirement(),
			video_memory_requirement: info.video_memory_requirement()
		}
	}
}

#[cfg(feature = "rpkg-rs")]
impl TryFrom<&ResourceInfo> for ResourceMetadata {
	type Error = FromResourceInfoError;

	#[try_fn]
	fn try_from(info: &ResourceInfo) -> Result<ResourceMetadata, Self::Error> {
		ResourceMetadata {
			id: RuntimeID::try_from(*info.rrid()).map_err(FromResourceInfoError::InvalidID)?,
			resource_type: info.data_type().try_into().unwrap(),
			references: info
				.references()
				.iter()
				.map(|(id, flags)| {
					Ok::<_, Self::Error>(ResourceReference {
						resource: RuntimeID::try_from(*id).map_err(FromResourceInfoError::InvalidID)?,
						flags: ReferenceFlags {
							reference_type: match flags.reference_type() {
								rpkg_rs::resource::resource_package::ReferenceType::INSTALL => ReferenceType::Install,
								rpkg_rs::resource::resource_package::ReferenceType::NORMAL => ReferenceType::Normal,
								rpkg_rs::resource::resource_package::ReferenceType::WEAK => ReferenceType::Weak
							},
							acquired: flags.is_acquired(),
							language_code: flags.language_code()
						}
					})
				})
				.collect::<Result<_, _>>()?,
			compressed: info.is_compressed(),
			scrambled: info.is_scrambled()
		}
	}
}
