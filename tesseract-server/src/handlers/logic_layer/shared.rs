use failure::{Error, format_err, bail};
use std::convert::{TryFrom};
use std::collections::HashMap;

use serde_derive::{Serialize, Deserialize};

use tesseract_core::names::{LevelName, Cut, Drilldown, Property, Measure};
use tesseract_core::query::{FilterQuery};
use tesseract_core::Query as TsQuery;
use tesseract_core::schema::{Cube};


#[derive(Debug, Clone)]
pub enum TimeValue {
    First,
    Last,
    Value(u32),
}

impl TimeValue {
    pub fn from_str(raw: String) -> Result<Self, Error> {
        if raw == "latest" {
            Ok(TimeValue::Last)
        } else if raw == "oldest" {
            Ok(TimeValue::First)
        } else {
            match raw.parse::<u32>() {
                Ok(n) => Ok(TimeValue::Value(n)),
                Err(_) => Err(format_err!("Wrong type for time argument."))
            }
        }
    }
}


#[derive(Debug, Clone)]
pub enum TimePrecision {
    Year,
    Quarter,
    Month,
    Week,
    Day,
}

impl TimePrecision {
    pub fn from_str(raw: String) -> Result<Self, Error> {
        match raw.as_ref() {
            "year" => Ok(TimePrecision::Year),
            "quarter" => Ok(TimePrecision::Quarter),
            "month" => Ok(TimePrecision::Month),
            "week" => Ok(TimePrecision::Week),
            "day" => Ok(TimePrecision::Day),
            _ => Err(format_err!("Wrong type for time precision argument."))
        }
    }
}


#[derive(Debug, Clone)]
pub struct Time {
    pub precision: TimePrecision,
    pub value: TimeValue,
}

impl Time {
    pub fn from_str(raw: String) -> Result<Self, Error> {
        let e: Vec<&str> = raw.split(".").collect();

        if e.len() != 2 {
            return Err(format_err!("Wrong format for time argument."));
        }

        let precision = match TimePrecision::from_str(e[0].to_string()) {
            Ok(precision) => precision,
            Err(err) => return Err(err),
        };
        let value = match TimeValue::from_str(e[1].to_string()) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };

        Ok(Time {precision, value})
    }

    pub fn from_key_value(key: String, value: String) -> Result<Self, Error> {
        let precision = match TimePrecision::from_str( key) {
            Ok(precision) => precision,
            Err(err) => return Err(err),
        };
        let value = match TimeValue::from_str(value) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };

        Ok(Time {precision, value})
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicLayerQueryOptTest {
    pub cube: String,
    pub cube_obj: Option<Cube>,

    pub drilldowns: Option<String>,
    #[serde(flatten)]
    pub cuts: Option<HashMap<String, String>>,
    pub time: Option<String>,
    measures: Option<String>,
    properties: Option<String>,
    filters: Option<String>,
    parents: Option<bool>,
    top: Option<String>,
    top_where: Option<String>,
    sort: Option<String>,
    limit: Option<String>,
    growth: Option<String>,
    rca: Option<String>,
    debug: Option<bool>,
//    distinct: Option<bool>,
//    nonempty: Option<bool>,
//    sparse: Option<bool>,
}

impl TryFrom<LogicLayerQueryOptTest> for TsQuery {
    type Error = Error;

    fn try_from(agg_query_opt: LogicLayerQueryOptTest) -> Result<Self, Self::Error> {
        let cube = match agg_query_opt.cube_obj {
            Some(c) => c,
            None => bail!("No cubes found with the given name")
        };

        let mut drilldowns: Vec<Drilldown> = vec![];
        let mut cuts: Vec<Cut> = vec![];
        let mut measures: Vec<Measure> = vec![];
        let mut properties: Vec<Property> = vec![];
        let mut filters: Vec<FilterQuery>= vec![];


        match agg_query_opt.drilldowns {
            Some(d) => {
                let drilldown_levels = deserialize_args(d);
//                println!(" ");
//                println!("{:?}", drilldown_levels);

                for level in drilldown_levels {
                    let (dimension, hierarchy) = match cube.identify_level(level.clone()) {
                        Ok(dh) => dh,
                        Err(_) => break
                    };
//                    println!("[{}].[{}].[{}]", dimension, hierarchy, level);

//                    drilldowns.push(
//
//                    );
                }
            },
            None => ()
        }


        let parents = agg_query_opt.parents.unwrap_or(false);

        let top = agg_query_opt.top
            .map(|t| t.parse())
            .transpose()?;
        let top_where = agg_query_opt.top_where
            .map(|t| t.parse())
            .transpose()?;
        let sort = agg_query_opt.sort
            .map(|s| s.parse())
            .transpose()?;
        let limit = agg_query_opt.limit
            .map(|l| l.parse())
            .transpose()?;
        let growth = agg_query_opt.growth
            .map(|g| g.parse())
            .transpose()?;
        let rca = agg_query_opt.rca
            .map(|r| r.parse())
            .transpose()?;

        let debug = agg_query_opt.debug.unwrap_or(false);

        Ok(TsQuery {
            drilldowns,
            cuts,
            measures,
            parents,
            properties,
            top,
            top_where,
            sort,
            limit,
            rca,
            growth,
            debug,
            filters,
        })
    }
}


