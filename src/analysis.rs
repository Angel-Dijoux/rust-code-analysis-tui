use crate::{
    error::{AppError, AppResult},
    metrics::*,
};
use ratatui::{prelude::*, widgets::*};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn analyze_directory(path: &Path) -> AppResult<Table<'static>> {
    if !path.is_dir() {
        return Err(AppError::AnalysisError(format!(
            "{} is not a directory",
            path.display()
        )));
    }
    let json_files: Vec<String> = WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
        })
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect();

    Ok(create_summary_table(MetricsSummary::summarize(
        json_files
            .par_iter()
            .filter_map(|file_path| read_json_file(file_path))
            .collect(),
    ))
    .to_owned())
}

fn create_summary_table(summary: MetricsSummary) -> Table<'static> {
    let rows = vec![
        create_table_row("NArgs", summary.nargs),
        create_table_row("NExits", summary.nexits),
        create_table_row("Cognitive Complexity", summary.cognitive),
        create_table_row("Cyclomatic Complexity", summary.cyclomatic),
        create_table_row("Halstead Metrics", summary.halstead),
        create_table_row("Lines of Code", summary.loc),
        create_table_row("Number of Methods", summary.nom),
        create_table_row("Maintainability Index", summary.mi),
        create_table_row("ABC Complexity", summary.abc),
    ];

    let widths = [Constraint::Percentage(30), Constraint::Percentage(70)];
    Table::new(rows, widths)
        .header(
            Row::new(vec!["Metric", "Summary"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .title("Metrics Summary")
                .borders(Borders::ALL),
        )
}

fn create_table_row<T: std::fmt::Display>(
    label: impl Into<String>,
    value: Option<T>,
) -> Row<'static> {
    Row::new(vec![
        Cell::from(label.into()),
        Cell::from(value.map_or_else(|| "N/A".to_owned(), |v| v.to_string())),
    ])
}

pub trait Merge: Sized + Clone + std::fmt::Debug + 'static {
    type Metric;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self>;
}

#[derive(Debug, Serialize, Default)]
pub struct MetricsSummary {
    nargs: Option<MetricValuesSummary>,
    nexits: Option<BasicSummary>,
    cognitive: Option<BasicSummary>,
    cyclomatic: Option<BasicSummary>,
    halstead: Option<HalsteadSummary>,
    loc: Option<LocSummary>,
    nom: Option<NomSummary>,
    mi: Option<MiSummary>,
    abc: Option<AbcSummary>,
}

impl MetricsSummary {
    pub fn summarize(json_data: Vec<JsonData>) -> MetricsSummary {
        json_data
            .iter()
            .flat_map(|data| data.metrics.as_ref())
            .fold(MetricsSummary::default(), |mut summary, metrics| {
                summary.nargs = MetricValuesSummary::merge(summary.nargs, &metrics.nargs);
                summary.nexits = BasicSummary::merge(summary.nexits, &metrics.nexits);
                summary.cognitive = BasicSummary::merge(summary.cognitive, &metrics.cognitive);
                summary.cyclomatic = BasicSummary::merge(summary.cyclomatic, &metrics.cyclomatic);
                summary.halstead = HalsteadSummary::merge(summary.halstead, &metrics.halstead);
                summary.loc = LocSummary::merge(summary.loc, &metrics.loc);
                summary.nom = NomSummary::merge(summary.nom, &metrics.nom);
                summary.mi = MiSummary::merge(summary.mi, &metrics.mi);
                summary.abc = AbcSummary::merge(summary.abc, &metrics.abc);
                summary
            })
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct MetricValuesSummary {
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
    pub count: usize,
}

use std::fmt;

impl fmt::Display for MetricValuesSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Total Functions: {}\nTotal Closures: {}\nAvg Functions: {:.2}\nAvg Closures: {:.2}\nTotal: {}\nAverage: {:.2}\nMin Functions: {}\nMax Functions: {}\nMin Closures: {}\nMax Closures: {}\nCount: {}",
            self.total_functions.unwrap_or(0.0),
            self.total_closures.unwrap_or(0.0),
            self.average_functions.unwrap_or(0.0),
            self.average_closures.unwrap_or(0.0),
            self.total.unwrap_or(0.0),
            self.average.unwrap_or(0.0),
            self.functions_min.unwrap_or(0.0),
            self.functions_max.unwrap_or(0.0),
            self.closures_min.unwrap_or(0.0),
            self.closures_max.unwrap_or(0.0),
            self.count
        )
    }
}

