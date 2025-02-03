use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metrics {
    pub nargs: Option<MetricValues>,
    pub nexits: Option<BasicMetric>,
    pub cognitive: Option<BasicMetric>,
    pub cyclomatic: Option<BasicMetric>,
    pub halstead: Option<Halstead>,
    pub loc: Option<Loc>,
    pub nom: Option<Nom>,
    pub mi: Option<Mi>,
    pub abc: Option<Abc>,
    pub wmc: Option<Wmc>,
    pub npm: Option<Npm>,
    pub npa: Option<Npa>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricValues {
    pub total_functions: Option<f64>,
    pub total_closures: Option<f64>,
    pub average_functions: Option<f64>,
    pub average_closures: Option<f64>,
    pub total: Option<f64>,
    pub average: Option<f64>,
    pub functions_min: Option<f64>,
    pub functions_max: Option<f64>,
    pub closures_min: Option<f64>,
    pub closures_max: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicMetric {
    pub sum: Option<f64>,
    pub average: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Halstead {
    pub n1: Option<f64>,
    #[serde(rename = "N1")]
    pub n1_upper: Option<f64>,
    pub n2: Option<f64>,
    #[serde(rename = "N2")]
    pub n2_upper: Option<f64>,
    pub length: Option<f64>,
    pub estimated_program_length: Option<f64>,
    pub purity_ratio: Option<f64>,
    pub vocabulary: Option<f64>,
    pub volume: Option<f64>,
    pub difficulty: Option<f64>,
    pub level: Option<f64>,
    pub effort: Option<f64>,
    pub time: Option<f64>,
    pub bugs: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Loc {
    pub sloc: Option<f64>,
    pub ploc: Option<f64>,
    pub lloc: Option<f64>,
    pub cloc: Option<f64>,
    pub blank: Option<f64>,
    pub sloc_average: Option<f64>,
    pub ploc_average: Option<f64>,
    pub lloc_average: Option<f64>,
    pub cloc_average: Option<f64>,
    pub blank_average: Option<f64>,
    pub sloc_min: Option<f64>,
    pub sloc_max: Option<f64>,
    pub cloc_min: Option<f64>,
    pub cloc_max: Option<f64>,
    pub ploc_min: Option<f64>,
    pub ploc_max: Option<f64>,
    pub lloc_min: Option<f64>,
    pub lloc_max: Option<f64>,
    pub blank_min: Option<f64>,
    pub blank_max: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nom {
    pub functions: Option<f64>,
    pub closures: Option<f64>,
    pub functions_average: Option<f64>,
    pub closures_average: Option<f64>,
    pub total: Option<f64>,
    pub average: Option<f64>,
    pub functions_min: Option<f64>,
    pub functions_max: Option<f64>,
    pub closures_min: Option<f64>,
    pub closures_max: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mi {
    pub mi_original: Option<f64>,
    pub mi_sei: Option<f64>,
    pub mi_visual_studio: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Abc {
    pub assignments: Option<f64>,
    pub branches: Option<f64>,
    pub conditions: Option<f64>,
    pub magnitude: Option<f64>,
    pub assignments_average: Option<f64>,
    pub branches_average: Option<f64>,
    pub conditions_average: Option<f64>,
    pub assignments_min: Option<f64>,
    pub assignments_max: Option<f64>,
    pub branches_min: Option<f64>,
    pub branches_max: Option<f64>,
    pub conditions_min: Option<f64>,
    pub conditions_max: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wmc {
    pub classes: Option<f64>,
    pub interfaces: Option<f64>,
    pub total: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Npm {
    pub classes: Option<f64>,
    pub interfaces: Option<f64>,
    pub class_methods: Option<f64>,
    pub interface_methods: Option<f64>,
    pub classes_average: Option<Option<f64>>,
    pub interfaces_average: Option<Option<f64>>,
    pub total: Option<f64>,
    pub total_methods: Option<f64>,
    pub average: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Npa {
    pub classes: Option<f64>,
    pub interfaces: Option<f64>,
    pub class_attributes: Option<f64>,
    pub interface_attributes: Option<f64>,
    pub classes_average: Option<Option<f64>>,
    pub interfaces_average: Option<Option<f64>>,
    pub total: Option<f64>,
    pub total_attributes: Option<f64>,
    pub average: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Space {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub kind: String,
    pub spaces: Vec<Space>,
    pub metrics: Option<Metrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonData {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub kind: String,
    pub spaces: Vec<Space>,
    pub metrics: Option<Metrics>,
}