pub fn deserialize_args(arg: String) -> Vec<String> {
    let mut open = false;
    let mut curr_str = "".to_string();
    let mut arg_vec: Vec<String> = vec![];

    for c in arg.chars() {
        let c_str = c.to_string();
        if c_str == "[" {
            open = true;
        } else if c_str == "]" {
            arg_vec.push(curr_str);
            curr_str = "".to_string();
            open = false;
        } else if c_str == "," {
            if open {
                curr_str += &c_str;
            } else {
                continue;
            }
        } else {
            curr_str += &c_str;
        }
    }

    if curr_str.len() >= 1 {
        arg_vec.push(curr_str);
    }

    arg_vec
}



#[derive(Debug, Clone)]
pub struct LogicLayerQueryOpt {
    pub cube: String,
    pub drilldowns: Option<Vec<String>>,
    pub cuts: Option<Vec<String>>,
    pub time: Option<HashMap<String, String>>,
    measures: Option<Vec<String>>,
    properties: Option<Vec<String>>,
    filters: Option<Vec<String>>,
    parents: Option<bool>,
    top: Option<String>,
    top_where: Option<String>,
    sort: Option<String>,
    limit: Option<String>,
    growth: Option<String>,
    rca: Option<String>,
    debug: Option<bool>,
//    distinct: Option<bool>,
//    nonempty: Option<bool>,
//    sparse: Option<bool>,
}

impl LogicLayerQueryOpt {
    fn deserialize_args(arg: String) -> Vec<String> {
        let mut open = false;
        let mut curr_str = "".to_string();
        let mut arg_vec: Vec<String> = vec![];

        for c in arg.chars() {
            let c_str = c.to_string();
            if c_str == "[" {
                open = true;
            } else if c_str == "]" {
                arg_vec.push(curr_str);
                curr_str = "".to_string();
                open = false;
            } else if c_str == "," {
                if open {
                    curr_str += &c_str;
                } else {
                    continue;
                }
            } else {
                curr_str += &c_str;
            }
        }

        if curr_str.len() >= 1 {
            arg_vec.push(curr_str);
        }

        arg_vec
    }