impl Merge for MetricValuesSummary {
    type Metric = MetricValues;

    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        metric
            .as_ref()
            .map(|m| {
                let summary = current.clone().unwrap_or_default();

                Some(MetricValuesSummary {
                    total_functions: Some(
                        summary.total_functions.unwrap_or(0.0) + m.total_functions.unwrap_or(0.0),
                    ),
                    total_closures: Some(
                        summary.total_closures.unwrap_or(0.0) + m.total_closures.unwrap_or(0.0),
                    ),
                    total: Some(summary.total.unwrap_or(0.0) + m.total.unwrap_or(0.0)),

                    average_functions: Some(
                        ((summary.average_functions.unwrap_or(0.0) * summary.count as f64)
                            + (m.average_functions.unwrap_or(0.0) * 1.0))
                            / (summary.count + 1) as f64,
                    ),
                    average_closures: Some(
                        ((summary.average_closures.unwrap_or(0.0) * summary.count as f64)
                            + (m.average_closures.unwrap_or(0.0) * 1.0))
                            / (summary.count + 1) as f64,
                    ),
                    average: Some(
                        ((summary.average.unwrap_or(0.0) * summary.count as f64)
                            + (m.average.unwrap_or(0.0) * 1.0))
                            / (summary.count + 1) as f64,
                    ),

                    functions_min: Some(
                        summary
                            .functions_min
                            .unwrap_or(f64::MAX)
                            .min(m.functions_min.unwrap_or(f64::MAX)),
                    ),
                    functions_max: Some(
                        summary
                            .functions_max
                            .unwrap_or(f64::MIN)
                            .max(m.functions_max.unwrap_or(f64::MIN)),
                    ),
                    closures_min: Some(
                        summary
                            .closures_min
                            .unwrap_or(f64::MAX)
                            .min(m.closures_min.unwrap_or(f64::MAX)),
                    ),
                    closures_max: Some(
                        summary
                            .closures_max
                            .unwrap_or(f64::MIN)
                            .max(m.closures_max.unwrap_or(f64::MIN)),
                    ),

                    count: summary.count + 1,
                })
            })
            .unwrap_or(current)
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct BasicSummary {
    sum: f64,
    average: f64,
    min: f64,
    max: f64,
    count: usize,
}

impl fmt::Display for BasicSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Sum: {:.2}\nAverage: {:.2}\nMin: {:.2}\nMax: {:.2}\nCount: {}",
            self.sum, self.average, self.min, self.max, self.count
        )
    }
}

impl Merge for BasicSummary {
    type Metric = BasicMetric;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        metric
            .as_ref()
            .map(|m| {
                let mut summary = current.clone().unwrap_or_default();
                summary.sum += m.sum.unwrap_or(0.0);
                summary.average += m.average.unwrap_or(0.0);
                summary.min = summary.min.min(m.min.unwrap_or(f64::MAX));
                summary.max = summary.max.max(m.max.unwrap_or(f64::MIN));
                summary.count += 1;
                Some(summary)
            })
            .unwrap_or(current)
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct HalsteadSummary {
    n1: f64,
    n2: f64,
    volume: f64,
    count: usize,
}

impl fmt::Display for HalsteadSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "n1: {:.2}\nn2: {:.2}\nVolume: {:.2}\nCount: {}",
            self.n1, self.n2, self.volume, self.count
        )
    }
}

