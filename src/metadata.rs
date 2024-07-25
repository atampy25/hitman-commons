use core::{fmt, str};
use std::{
	fmt::{Debug, Display},
	str::FromStr
};

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde_hex::{SerHex, StrictCap};

use thiserror::Error;
use tryvial::try_fn;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct ResourceID(#[cfg_attr(feature = "serde", serde(with = "SerHex::<StrictCap>"))] u64);

#[cfg(feature = "specta")]
impl specta::Type for ResourceID {
	fn inline(_: &mut specta::TypeMap, _: &[specta::DataType]) -> specta::DataType {
		specta::DataType::Primitive(specta::PrimitiveType::String)
	}
}

#[derive(Error, Debug)]
pub enum FromU64Error {
	#[error("value too high to be valid ResourceID")]
	TooHigh
}

impl TryFrom<u64> for ResourceID {
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

impl From<ResourceID> for u64 {
	fn from(val: ResourceID) -> Self {
		val.0
	}
}

#[derive(Error, Debug)]
pub enum FromStrError {
	#[error("invalid u64: {0}")]
	InvalidNumber(#[from] std::num::ParseIntError),

	#[error("invalid length")]
	InvalidLength,

	#[error("invalid ResourceID: {0}")]
	InvalidID(#[from] FromU64Error)
}

impl FromStr for ResourceID {
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

impl Display for ResourceID {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{:016X}", self.0)
	}
}

impl Debug for ResourceID {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{:016X}", self.0)
	}
}

impl ResourceID {
	#[try_fn]
	pub fn from_any(val: &str) -> Result<Self, FromStrError> {
		if val.starts_with('0') {
			ResourceID::from_str(val)?
		} else {
			ResourceID::from_path(val)
		}
	}

	pub fn from_path(path: &str) -> Self {
		let digest = md5::compute(path);

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

#[cfg(feature = "rpkg-rs")]
impl TryFrom<rpkg_rs::resource::runtime_resource_id::RuntimeResourceID> for ResourceID {
	type Error = FromStrError;

	#[try_fn]
	fn try_from(val: rpkg_rs::resource::runtime_resource_id::RuntimeResourceID) -> Result<Self, Self::Error> {
		// TODO: We should be able to use the u64 directly instead of having to convert to/from a string.
		ResourceID::from_str(&val.to_hex_string())?
	}
}

#[cfg(feature = "rpkg-rs")]
impl From<ResourceID> for rpkg_rs::resource::runtime_resource_id::RuntimeResourceID {
	fn from(val: ResourceID) -> rpkg_rs::resource::runtime_resource_id::RuntimeResourceID {
		rpkg_rs::resource::runtime_resource_id::RuntimeResourceID::from(Into::<u64>::into(val))
	}
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResourceReference {
	pub resource: ResourceID,

	#[cfg_attr(feature = "serde", serde(flatten))]
	pub flags: ReferenceFlags
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ReferenceFlags {
	#[cfg_attr(feature = "serde", serde(rename = "type"))]
	pub reference_type: ReferenceType,

	#[cfg_attr(feature = "serde", serde(default))]
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
	pub acquired: bool,

	#[cfg_attr(feature = "serde", serde(default))]
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_zero"))]
	pub language_code: u8
}

#[cfg(feature = "serde")]
fn is_false(val: &bool) -> bool {
	!val
}

#[cfg(feature = "serde")]
fn is_zero(val: &u8) -> bool {
	*val == 0
}

impl ReferenceFlags {
	pub fn from_modern(flag: &u8) -> Self {
		Self {
			reference_type: match flag & 0b1100_0000 {
				0 => ReferenceType::Install,
				1 => ReferenceType::Normal,
				2 => ReferenceType::Weak,
				_ => ReferenceType::Normal
			},
			acquired: (flag & 0b0010_0000) != 0,
			language_code: flag & 0b0001_1111
		}
	}

	pub fn as_modern(&self) -> u8 {
		0x1f | ((self.acquired as u8) << 0x5) | ((self.reference_type as u8) << 0x6)
	}
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ReferenceType {
	Install,
	Normal,
	Weak
}

/// Core information about a resource.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResourceMetadata {
	pub id: ResourceID,

	#[cfg_attr(feature = "serde", serde(rename = "type"))]
	pub resource_type: ResourceType,

	pub references: Vec<ResourceReference>
}

/// Extended information about a resource.
///
/// Where necessary, much of this information can automatically be computed from the core information and the resource data itself.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ExtendedResourceMetadata {
	#[cfg_attr(feature = "serde", serde(flatten))]
	pub core_info: ResourceMetadata,

	pub data_size: u32,
	pub data_offset: u64,
	pub compressed_size_and_is_scrambled_flag: u32,
	pub references_chunk_size: usize,
	pub states_chunk_size: usize,
	pub system_memory_requirement: u32,
	pub video_memory_requirement: u32
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct ResourceType([u8; 4]);

#[cfg(feature = "specta")]
impl specta::Type for ResourceType {
	fn inline(_: &mut specta::TypeMap, _: &[specta::DataType]) -> specta::DataType {
		specta::DataType::Primitive(specta::PrimitiveType::String)
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

#[cfg(feature = "rpkg-rs")]
use rpkg_rs::resource::resource_info::ResourceInfo;

#[cfg(feature = "rpkg-rs")]
#[derive(Error, Debug)]
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
							resource: ResourceID::from_str(&id.to_hex_string())
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
					.collect::<Result<_, _>>()?
			},
			data_size: info.size(),
			data_offset: info.data_offset(),
			compressed_size_and_is_scrambled_flag: (info.compressed_size().unwrap_or(0)
				| (if info.is_scrambled() { 0x80000000 } else { 0x0 })) as u32,
			references_chunk_size: info.reference_chunk_size(),
			states_chunk_size: info.states_chunk_size(),
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
						resource: ResourceID::from_str(&id.to_hex_string())
							.map_err(FromResourceInfoError::InvalidID)?,
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
				.collect::<Result<_, _>>()?
		}
	}
}
