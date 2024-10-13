use core::{fmt, str};
use std::{
	fmt::{Debug, Display},
	io::{Cursor, Read, Seek, SeekFrom},
	str::FromStr
};

#[cfg(feature = "serde")]
use serde::{de::Visitor, ser::SerializeStruct, Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde_hex::{SerHex, StrictCap};

use thiserror::Error;
use tryvial::try_fn;

use crate::rpkg_tool::{RpkgInteropError, RpkgResourceMeta};

#[cfg(feature = "rune")]
#[try_fn]
pub fn rune_module() -> Result<rune::Module, rune::ContextError> {
	let mut module = rune::Module::with_crate_item("hitman_commons", ["metadata"])?;

	module.ty::<RuntimeID>()?;
	module.ty::<FromU64Error>()?;
	module.ty::<FromStrError>()?;
	module.ty::<ResourceReference>()?;
	module.ty::<ReferenceFlags>()?;
	module.ty::<ReferenceType>()?;
	module.ty::<ResourceTypeError>()?;
	module.ty::<ResourceMetadata>()?;
	module.ty::<ExtendedResourceMetadata>()?;
	module.ty::<MetadataCalculationError>()?;
	module.ty::<FromRpkgResourceMetaError>()?;

	#[cfg(feature = "rpkg-rs")]
	module.ty::<FromResourceInfoError>()?;

	module
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
#[cfg_attr(
	feature = "rune",
	rune_functions(
		Self::from_any__meta,
		Self::from_path__meta,
		Self::r_from_str,
		Self::r_from_u64,
		Self::r_as_u64
	)
)]
#[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct RuntimeID(#[cfg_attr(feature = "serde", serde(with = "SerHex::<StrictCap>"))] u64);

#[cfg(feature = "specta")]
impl specta::Type for RuntimeID {
	fn inline(_: &mut specta::TypeMap, _: &[specta::DataType]) -> specta::DataType {
		specta::DataType::Primitive(specta::PrimitiveType::String)
	}
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for RuntimeID {
	fn schema_name() -> String {
		"RuntimeID".to_owned()
	}

	fn schema_id() -> std::borrow::Cow<'static, str> {
		std::borrow::Cow::Borrowed("RuntimeID")
	}

	fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
		schemars::schema::SchemaObject {
			instance_type: Some(schemars::schema::InstanceType::String.into()),
			string: Some(Box::new(schemars::schema::StringValidation {
				pattern: Some(r"^00[0-9A-F]{14}$".to_owned()),
				..Default::default()
			})),
			..Default::default()
		}
		.into()
	}
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune(constructor))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
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
#[cfg_attr(feature = "rune", rune(constructor))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
pub enum FromStrError {
	#[error("invalid u64: {0}")]
	InvalidNumber(#[from] std::num::ParseIntError),

	#[error("invalid length")]
	InvalidLength,

	#[error("invalid RuntimeID: {0}")]
	InvalidID(#[from] FromU64Error)
}

impl FromStr for RuntimeID {
	type Err = FromStrError;

	#[try_fn]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.len() != 16 {
			return Err(FromStrError::InvalidLength);
		}

		let val = u64::from_str_radix(s, 16).map_err(FromStrError::InvalidNumber)?;
		val.try_into().map_err(FromStrError::InvalidID)?
	}
}

impl Display for RuntimeID {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{:016X}", self.0)
	}
}

impl Debug for RuntimeID {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{:016X}", self.0)
	}
}

impl RuntimeID {
	#[try_fn]
	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::from_any))]
	pub fn from_any(val: &str) -> Result<Self, FromStrError> {
		if val.starts_with('0') {
			RuntimeID::from_str(val)?
		} else {
			RuntimeID::from_path(val)
		}
	}

	#[cfg_attr(feature = "rune", rune::function(keep, path = Self::from_path))]
	pub fn from_path(path: &str) -> Self {
		let digest = md5::compute(path.to_ascii_lowercase());

		let mut val = 0u64;
		for i in 1..8 {
			val |= u64::from(digest[i]) << (8 * (7 - i));
		}

		Self(val)
	}

	pub fn as_u64(&self) -> &u64 {
		&self.0
	}
}

#[cfg(feature = "rune")]
impl RuntimeID {
	#[rune::function(path = Self::from_str)]
	fn r_from_str(s: &str) -> Result<Self, FromStrError> {
		Self::from_str(s)
	}