    pub fn from_params_map(params_map: HashMap<String, String>, cube_obj: Cube) -> Result<Self, Error> {
        let mut cube: String = "".to_string();
        let mut drilldowns: Option<Vec<String>> = None;
        let mut cuts: Option<Vec<String>> = None;
        let mut measures: Option<Vec<String>> = None;
        let mut time: Option<HashMap<String, String>> = None;
        let mut properties: Option<Vec<String>> = None;
        let mut parents: Option<bool> = None;
        let mut top: Option<String> = None;
        let mut top_where: Option<String> = None;
        let mut sort: Option<String> = None;
        let mut limit: Option<String> = None;
        let mut growth: Option<String> = None;
        let mut rca: Option<String> = None;
        let mut debug: Option<bool> = None;
        let mut filters: Option<Vec<String>> = None;

        let mut time_map: HashMap<String, String> = HashMap::new();
        let mut cuts_vec: Vec<String> = vec![];

        for (k, v) in params_map.iter() {
            let param = k.clone();
            let value = v.clone();

            if param == "cube" {
                cube = value;
            } else if param == "drilldowns" {
                let drilldown_levels = LogicLayerQueryOpt::deserialize_args(value);
                let mut d: Vec<String> = vec![];

                for level in drilldown_levels {
                    let (dimension, hierarchy) = match cube_obj.identify_level(level.clone()) {
                        Ok(dh) => dh,
                        Err(_) => break
                    };

                    d.push(
                        format!("[{}].[{}].[{}]", dimension, hierarchy, level)
                    );
                }

                drilldowns = Some(d);
            } else if param == "measures" {
                measures = Some(LogicLayerQueryOpt::deserialize_args(value));
            } else if param == "time" {
                let time_op: Vec<String> = value.split(".").map(|s| s.to_string()).collect();
                if time_op.len() != 2 {
                    return Err(format_err!("Wrong format for time argument."));
                }
                time_map.insert(time_op[0].clone(), time_op[1].clone());
            } else if param == "properties" {
                let property_levels = LogicLayerQueryOpt::deserialize_args(value);

                let mut p: Vec<String> = vec![];

                for property in property_levels {
                    let (dimension, hierarchy, level) = match cube_obj.identify_property(property.clone()) {
                        Ok(dh) => dh,
                        Err(_) => break
                    };

                    p.push(
                        format!("[{}].[{}].[{}].[{}]", dimension, hierarchy, level, property)
                    );
                }

                properties = Some(p);
            } else if param == "parents" {
                if value == "true" {
                    parents = Some(true);
                } else {
                    parents = Some(false);
                }
            } else if param == "top" {
                top = Some(value);
            } else if param == "top_where" {
                top_where = Some(value);
            } else if param == "sort" {
                sort = Some(value);
            } else if param == "limit" {
                limit = Some(value);
            } else if param == "growth" {
                growth = Some(value);
            } else if param == "rca" {
                rca = Some(value);
            } else if param == "debug" {
                if value == "true" {
                    debug = Some(true);
                } else {
                    debug = Some(false);
                }
            } else {
                // Support for arbitrary cuts
                let (dimension, hierarchy) = match cube_obj.identify_level(param.clone()) {
                    Ok(dh) => dh,
                    Err(_) => break
                };
                cuts_vec.push(format!("[{}].[{}].[{}].[{}]", dimension, hierarchy, param, value));
            }
        }

        // TODO: Add filter support

        if time_map.len() >= 1 {
            time = Some(time_map);
        }

        if cuts_vec.len() >= 1 {
            cuts = Some(cuts_vec);
        }

        Ok(
            LogicLayerQueryOpt {
                cube,
                drilldowns,
                cuts,
                measures,
                time,
                properties,
                parents,
                top,
                top_where,
                sort,
                limit,
                growth,
                rca,
                debug,
                filters
            }
        )
    }
}


impl TryFrom<LogicLayerQueryOpt> for TsQuery {
    type Error = Error;

    fn try_from(agg_query_opt: LogicLayerQueryOpt) -> Result<Self, Self::Error> {
        let drilldowns: Vec<_> = agg_query_opt.drilldowns
            .map(|ds| {
                ds.iter().map(|d| d.parse()).collect()
            })
            .unwrap_or(Ok(vec![]))?;

        let cuts: Vec<_> = agg_query_opt.cuts
            .map(|cs| {
                cs.iter().map(|c| c.parse()).collect()
            })
            .unwrap_or(Ok(vec![]))?;

        let measures: Vec<_> = agg_query_opt.measures
            .map(|ms| {
                ms.iter().map(|m| m.parse()).collect()
            })
            .unwrap_or(Ok(vec![]))?;

        let properties: Vec<_> = agg_query_opt.properties
            .map(|ms| {
                ms.iter().map(|m| m.parse()).collect()
            })
            .unwrap_or(Ok(vec![]))?;

        let filters: Vec<_> = agg_query_opt.filters
            .map(|fs| {
                fs.iter().map(|f| f.parse()).collect()
            })
            .unwrap_or(Ok(vec![]))?;

        let parents = agg_query_opt.parents.unwrap_or(false);

        let top = agg_query_opt.top
            .map(|t| t.parse())
            .transpose()?;
        let top_where = agg_query_opt.top_where
            .map(|t| t.parse())
            .transpose()?;
        let sort = agg_query_opt.sort
            .map(|s| s.parse())
            .transpose()?;
        let limit = agg_query_opt.limit
            .map(|l| l.parse())
            .transpose()?;

        let growth = agg_query_opt.growth
            .map(|g| g.parse())
            .transpose()?;

        let rca = agg_query_opt.rca
            .map(|r| r.parse())
            .transpose()?;

        let debug = agg_query_opt.debug.unwrap_or(false);

        Ok(TsQuery {
            drilldowns,
            cuts,
            measures,
            parents,
            properties,
            top,
            top_where,
            sort,
            limit,
            rca,
            growth,
            debug,
            filters,
        })
    }
}