impl Merge for HalsteadSummary {
    type Metric = Halstead;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        metric
            .as_ref()
            .map(|m| {
                let mut summary = current.clone().unwrap_or_default();
                summary.n1 += m.n1.unwrap_or(0.0);
                summary.n2 += m.n2.unwrap_or(0.0);
                summary.volume += m.volume.unwrap_or(0.0);
                summary.count += 1;
                Some(summary)
            })
            .unwrap_or(current)
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct LocSummary {
    sloc: f64,
    ploc: f64,
    count: usize,
}

impl fmt::Display for LocSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SLOC: {:.2}\nPLOC: {:.2}\nCount: {}",
            self.sloc, self.ploc, self.count
        )
    }
}

impl Merge for LocSummary {
    type Metric = Loc;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        metric
            .as_ref()
            .map(|m| {
                let mut summary = current.clone().unwrap_or_default();
                summary.sloc += m.sloc.unwrap_or(0.0);
                summary.ploc += m.ploc.unwrap_or(0.0);
                summary.count += 1;
                Some(summary)
            })
            .unwrap_or(current)
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct NomSummary {
    functions: f64,
    closures: f64,
    total: f64,
    count: usize,
}

impl fmt::Display for NomSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Functions: {:.2}\nClosures: {:.2}\nTotal: {:.2}\nCount: {}",
            self.functions, self.closures, self.total, self.count
        )
    }
}

impl Merge for NomSummary {
    type Metric = Nom;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        metric
            .as_ref()
            .map(|m| {
                let mut summary = current.clone().unwrap_or_default();
                summary.functions += m.functions.unwrap_or(0.0);
                summary.closures += m.closures.unwrap_or(0.0);
                summary.total += m.total.unwrap_or(0.0);
                summary.count += 1;
                Some(summary)
            })
            .unwrap_or(current)
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct MiSummary {
    mi_original: f64,
    mi_sei: f64,
    mi_visual_studio: f64,
    count: usize,
}

impl fmt::Display for MiSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MI Original: {:.2}\nMI SEI: {:.2}\nMI VS: {:.2}\nCount: {}",
            self.mi_original, self.mi_sei, self.mi_visual_studio, self.count
        )
    }
}

impl Merge for MiSummary {
    type Metric = Mi;

    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        metric
            .as_ref()
            .map(|m| {
                let mut summary = current.clone().unwrap_or_default();
                summary.mi_original += m.mi_original.unwrap_or(0.0);
                summary.mi_sei += m.mi_sei.unwrap_or(0.0);
                summary.mi_visual_studio += m.mi_visual_studio.unwrap_or(0.0);
                summary.count += 1;
                Some(summary)
            })
            .unwrap_or(current)
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct AbcSummary {
    assignments: f64,
    branches: f64,
    conditions: f64,
    count: usize,
}

impl fmt::Display for AbcSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Assignments: {:.2}\nBranches: {:.2}\nConditions: {:.2}\nCount: {}",
            self.assignments, self.branches, self.conditions, self.count
        )
    }
}

impl Merge for AbcSummary {
    type Metric = Abc;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        metric
            .as_ref()
            .map(|m| {
                let mut summary = current.clone().unwrap_or_default();
                summary.assignments += m.assignments.unwrap_or(0.0);
                summary.branches += m.branches.unwrap_or(0.0);
                summary.conditions += m.conditions.unwrap_or(0.0);
                summary.count += 1;
                Some(summary)
            })
            .unwrap_or(current)
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct WmcSummary {
    classes: f64,
    interfaces: f64,
    total: f64,
    count: usize,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct NpmSummary {
    classes: f64,
    interfaces: f64,
    class_methods: f64,
    total: f64,
    count: usize,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct NpaSummary {
    classes: f64,
    interfaces: f64,
    total: f64,
    count: usize,
}

fn read_json_file(file_path: &str) -> Option<JsonData> {
    match fs::read_to_string(file_path) {
        Ok(content) => match serde_json::from_str::<JsonData>(&content) {
            Ok(data) => Some(data),
            Err(err) => {
                eprintln!("Failed to parse JSON {}: {}", file_path, err);
                None
            }
        },
        Err(err) => {
            eprintln!("Failed to read file {}: {}", file_path, err);
            None
        }
    }
}