	#[rune::function(path = Self::from_u64)]
	fn r_from_u64(val: u64) -> Result<Self, FromU64Error> {
		Self::try_from(val)
	}

	#[rune::function(path = Self::as_u64)]
	fn r_as_u64(&self) -> u64 {
		self.0
	}
}

#[cfg(feature = "rpkg-rs")]
impl TryFrom<rpkg_rs::resource::runtime_resource_id::RuntimeResourceID> for RuntimeID {
	type Error = FromStrError;

	#[try_fn]
	fn try_from(val: rpkg_rs::resource::runtime_resource_id::RuntimeResourceID) -> Result<Self, Self::Error> {
		// TODO: We should be able to use the u64 directly instead of having to convert to/from a string.
		RuntimeID::from_str(&val.to_hex_string())?
	}
}

#[cfg(feature = "rpkg-rs")]
impl From<RuntimeID> for rpkg_rs::resource::runtime_resource_id::RuntimeResourceID {
	fn from(val: RuntimeID) -> rpkg_rs::resource::runtime_resource_id::RuntimeResourceID {
		rpkg_rs::resource::runtime_resource_id::RuntimeResourceID::from(Into::<u64>::into(val))
	}
}

#[cfg_attr(feature = "rune", serde_with::apply(_ => #[rune(get, set)]))]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DEBUG))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResourceReference {
	pub resource: RuntimeID,
	pub flags: ReferenceFlags
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for ResourceReference {
	fn schema_name() -> String {
		"ResourceReference".to_owned()
	}

	fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
		// Either a RuntimeID or a ReferenceWithFlags object

		gen.subschema_for::<ResourceReferenceProxy>()
	}
}

#[cfg(feature = "specta")]
impl specta::Type for ResourceReference {
	fn inline(type_map: &mut specta::TypeMap, generics: &[specta::DataType]) -> specta::DataType {
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
			ResourceReferenceProxy::RuntimeID(self.resource).serialize(serializer)
		} else {
			ResourceReferenceProxy::ReferenceWithFlags {
				resource: self.resource,
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
#[cfg_attr(feature = "rune", rune_derive(STRING_DEBUG))]
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

impl ReferenceFlags {
	#[cfg_attr(feature = "rune", rune::function(path = Self::default))]
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
#[cfg_attr(feature = "rune", rune_derive(STRING_DEBUG))]
#[cfg_attr(feature = "rune", rune(constructor))]
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
#[cfg_attr(feature = "rune", rune_derive(STRING_DEBUG))]
#[cfg_attr(
	feature = "rune",
	rune_functions(Self::infer_scrambled__meta, Self::infer_compressed__meta, Self::to_extended__meta)
)]
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
}

#[cfg(feature = "serde")]
impl Serialize for ResourceMetadata {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		let mut state = serializer.serialize_struct("ResourceMetadata", 5)?;
		state.serialize_field("id", &self.id)?;
		state.serialize_field("type", &self.resource_type)?;
		state.serialize_field("references", &self.references)?;

		if self.scrambled != Self::infer_scrambled(self.resource_type) {
			state.serialize_field("scrambled", &self.scrambled)?;
		} else {
			state.skip_field("scrambled")?;
		}

		if self.compressed != Self::infer_compressed(self.resource_type) {
			state.serialize_field("compressed", &self.compressed)?;
		} else {
			state.skip_field("compressed")?;
		}

		state.end()
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ResourceMetadata {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		deserializer.deserialize_struct(
			"ResourceMetadata",
			&["id", "type", "references", "scrambled", "compressed"],
			ResMetaVisitor
		)
	}
}

#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "camelCase")]
enum ResMetaField {
	Id,
	Type,
	References,
	Scrambled,
	Compressed
}

#[cfg(feature = "serde")]
struct ResMetaVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for ResMetaVisitor {
	type Value = ResourceMetadata;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("an object containing a resource's metadata")
	}

