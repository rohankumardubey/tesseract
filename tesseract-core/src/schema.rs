use serde_derive::Serialize;
use std::convert::From;
use failure::{Error, format_err};

pub mod aggregator;
pub mod metadata;
mod json;
mod xml;

pub use crate::schema::{
    json::SchemaConfigJson,
    json::DimensionConfigJson,
    json::HierarchyConfigJson,
    json::LevelConfigJson,
    json::MeasureConfigJson,
    json::TableConfigJson,
    json::PropertyConfigJson,
    json::AnnotationConfigJson,
    xml::SchemaConfigXML,
    xml::DimensionConfigXML,
    xml::HierarchyConfigXML,
    xml::LevelConfigXML,
    xml::MeasureConfigXML,
    xml::TableConfigXML,
    xml::PropertyConfigXML,
};
use crate::names::{LevelName, Measure as MeasureName};
use crate::query_ir::MemberType;
pub use self::aggregator::Aggregator;


#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Schema {
    pub name: String,
    pub cubes: Vec<Cube>,
    pub annotations: Option<Vec<Annotation>>,
}

impl From<SchemaConfigJson> for Schema {
    fn from(schema_config: SchemaConfigJson) -> Self {
        // TODO
        // check for:
        // - duplicate cube names,
        // - duplicate dim names

        let mut cubes = vec![];

        for cube_config in schema_config.cubes {
            let mut dimensions: Vec<_> = cube_config.dimensions.into_iter()
                .map(|dim| dim.into())
                .collect();
            let measures = cube_config.measures.into_iter()
                .map(|mea| mea.into())
                .collect();
            let cube_annotations = cube_config.annotations
                .map(|anns| {
                    anns.into_iter()
                        .map(|ann| ann.into())
                        .collect()
                });

            // special case: check for dimension_usages
            if let Some(dim_usages) = cube_config.dimension_usages {
                for dim_usage in dim_usages {
                    // prep annotations to be merged with shared dim annotations
                    let dim_usage_annotations: Option<Vec<Annotation>> = dim_usage.annotations
                        .map(|anns| {
                            anns.into_iter()
                                .map(|ann| ann.into())
                                .collect()
                        });

                    if let Some(ref shared_dims) = schema_config.shared_dimensions {
                        for shared_dim_config in shared_dims {
                            if dim_usage.name == shared_dim_config.name {
                                let hierarchies = shared_dim_config.hierarchies.iter()
                                    .map(|h| h.clone().into())
                                    .collect();
                                let shared_dim_annotations: Option<Vec<Annotation>> = shared_dim_config.annotations.as_ref()
                                    .map(|anns| {
                                        anns.into_iter()
                                            .map(|ann| ann.clone().into())
                                            .collect()
                                    });
                                let dim_annotations = shared_dim_annotations
                                    .and_then(|shared_dim_anns| {
                                        dim_usage_annotations.as_ref().map(|dim_usage_anns| {
                                            let mut dim_anns = shared_dim_anns.clone();
                                            dim_anns.extend_from_slice(&dim_usage_anns);
                                            dim_anns
                                        })
                                    });


                                dimensions.push(Dimension {
                                    name: shared_dim_config.name.clone(),
                                    foreign_key: Some(dim_usage.foreign_key.clone()),
                                    hierarchies,
                                    annotations: dim_annotations,
                                });
                            }
                        }
                    }
                }
            }

            cubes.push(Cube {
                name: cube_config.name,
                table: cube_config.table.into(),
                can_aggregate: false,
                dimensions,
                measures,
                annotations: cube_annotations,
            });
        }

        let schema_annotations = schema_config.annotations
            .map(|anns| {
                anns.into_iter()
                    .map(|ann| ann.into())
                    .collect()
            });

        Schema {
            name: schema_config.name,
            cubes,
            annotations: schema_annotations,
        }
    }
}

/// No `From<CubeConfig>` because the transition needs to be made at Schema
/// level in order to take into account shared dims.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cube {
    pub name: String,
    pub table: Table,
    pub can_aggregate: bool,
    pub dimensions: Vec<Dimension>,
    pub measures: Vec<Measure>,
    pub annotations: Option<Vec<Annotation>>,
}

impl Cube {
    /// Returns a Vec<String> of all the dimension name options for a given Cube.
    pub fn get_all_level_names(&self) -> Vec<LevelName> {
        let mut dimension_names: Vec<LevelName> = vec![];

        for dimension in &self.dimensions {
            let dimension_name = dimension.name.clone();
            for hierarchy in &dimension.hierarchies {
                let hierarchy_name = hierarchy.name.clone();
                for level in &hierarchy.levels {
                    let level_name = level.name.clone();
                    dimension_names.push(
                        LevelName {
                            dimension: dimension_name.clone(),
                            hierarchy: hierarchy_name.clone(),
                            level: level_name,
                        }
                    );
                }
            }
        }

        dimension_names
    }

    /// Returns a Vec<String> of all the measure names for a given Cube.
    pub fn get_all_measure_names(&self) -> Vec<MeasureName> {
        let mut measure_names: Vec<MeasureName> = vec![];

        for measure in &self.measures {
            measure_names.push(
                MeasureName::new(measure.name.clone())
            );
        }

        measure_names
    }

