use csv;
use failure::{Error, format_err};
use serde_json::json;
use std::collections::HashMap;

use crate::dataframe::{DataFrame, ColumnData};

#[derive(Debug, Clone)]
pub enum FormatType{
    Csv,
    JsonRecords,
}

impl std::str::FromStr for FormatType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "csv" => Ok(FormatType::Csv),
            "jsonrecords" => Ok(FormatType::JsonRecords),
            _ => Err(format_err!("{} is not a supported format", s)),
        }
    }
}

pub fn format_records(headers: &[String], df: DataFrame, format_type: FormatType) -> Result<String, Error> {
    match format_type {
        FormatType::Csv => Ok(format_csv(headers, df)?),
        FormatType::JsonRecords => Ok(format_jsonrecords(headers, df)?),
    }
}

fn format_csv(headers: &[String], df: DataFrame) -> Result<String, Error> {
    let mut wtr = csv::WriterBuilder::new()
        .from_writer(vec![]);

    // write header
    wtr.write_record(headers)?;

    let mut row_buf = vec![];

    // write data
    for row_idx in 0..df.len() {
        for col_idx in 0..df.columns.len() {
            let val = match df.columns[col_idx].column_data {
                ColumnData::Int8(ref ns) => ns[row_idx].to_string(),
                ColumnData::Int16(ref ns) => ns[row_idx].to_string(),
                ColumnData::Int32(ref ns) => ns[row_idx].to_string(),
                ColumnData::Int64(ref ns) => ns[row_idx].to_string(),
                ColumnData::UInt8(ref ns) => ns[row_idx].to_string(),
                ColumnData::UInt16(ref ns) => ns[row_idx].to_string(),
                ColumnData::UInt32(ref ns) => ns[row_idx].to_string(),
                ColumnData::UInt64(ref ns) => ns[row_idx].to_string(),
                ColumnData::Float32(ref ns) => ns[row_idx].to_string(),
                ColumnData::Float64(ref ns) => ns[row_idx].to_string(),
                ColumnData::Text(ref ss) => ss[row_idx].to_string(),
            };

            row_buf.push(val);
        }
        wtr.write_record(&row_buf)?;

        row_buf.clear();
    }

    let res = String::from_utf8(wtr.into_inner()?)?;

    Ok(res)
}

fn format_jsonrecords(headers: &[String], df: DataFrame) -> Result<String, Error> {
    // each HashMap is a row
    let mut rows = vec![];


    // write data
    for row_idx in 0..df.len() {
        let mut row = HashMap::new();
        for col_idx in 0..df.columns.len() {
            let val = match df.columns[col_idx].column_data {
                ColumnData::Int8(ref ns) => ns[row_idx].to_string(),
                ColumnData::Int16(ref ns) => ns[row_idx].to_string(),
                ColumnData::Int32(ref ns) => ns[row_idx].to_string(),
                ColumnData::Int64(ref ns) => ns[row_idx].to_string(),
                ColumnData::UInt8(ref ns) => ns[row_idx].to_string(),
                ColumnData::UInt16(ref ns) => ns[row_idx].to_string(),
                ColumnData::UInt32(ref ns) => ns[row_idx].to_string(),
                ColumnData::UInt64(ref ns) => ns[row_idx].to_string(),
                ColumnData::Float32(ref ns) => ns[row_idx].to_string(),
                ColumnData::Float64(ref ns) => ns[row_idx].to_string(),
                ColumnData::Text(ref ss) => ss[row_idx].to_string(),
            };

            row.insert(&headers[col_idx], val);
        }

        rows.push(row);
    }

    let res = json!({
        "data": rows,
    });

    Ok(res.to_string())
}