	#[try_fn]
	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: serde::de::MapAccess<'de>
	{
		let mut id = None;
		let mut resource_type = None;
		let mut references = None;
		let mut scrambled = None;
		let mut compressed = None;
		while let Some(key) = map.next_key()? {
			match key {
				ResMetaField::Id => {
					if id.is_some() {
						return Err(serde::de::Error::duplicate_field("id"));
					}

					id = Some(map.next_value()?);
				}

				ResMetaField::Type => {
					if resource_type.is_some() {
						return Err(serde::de::Error::duplicate_field("type"));
					}

					resource_type = Some(map.next_value()?);
				}

				ResMetaField::References => {
					if references.is_some() {
						return Err(serde::de::Error::duplicate_field("references"));
					}

					references = Some(map.next_value()?);
				}

				ResMetaField::Scrambled => {
					if scrambled.is_some() {
						return Err(serde::de::Error::duplicate_field("scrambled"));
					}

					scrambled = Some(map.next_value()?);
				}

				ResMetaField::Compressed => {
					if compressed.is_some() {
						return Err(serde::de::Error::duplicate_field("compressed"));
					}

					compressed = Some(map.next_value()?);
				}
			}
		}

		let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
		let resource_type = resource_type.ok_or_else(|| serde::de::Error::missing_field("type"))?;
		let references = references.ok_or_else(|| serde::de::Error::missing_field("references"))?;
		let scrambled = scrambled.unwrap_or_else(|| ResourceMetadata::infer_scrambled(resource_type));
		let compressed = compressed.unwrap_or_else(|| ResourceMetadata::infer_compressed(resource_type));

		ResourceMetadata {
			id,
			resource_type,
			references,
			scrambled,
			compressed
		}
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
#[cfg_attr(feature = "rune", rune_derive(STRING_DEBUG))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ExtendedResourceMetadata {
	#[cfg_attr(feature = "serde", serde(flatten))]
	pub core_info: ResourceMetadata,

	pub system_memory_requirement: u32,
	pub video_memory_requirement: u32
}

#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct ResourceType([u8; 4]);

#[cfg(feature = "specta")]
impl specta::Type for ResourceType {
	fn inline(_: &mut specta::TypeMap, _: &[specta::DataType]) -> specta::DataType {
		specta::DataType::Primitive(specta::PrimitiveType::String)
	}
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for ResourceType {
	fn schema_name() -> String {
		"ResourceType".to_owned()
	}

	fn schema_id() -> std::borrow::Cow<'static, str> {
		std::borrow::Cow::Borrowed("ResourceType")
	}

	fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
		schemars::schema::SchemaObject {
			instance_type: Some(schemars::schema::InstanceType::String.into()),
			string: Some(Box::new(schemars::schema::StringValidation {
				pattern: Some(r"^[A-Z]{4}$".to_owned()),
				..Default::default()
			})),
			..Default::default()
		}
		.into()
	}
}

#[cfg(feature = "serde")]
impl Serialize for ResourceType {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		unsafe { serializer.serialize_str(str::from_utf8_unchecked(&self.0)) }
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ResourceType {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		deserializer.deserialize_string(ResTypeVisitor)
	}
}

