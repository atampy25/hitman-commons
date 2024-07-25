#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use serde_json::Value;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BlueprintSubEntity {
	pub logical_parent: EntityReference,
	pub entity_type_resource_index: usize,

	#[cfg_attr(feature = "serde", serde(rename = "entityId"))]
	pub entity_id: u64,

	pub editor_only: bool,
	pub entity_name: String,
	pub property_aliases: Vec<PropertyAlias>,
	pub exposed_entities: Vec<ExposedEntity>,
	pub exposed_interfaces: Vec<(String, usize)>,
	pub entity_subsets: Vec<(String, EntitySubset)>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EntityBlueprint {
	pub sub_type: i8,
	pub root_entity_index: usize,
	pub sub_entities: Vec<BlueprintSubEntity>,
	pub external_scene_type_indices_in_resource_header: Vec<usize>,
	pub pin_connections: Vec<PinConnection>,
	pub input_pin_forwardings: Vec<PinConnection>,
	pub output_pin_forwardings: Vec<PinConnection>,
	pub override_deletes: Vec<EntityReference>,
	pub pin_connection_overrides: Vec<ExternalPinConnection>,
	pub pin_connection_override_deletes: Vec<ExternalPinConnection>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FactorySubEntity {
	pub logical_parent: EntityReference,
	pub entity_type_resource_index: usize,
	pub property_values: Vec<Property>,
	pub post_init_property_values: Vec<Property>,

	#[cfg_attr(feature = "serde", serde(default = "Vec::new"))] // H2 does not have this property
	pub platform_specific_property_values: Vec<PlatformSpecificProperty>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EntityFactory {
	pub sub_type: i8,
	pub blueprint_index_in_resource_header: i32,
	pub root_entity_index: usize,
	pub sub_entities: Vec<FactorySubEntity>,
	pub property_overrides: Vec<PropertyOverride>,
	pub external_scene_type_indices_in_resource_header: Vec<usize>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EntityReference {
	#[cfg_attr(feature = "serde", serde(rename = "entityID"))]
	pub entity_id: u64,

	pub external_scene_index: i32,
	pub entity_index: i32,
	pub exposed_entity: String
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ExposedEntity {
	pub s_name: String,
	pub b_is_array: bool,
	pub a_targets: Vec<EntityReference>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PinConnection {
	#[cfg_attr(feature = "serde", serde(rename = "fromID"))]
	pub from_id: usize,

	#[cfg_attr(feature = "serde", serde(rename = "toID"))]
	pub to_id: usize,

	pub from_pin_name: String,
	pub to_pin_name: String,
	pub constant_pin_value: PropertyValue
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PlatformSpecificProperty {
	pub property_value: Property,
	pub platform: String,
	pub post_init: bool
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PropertyAlias {
	pub s_alias_name: String,

	#[cfg_attr(feature = "serde", serde(rename = "entityID"))]
	pub entity_id: usize,

	pub s_property_name: String
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PropertyOverride {
	pub property_owner: EntityReference,
	pub property_value: Property
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EntitySubset {
	pub entities: Vec<usize>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ExternalPinConnection {
	pub from_entity: EntityReference,
	pub to_entity: EntityReference,
	pub from_pin_name: String,
	pub to_pin_name: String,
	pub constant_pin_value: PropertyValue
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Property {
	#[cfg_attr(feature = "serde", serde(rename = "nPropertyID"))]
	pub n_property_id: PropertyID,

	#[cfg_attr(feature = "serde", serde(rename = "value"))]
	pub value: PropertyValue
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PropertyValue {
	#[cfg_attr(feature = "serde", serde(rename = "$type"))]
	pub property_type: String,

	#[cfg_attr(feature = "serde", serde(rename = "$val"))]
	pub property_value: Value
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum PropertyID {
	Int(u64),
	String(String)
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FactorySubEntityLegacy {
	pub logical_parent: EntityReference,
	pub entity_type_resource_index: usize,
	pub property_values: Vec<Property>,
	pub post_init_property_values: Vec<Property>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EntityFactoryLegacy {
	pub sub_type: i8,
	pub blueprint_index_in_resource_header: i32,
	pub root_entity_index: usize,
	pub entity_templates: Vec<FactorySubEntityLegacy>,
	pub property_overrides: Vec<PropertyOverride>,
	pub external_scene_type_indices_in_resource_header: Vec<usize>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BlueprintSubEntityLegacy {
	pub logical_parent: EntityReference,
	pub entity_type_resource_index: usize,
	pub entity_id: u64,
	pub entity_name: String,
	pub property_aliases: Vec<PropertyAlias>,
	pub exposed_entities: Vec<(String, EntityReference)>,
	pub exposed_interfaces: Vec<(String, usize)>,
	pub entity_subsets: Vec<(String, EntitySubset)>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EntityBlueprintLegacy {
	pub sub_type: i8,
	pub root_entity_index: usize,
	pub entity_templates: Vec<BlueprintSubEntityLegacy>,
	pub external_scene_type_indices_in_resource_header: Vec<usize>,
	pub pin_connections: Vec<PinConnectionLegacy>,
	pub input_pin_forwardings: Vec<PinConnectionLegacy>,
	pub output_pin_forwardings: Vec<PinConnectionLegacy>,
	pub override_deletes: Vec<EntityReference>
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PinConnectionLegacy {
	#[cfg_attr(feature = "serde", serde(rename = "fromID"))]
	pub from_id: usize,

	#[cfg_attr(feature = "serde", serde(rename = "toID"))]
	pub to_id: usize,

	pub from_pin_name: String,
	pub to_pin_name: String
}

impl EntityFactoryLegacy {
	pub fn into_modern(self) -> EntityFactory {
		EntityFactory {
			sub_type: self.sub_type,
			blueprint_index_in_resource_header: self.blueprint_index_in_resource_header,
			root_entity_index: self.root_entity_index,
			sub_entities: self
				.entity_templates
				.into_iter()
				.map(|x| FactorySubEntity {
					entity_type_resource_index: x.entity_type_resource_index,
					logical_parent: x.logical_parent,
					platform_specific_property_values: Vec::with_capacity(0),
					property_values: x.property_values,
					post_init_property_values: x.post_init_property_values
				})
				.collect(),
			property_overrides: self.property_overrides,
			external_scene_type_indices_in_resource_header: self.external_scene_type_indices_in_resource_header
		}
	}
}

impl EntityFactory {
	pub fn into_legacy(self) -> EntityFactoryLegacy {
		EntityFactoryLegacy {
			sub_type: self.sub_type,
			blueprint_index_in_resource_header: self.blueprint_index_in_resource_header,
			root_entity_index: self.root_entity_index,
			entity_templates: self
				.sub_entities
				.into_iter()
				.map(|x| FactorySubEntityLegacy {
					entity_type_resource_index: x.entity_type_resource_index,
					logical_parent: x.logical_parent,
					property_values: x.property_values,
					post_init_property_values: x.post_init_property_values
				})
				.collect(),
			property_overrides: self.property_overrides,
			external_scene_type_indices_in_resource_header: self.external_scene_type_indices_in_resource_header
		}
	}
}

impl EntityBlueprintLegacy {
	pub fn into_modern(self) -> EntityBlueprint {
		EntityBlueprint {
			sub_type: self.sub_type,
			root_entity_index: self.root_entity_index,
			sub_entities: self
				.entity_templates
				.into_iter()
				.map(|x| BlueprintSubEntity {
					entity_id: x.entity_id,
					editor_only: false,
					entity_name: x.entity_name,
					entity_subsets: x.entity_subsets,
					entity_type_resource_index: x.entity_type_resource_index,
					exposed_entities: x
						.exposed_entities
						.into_iter()
						.map(|(x, y)| ExposedEntity {
							b_is_array: false,
							a_targets: vec![y],
							s_name: x
						})
						.collect(),
					exposed_interfaces: x.exposed_interfaces,
					logical_parent: x.logical_parent,
					property_aliases: x.property_aliases
				})
				.collect(),
			external_scene_type_indices_in_resource_header: self.external_scene_type_indices_in_resource_header,
			pin_connections: self
				.pin_connections
				.into_iter()
				.map(|x| PinConnection {
					from_id: x.from_id,
					from_pin_name: x.from_pin_name,
					to_id: x.to_id,
					to_pin_name: x.to_pin_name,
					constant_pin_value: PropertyValue {
						property_type: "void".to_string(),
						property_value: Value::Null
					}
				})
				.collect(),
			input_pin_forwardings: self
				.input_pin_forwardings
				.into_iter()
				.map(|x| PinConnection {
					from_id: x.from_id,
					from_pin_name: x.from_pin_name,
					to_id: x.to_id,
					to_pin_name: x.to_pin_name,
					constant_pin_value: PropertyValue {
						property_type: "void".to_string(),
						property_value: Value::Null
					}
				})
				.collect(),
			output_pin_forwardings: self
				.output_pin_forwardings
				.into_iter()
				.map(|x| PinConnection {
					from_id: x.from_id,
					from_pin_name: x.from_pin_name,
					to_id: x.to_id,
					to_pin_name: x.to_pin_name,
					constant_pin_value: PropertyValue {
						property_type: "void".to_string(),
						property_value: Value::Null
					}
				})
				.collect(),
			override_deletes: self.override_deletes,
			pin_connection_overrides: Vec::with_capacity(0),
			pin_connection_override_deletes: Vec::with_capacity(0)
		}
	}
}

impl EntityBlueprint {
	pub fn into_legacy(self) -> EntityBlueprintLegacy {
		EntityBlueprintLegacy {
			sub_type: self.sub_type,
			root_entity_index: self.root_entity_index,
			entity_templates: self
				.sub_entities
				.into_iter()
				.map(|x| BlueprintSubEntityLegacy {
					entity_id: x.entity_id,
					entity_name: x.entity_name,
					entity_subsets: x.entity_subsets,
					entity_type_resource_index: x.entity_type_resource_index,
					exposed_entities: x
						.exposed_entities
						.into_iter()
						.filter(|x| x.a_targets.len() == 1)
						.map(|mut x| (x.s_name, x.a_targets.remove(0)))
						.collect(),
					exposed_interfaces: x.exposed_interfaces,
					logical_parent: x.logical_parent,
					property_aliases: x.property_aliases
				})
				.collect(),
			external_scene_type_indices_in_resource_header: self.external_scene_type_indices_in_resource_header,
			pin_connections: self
				.pin_connections
				.into_iter()
				.map(|x| PinConnectionLegacy {
					from_id: x.from_id,
					from_pin_name: x.from_pin_name,
					to_id: x.to_id,
					to_pin_name: x.to_pin_name
				})
				.collect(),
			input_pin_forwardings: self
				.input_pin_forwardings
				.into_iter()
				.map(|x| PinConnectionLegacy {
					from_id: x.from_id,
					from_pin_name: x.from_pin_name,
					to_id: x.to_id,
					to_pin_name: x.to_pin_name
				})
				.collect(),
			output_pin_forwardings: self
				.output_pin_forwardings
				.into_iter()
				.map(|x| PinConnectionLegacy {
					from_id: x.from_id,
					from_pin_name: x.from_pin_name,
					to_id: x.to_id,
					to_pin_name: x.to_pin_name
				})
				.collect(),
			override_deletes: self.override_deletes
		}
	}
}