    /// Finds the dimension and hierarchy names for a given level.
    pub fn identify_level(&self, level_name: String) -> Result<(String, String), Error> {
        for dimension in self.dimensions.clone() {
            for hierarchy in dimension.hierarchies.clone() {
                for level in hierarchy.levels.clone() {
                    if level.name == level_name {
                        return Ok((dimension.name, hierarchy.name))
                    }
                }
            }
        }

        Err(format_err!("'{}' not found", level_name))
    }

    /// Finds the dimension, hierarchy, and level names for a given property.
    pub fn identify_property(&self, property_name: String) -> Result<(String, String, String), Error> {
        for dimension in self.dimensions.clone() {
            for hierarchy in dimension.hierarchies.clone() {
                for level in hierarchy.levels.clone() {
                    match level.properties {
                        Some(props) => {
                            for property in props {
                                if property.name == property_name {
                                    return Ok((dimension.name, hierarchy.name, level.name))
                                }
                            }
                        },
                        None => continue
                    }
                }
            }
        }

        Err(format_err!("'{}' not found", property_name))
    }
}


#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Dimension {
    pub name: String,
    pub foreign_key: Option<String>,
    pub hierarchies: Vec<Hierarchy>,
    pub annotations: Option<Vec<Annotation>>,
}

impl From<DimensionConfigJson> for Dimension {
    fn from(dimension_config: DimensionConfigJson) -> Self {
        let hierarchies = dimension_config.hierarchies.into_iter()
            .map(|h| h.into())
            .collect();
        let annotations = dimension_config.annotations
            .map(|anns| {
                anns.into_iter()
                    .map(|ann| ann.into())
                    .collect()
            });

        Dimension {
            name: dimension_config.name,
            foreign_key: dimension_config.foreign_key,
            hierarchies: hierarchies,
            annotations: annotations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hierarchy {
    pub name: String,
    pub table: Option<Table>,
    pub primary_key: String,
    pub levels: Vec<Level>,
    pub annotations: Option<Vec<Annotation>>,
}

impl From<HierarchyConfigJson> for Hierarchy {
    fn from(hierarchy_config: HierarchyConfigJson) -> Self {
        let levels: Vec<Level> = hierarchy_config.levels.into_iter()
            .map(|l| l.into())
            .collect();
        let annotations = hierarchy_config.annotations
            .map(|anns| {
                anns.into_iter()
                    .map(|ann| ann.into())
                    .collect()
            });

        let primary_key = hierarchy_config.primary_key
            .unwrap_or_else(|| {
                levels.iter()
                    .last()
                    .expect("TODO check that there's at least 1 level")
                    .key_column
                    .clone()
            });

        Hierarchy {
            name: hierarchy_config.name,
            table: hierarchy_config.table.map(|t| t.into()),
            primary_key,
            levels,
            annotations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Level {
    pub name: String,
    pub key_column: String,
    pub name_column: Option<String>,
    pub properties: Option<Vec<Property>>,
    pub key_type: Option<MemberType>,
    pub annotations: Option<Vec<Annotation>>,
}

impl From<LevelConfigJson> for Level {
    fn from(level_config: LevelConfigJson) -> Self {
        let properties = level_config.properties
            .map(|ps| {
                ps.into_iter()
                    .map(|p| p.into())
                    .collect()
            });
        let annotations = level_config.annotations
            .map(|anns| {
                anns.into_iter()
                    .map(|ann| ann.into())
                    .collect()
            });

        Level {
            name: level_config.name,
            key_column: level_config.key_column,
            name_column: level_config.name_column,
            properties,
            key_type: level_config.key_type,
            annotations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Measure{
    pub name: String,
    pub column: String,
    pub aggregator: Aggregator,
    pub annotations: Option<Vec<Annotation>>,
}

impl From<MeasureConfigJson> for Measure {
    fn from(measure_config: MeasureConfigJson) -> Self {
        let annotations = measure_config.annotations
            .map(|anns| {
                anns.into_iter()
                    .map(|ann| ann.into())
                    .collect()
            });

        Measure {
            name: measure_config.name,
            column: measure_config.column,
            aggregator: measure_config.aggregator,
            annotations: annotations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Table{
    pub name: String,
    pub schema: Option<String>,
    pub primary_key: Option<String>,
}

impl From<TableConfigJson> for Table {
    fn from(table_config: TableConfigJson) -> Self {
        Table {
            name: table_config.name,
            schema: table_config.schema,
            primary_key: table_config.primary_key,
        }
    }
}

impl Table {
    pub fn full_name(&self) -> String {
        if let Some(ref schema) = self.schema {
            format!("{}.{}", schema, self.name)
        } else {
            self.name.to_owned()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Property{
    pub name: String,
    pub column: String,
    pub annotations: Option<Vec<Annotation>>,
}

impl From<PropertyConfigJson> for Property {
    fn from(property_config: PropertyConfigJson) -> Self {
        let annotations = property_config.annotations
            .map(|anns| {
                anns.into_iter()
                    .map(|ann| ann.into())
                    .collect()
            });

        Property {
            name: property_config.name,
            column: property_config.column,
            annotations: annotations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Annotation{
    pub name: String,
    pub text: String,
}

impl From<AnnotationConfigJson> for Annotation {
    fn from(annotation_config: AnnotationConfigJson) -> Self {
        Annotation {
            name: annotation_config.name,
            text: annotation_config.text,
        }
    }
}
