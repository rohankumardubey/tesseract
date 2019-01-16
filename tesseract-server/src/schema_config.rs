use failure::{Error, format_err};

use tesseract_core::Schema;


/// Reads a schema from an XML or JSON file and converts it into a `tesseract_core::Schema` object.
pub fn read_schema(schema_path: &String) -> Result<Schema, Error> {
    let schema_str = std::fs::read_to_string(&schema_path)
        .map_err(|_| format_err!("Schema file not found at {}", schema_path))?;

    let mut schema: Schema;

    if schema_path.ends_with("xml") {
        schema = Schema::from_xml(&schema_str)?;
    } else if schema_path.ends_with("json") {
        schema = Schema::from_json(&schema_str)?;
    } else {
        panic!("Schema format not supported");
    }

    Ok(schema)
}