#[cfg(feature = "serde")]
struct ResTypeVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for ResTypeVisitor {
	type Value = ResourceType;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a 4-character string representing a resource type")
	}

	#[try_fn]
	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: serde::de::Error
	{
		ResourceType(v.into_bytes().try_into().map_err(|_| E::custom("invalid length"))?)
	}

	#[try_fn]
	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error
	{
		ResourceType(v.as_bytes().try_into().map_err(|_| E::custom("invalid length"))?)
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
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
#[cfg_attr(feature = "rune", rune(constructor))]
pub enum ResourceTypeError {
	#[error("invalid length")]
	InvalidLength,

	#[error("invalid UTF-8: {0}")]
	InvalidString(#[from] std::string::FromUtf8Error)
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

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
#[cfg_attr(feature = "rune", rune(constructor))]
pub enum MetadataCalculationError {
	#[error("seek error: {0}")]
	Seek(#[from] std::io::Error),

	#[error("unknown resource type {0}")]
	UnknownResourceType(ResourceType)
}

impl ResourceMetadata {
	#[try_fn]
	#[cfg_attr(feature = "rune", rune::function(keep))]
	pub fn to_extended(self, data: &[u8]) -> Result<ExtendedResourceMetadata, MetadataCalculationError> {
		ExtendedResourceMetadata {
			system_memory_requirement: match self.resource_type.as_ref() {
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

				_ => return Err(MetadataCalculationError::UnknownResourceType(self.resource_type))
			},
			video_memory_requirement: match self.resource_type.as_ref() {
				"AIBB" | "AIBX" | "AIBZ" | "AIRG" | "ASEB" | "ASET" | "ASVA" | "ATMD" | "BLOB" | "BMSK" | "BORG"
				| "CBLU" | "CLNG" | "CPPT" | "CRMD" | "DITL" | "DLGE" | "DSWB" | "ECPB" | "ECPT" | "ENUM" | "ERES"
				| "GFXF" | "GFXI" | "GFXV" | "JSON" | "LINE" | "LOCR" | "MATB" | "MATE" | "MATI" | "MATT" | "GIDX"
				| "NAVP" | "ORES" | "PRIM" | "REPO" | "RTLV" | "SDEF" | "TBLU" | "TELI" | "TEMP" | "UICB" | "UICT"
				| "VIDB" | "VTXD" | "WBNK" | "WSGB" | "WSGT" | "WSWB" | "WSWT" | "WWEM" | "WWES" | "WWEV" => 0xFFFFFFFF,

				"ALOC" | "FXAC" | "FXAS" | "MJBA" | "MRTN" | "MRTR" | "PREL" | "SCDA" | "YSHP" => 0,

				"BOXC" | "HIKC" | "IMAP" | "SLMX" | "TEXD" | "TEXT" => todo!(),

				_ => return Err(MetadataCalculationError::UnknownResourceType(self.resource_type))
			},
			core_info: self
		}
	}
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "rune", derive(better_rune_derive::Any))]
#[cfg_attr(feature = "rune", rune(item = ::hitman_commons::metadata))]
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
#[cfg_attr(feature = "rune", rune(constructor))]
pub enum FromRpkgResourceMetaError {
	#[error("couldn't normalise hashes: {0}")]
	HashNormalisation(RpkgInteropError),

	#[error("invalid ResourceID: {0}")]
	InvalidID(#[from] FromStrError),

	#[error("invalid flag: {0}")]
	InvalidFlag(#[from] std::num::ParseIntError),

	#[error("invalid resource type: {0}")]
	InvalidType(#[from] ResourceTypeError)
}

impl TryFrom<RpkgResourceMeta> for ResourceMetadata {
	type Error = FromRpkgResourceMetaError;

	#[try_fn]
	fn try_from(mut meta: RpkgResourceMeta) -> Result<Self, Self::Error> {
		meta.normalise_hashes()
			.map_err(FromRpkgResourceMetaError::HashNormalisation)?;

		Self {
			id: meta.hash_value.parse().map_err(FromRpkgResourceMetaError::InvalidID)?,
			resource_type: meta.hash_resource_type.try_into()?,
			compressed: meta.hash_size & 0x7FFFFFFF != 0,
			scrambled: meta.hash_size & 0x80000000 == 0x80000000,
			references: meta
				.hash_reference_data
				.into_iter()
				.map(|x| {
					Ok(ResourceReference {
						resource: x.hash.parse().map_err(FromRpkgResourceMetaError::InvalidID)?,
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
	fn try_from(mut meta: RpkgResourceMeta) -> Result<Self, Self::Error> {
		meta.normalise_hashes()
			.map_err(FromRpkgResourceMetaError::HashNormalisation)?;

		Self {
			core_info: ResourceMetadata {
				id: meta.hash_value.parse().map_err(FromRpkgResourceMetaError::InvalidID)?,
				resource_type: meta.hash_resource_type.try_into()?,
				compressed: meta.hash_size & 0x7FFFFFFF != 0,
				scrambled: meta.hash_size & 0x80000000 == 0x80000000,
				references: meta
					.hash_reference_data
					.into_iter()
					.map(|x| {
						Ok(ResourceReference {
							resource: x.hash.parse().map_err(FromRpkgResourceMetaError::InvalidID)?,
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
#[cfg_attr(feature = "rune", rune_derive(STRING_DISPLAY, STRING_DEBUG))]
#[cfg_attr(feature = "rune", rune(constructor))]
pub enum FromResourceInfoError {
	#[error("invalid ResourceID: {0}")]
	InvalidID(#[from] FromStrError),

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
				id: (*info.rrid()).try_into().map_err(FromResourceInfoError::InvalidID)?,
				resource_type: info.data_type().try_into().unwrap(),
				references: info
					.references()
					.iter()
					.map(|(id, flags)| {
						Ok::<_, Self::Error>(ResourceReference {
							resource: RuntimeID::from_str(&id.to_hex_string())
								.map_err(FromResourceInfoError::InvalidID)?,
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
			id: (*info.rrid()).try_into().map_err(FromResourceInfoError::InvalidID)?,
			resource_type: info.data_type().try_into().unwrap(),
			references: info
				.references()
				.iter()
				.map(|(id, flags)| {
					Ok::<_, Self::Error>(ResourceReference {
						resource: RuntimeID::from_str(&id.to_hex_string()).map_err(FromResourceInfoError::InvalidID)?,
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
