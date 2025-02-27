use crate::{
    error::{AppError, AppResult},
    metrics::*,
};
use ratatui::{prelude::*, widgets::*};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use walkdir::WalkDir;

macro_rules! add_details {
    ($rows:ident, $title:expr, $option:expr) => {{
        use ratatui::style::{Color, Modifier, Style};

        let header_style = Style::default()
            .fg(Color::LightBlue)
            .add_modifier(Modifier::BOLD);
        let key_style = Style::default().fg(Color::Yellow);
        let value_style = Style::default().fg(Color::Green);

        if let Some(ref metric) = $option {
            $rows.push(Row::new(vec![
                Cell::from($title).style(header_style),
                Cell::from(""),
            ]));

            for (key, value) in metric.details() {
                $rows.push(Row::new(vec![
                    Cell::from(key).style(key_style),
                    Cell::from(value).style(value_style),
                ]));
            }
        } else {
            $rows.push(Row::new(vec![
                Cell::from($title).style(header_style),
                Cell::from("N/A").style(value_style),
            ]));
        }
    }};
}

pub fn analyze_directory(path: &Path) -> AppResult<Table<'static>> {
    if !path.is_dir() {
        return Err(AppError::AnalysisError(format!(
            "{} is not a directory",
            path.display()
        )));
    }
    let json_files: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && e.path().extension() == Some(std::ffi::OsStr::new("json"))
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    let data: Vec<_> = json_files
        .par_iter()
        .filter_map(|p| read_json_file(p))
        .collect();
    Ok(create_summary_table(MetricsSummary::summarize(data)).to_owned())
}

pub fn create_summary_table(summary: MetricsSummary) -> Table<'static> {
    let mut rows = Vec::new();

    add_details!(rows, "NArgs", summary.nargs);
    add_details!(rows, "NExits", summary.nexits);
    add_details!(rows, "Cognitive Complexity", summary.cognitive);
    add_details!(rows, "Cyclomatic Complexity", summary.cyclomatic);
    add_details!(rows, "Halstead Metrics", summary.halstead);
    add_details!(rows, "Lines of Code", summary.loc);
    add_details!(rows, "Number of Methods", summary.nom);
    add_details!(rows, "Maintainability Index", summary.mi);
    add_details!(rows, "ABC Complexity", summary.abc);

    Table::new(
        rows,
        [Constraint::Percentage(30), Constraint::Percentage(70)],
    )
    .header(
        Row::new(vec!["Metric", "Summary"]).style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .column_spacing(3)
    .block(
        Block::default()
            .title("Metrics Summary")
            .borders(Borders::ALL),
    )
    .style(Style::default().fg(Color::White))
}

fn read_json_file(file_path: &str) -> Option<JsonData> {
    fs::read_to_string(file_path)
        .ok()
        .and_then(|content| serde_json::from_str::<JsonData>(&content).ok())
        .or_else(|| {
            eprintln!("Failed to read or parse {}", file_path);
            None
        })
}

fn update_average(old: Option<f64>, count: usize, new: Option<f64>) -> Option<f64> {
    Some(((old.unwrap_or(0.0) * count as f64) + new.unwrap_or(0.0)) / (count as f64 + 1.0))
}

trait Countable {
    fn add_count(&mut self);
}

impl Countable for MetricValuesSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}
impl Countable for BasicSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}
impl Countable for HalsteadSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}
impl Countable for LocSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}
impl Countable for NomSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}
impl Countable for MiSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}
impl Countable for AbcSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}

impl Countable for WmcSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}

impl Countable for NpmSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}

impl Countable for NpaSummary {
    fn add_count(&mut self) {
        self.count += 1;
    }
}

fn merge_with<T, M, F>(current: Option<T>, metric: &Option<M>, updater: F) -> Option<T>
where
    T: Default + Clone + Copy + Countable,
    F: Fn(&mut T, &M),
{
    metric
        .as_ref()
        .map(|m| {
            let mut summary = current.unwrap_or_default();
            updater(&mut summary, m);
            summary.add_count();
            summary
        })
        .or(current)
}

pub trait Merge: Sized + Clone + std::fmt::Debug + 'static {
    type Metric;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self>;
}

trait Detailed {
    fn details(&self) -> Vec<(String, String)>;
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
    pub fn summarize(json_data: Vec<JsonData>) -> Self {
        json_data.iter().flat_map(|d| d.metrics.as_ref()).fold(
            Default::default(),
            |mut summary, metrics| {
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
            },
        )
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
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

impl Merge for MetricValuesSummary {
    type Metric = MetricValues;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        merge_with(current, metric, |s, m| {
            s.total_functions =
                Some(s.total_functions.unwrap_or(0.0) + m.total_functions.unwrap_or(0.0));
            s.total_closures =
                Some(s.total_closures.unwrap_or(0.0) + m.total_closures.unwrap_or(0.0));
            s.total = Some(s.total.unwrap_or(0.0) + m.total.unwrap_or(0.0));
            s.average_functions = update_average(s.average_functions, s.count, m.average_functions);
            s.average_closures = update_average(s.average_closures, s.count, m.average_closures);
            s.average = update_average(s.average, s.count, m.average);
            s.functions_min = Some(
                s.functions_min
                    .unwrap_or(f64::MAX)
                    .min(m.functions_min.unwrap_or(f64::MAX)),
            );
            s.functions_max = Some(
                s.functions_max
                    .unwrap_or(f64::MIN)
                    .max(m.functions_max.unwrap_or(f64::MIN)),
            );
            s.closures_min = Some(
                s.closures_min
                    .unwrap_or(f64::MAX)
                    .min(m.closures_min.unwrap_or(f64::MAX)),
            );
            s.closures_max = Some(
                s.closures_max
                    .unwrap_or(f64::MIN)
                    .max(m.closures_max.unwrap_or(f64::MIN)),
            );
        })
    }
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct BasicSummary {
    sum: f64,
    average: f64,
    min: f64,
    max: f64,
    count: usize,
}

impl std::fmt::Display for BasicSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        merge_with(current, metric, |s, m| {
            s.sum += m.sum.unwrap_or(0.0);
            s.average += m.average.unwrap_or(0.0);
            s.min = s.min.min(m.min.unwrap_or(f64::MAX));
            s.max = s.max.max(m.max.unwrap_or(f64::MIN));
        })
    }
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct HalsteadSummary {
    n1: f64,
    n2: f64,
    volume: f64,
    purity_ratio: f64,
    bugs: f64,
    difficulty: f64,
    estimated_program_lenght: f64,
    vocabulary: f64,
    level: f64,
    count: usize,
}

impl Merge for HalsteadSummary {
    type Metric = Halstead;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        merge_with(current, metric, |s, m| {
            s.n1 += m.n1.unwrap_or(0.0);
            s.n2 += m.n2.unwrap_or(0.0);
            s.volume += m.volume.unwrap_or(0.0);
            s.bugs += m.bugs.unwrap_or(0.0);
            s.difficulty += m.difficulty.unwrap_or(0.0);
            s.estimated_program_lenght += m.estimated_program_length.unwrap_or(0.0);
            s.vocabulary += m.vocabulary.unwrap_or(0.0);
            s.level += m.level.unwrap_or(0.0);
            s.purity_ratio += m.purity_ratio.unwrap_or(0.0);
        })
    }
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct LocSummary {
    sloc: f64,
    ploc: f64,
    count: usize,
    lloc: f64,
    cloc: f64,
    blank: f64,
    sloc_average: f64,
    ploc_average: f64,
    lloc_average: f64,
    cloc_average: f64,
    blank_average: f64,
    sloc_min: f64,
    sloc_max: f64,
    cloc_min: f64,
    cloc_max: f64,
    ploc_min: f64,
    ploc_max: f64,
    lloc_min: f64,
    lloc_max: f64,
    blank_min: f64,
    blank_max: f64,
}

impl Merge for LocSummary {
    type Metric = Loc;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        merge_with(current, metric, |s, m| {
            s.sloc += m.sloc.unwrap_or(0.0);
            s.ploc += m.ploc.unwrap_or(0.0);
            s.sloc_average += m.sloc_average.unwrap_or(0.0);
            s.ploc_average += m.ploc_average.unwrap_or(0.0);
            s.lloc_average += m.lloc_average.unwrap_or(0.0);
            s.cloc_average += m.cloc_average.unwrap_or(0.0);
            s.blank_average += m.blank_average.unwrap_or(0.0);
            s.sloc_min = if s.sloc_min == 0.0 {
                m.sloc_min.unwrap_or(0.0)
            } else {
                s.sloc_min.min(m.sloc_min.unwrap_or(0.0))
            };
            s.ploc_min = if s.ploc_min == 0.0 {
                m.ploc_min.unwrap_or(0.0)
            } else {
                s.ploc_min.min(m.ploc_min.unwrap_or(0.0))
            };
            s.lloc_min = if s.lloc_min == 0.0 {
                m.lloc_min.unwrap_or(0.0)
            } else {
                s.lloc_min.min(m.lloc_min.unwrap_or(0.0))
            };
            s.cloc_min = if s.cloc_min == 0.0 {
                m.cloc_min.unwrap_or(0.0)
            } else {
                s.cloc_min.min(m.cloc_min.unwrap_or(0.0))
            };
            s.blank_min = if s.blank_min == 0.0 {
                m.blank_min.unwrap_or(0.0)
            } else {
                s.blank_min.min(m.blank_min.unwrap_or(0.0))
            };
            s.sloc_max = s.sloc_max.max(m.sloc_max.unwrap_or(0.0));
            s.ploc_max = s.ploc_max.max(m.ploc_max.unwrap_or(0.0));
            s.lloc_max = s.lloc_max.max(m.lloc_max.unwrap_or(0.0));
            s.cloc_max = s.cloc_max.max(m.cloc_max.unwrap_or(0.0));
            s.blank_max = s.blank_max.max(m.blank_max.unwrap_or(0.0));
        })
    }
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct NomSummary {
    functions: f64,
    closures: f64,
    total: f64,
    count: usize,
}

impl Merge for NomSummary {
    type Metric = Nom;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        merge_with(current, metric, |s, m| {
            s.functions += m.functions.unwrap_or(0.0);
            s.closures += m.closures.unwrap_or(0.0);
            s.total += m.total.unwrap_or(0.0);
        })
    }
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct MiSummary {
    mi_original: f64,
    mi_sei: f64,
    mi_visual_studio: f64,
    count: usize,
}

impl Merge for MiSummary {
    type Metric = Mi;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        merge_with(current, metric, |s, m| {
            s.mi_original += m.mi_original.unwrap_or(0.0);
            s.mi_sei += m.mi_sei.unwrap_or(0.0);
            s.mi_visual_studio += m.mi_visual_studio.unwrap_or(0.0);
        })
    }
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct AbcSummary {
    assignments: f64,
    branches: f64,
    conditions: f64,
    count: usize,
}

impl Merge for AbcSummary {
    type Metric = Abc;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        merge_with(current, metric, |s, m| {
            s.assignments += m.assignments.unwrap_or(0.0);
            s.branches += m.branches.unwrap_or(0.0);
            s.conditions += m.conditions.unwrap_or(0.0);
        })
    }
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct WmcSummary {
    pub classes: f64,
    pub interfaces: f64,
    pub total: f64,
    pub count: usize,
}

impl Merge for WmcSummary {
    type Metric = Wmc;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        merge_with(current, metric, |s, m| {
            s.classes += m.classes.unwrap_or(0.0);
            s.interfaces += m.interfaces.unwrap_or(0.0);
            s.total += m.total.unwrap_or(0.0);
        })
    }
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct NpmSummary {
    pub classes: f64,
    pub interfaces: f64,
    pub class_methods: f64,
    pub total: f64,
    pub count: usize,
}

impl Merge for NpmSummary {
    type Metric = Npm;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        merge_with(current, metric, |s, m| {
            s.classes += m.classes.unwrap_or(0.0);
            s.interfaces += m.interfaces.unwrap_or(0.0);
            s.class_methods += m.class_methods.unwrap_or(0.0);
            s.total += m.total.unwrap_or(0.0);
        })
    }
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct NpaSummary {
    pub classes: f64,
    pub interfaces: f64,
    pub total: f64,
    pub count: usize,
}

impl Merge for NpaSummary {
    type Metric = Npa;
    fn merge(current: Option<Self>, metric: &Option<Self::Metric>) -> Option<Self> {
        merge_with(current, metric, |s, m| {
            s.classes += m.classes.unwrap_or(0.0);
            s.interfaces += m.interfaces.unwrap_or(0.0);
            s.total += m.total.unwrap_or(0.0);
        })
    }
}

impl Detailed for MetricValuesSummary {
    fn details(&self) -> Vec<(String, String)> {
        vec![
            (
                "Total Functions".into(),
                self.total_functions.map_or("N/A".into(), |v| v.to_string()),
            ),
            (
                "Total Closures".into(),
                self.total_closures.map_or("N/A".into(), |v| v.to_string()),
            ),
            (
                "Avg Functions".into(),
                self.average_functions
                    .map_or("N/A".into(), |v| format!("{:.2}", v)),
            ),
            (
                "Avg Closures".into(),
                self.average_closures
                    .map_or("N/A".into(), |v| format!("{:.2}", v)),
            ),
            (
                "Total".into(),
                self.total.map_or("N/A".into(), |v| v.to_string()),
            ),
            (
                "Average".into(),
                self.average.map_or("N/A".into(), |v| format!("{:.2}", v)),
            ),
            (
                "Min Functions".into(),
                self.functions_min.map_or("N/A".into(), |v| v.to_string()),
            ),
            (
                "Max Functions".into(),
                self.functions_max.map_or("N/A".into(), |v| v.to_string()),
            ),
            (
                "Min Closures".into(),
                self.closures_min.map_or("N/A".into(), |v| v.to_string()),
            ),
            (
                "Max Closures".into(),
                self.closures_max.map_or("N/A".into(), |v| v.to_string()),
            ),
            ("Count".into(), self.count.to_string()),
        ]
    }
}

impl Detailed for BasicSummary {
    fn details(&self) -> Vec<(String, String)> {
        vec![
            ("Sum".into(), format!("{:.2}", self.sum)),
            ("Average".into(), format!("{:.2}", self.average)),
            ("Min".into(), format!("{:.2}", self.min)),
            ("Max".into(), format!("{:.2}", self.max)),
            ("Count".into(), self.count.to_string()),
        ]
    }
}

impl Detailed for HalsteadSummary {
    fn details(&self) -> Vec<(String, String)> {
        vec![
            ("n1".into(), format!("{:.2}", self.n1)),
            ("n2".into(), format!("{:.2}", self.n2)),
            ("purity_ratio".into(), format!("{:.2}", self.purity_ratio)),
            ("Bugs".into(), format!("{:.2}", self.bugs)),
            (
                "Estimated Program Lenght".into(),
                format!("{:.2}", self.estimated_program_lenght),
            ),
            ("Vocabulary".into(), format!("{:.2}", self.vocabulary)),
            ("Difficulty".into(), format!("{:.2}", self.difficulty)),
            ("Level".into(), format!("{:.2}", self.level)),
            ("Volume".into(), format!("{:.2}", self.volume)),
            ("Count".into(), self.count.to_string()),
        ]
    }
}

impl Detailed for LocSummary {
    fn details(&self) -> Vec<(String, String)> {
        vec![
            ("SLOC".into(), format!("{:.2}", self.sloc)),
            ("PLOC".into(), format!("{:.2}", self.ploc)),
            ("LLOC".into(), format!("{:.2}", self.lloc)),
            ("CLOC".into(), format!("{:.2}", self.cloc)),
            ("Blank".into(), format!("{:.2}", self.blank)),
            ("SLOC Avg".into(), format!("{:.2}", self.sloc_average)),
            ("PLOC Avg".into(), format!("{:.2}", self.ploc_average)),
            ("LLOC Avg".into(), format!("{:.2}", self.lloc_average)),
            ("CLOC Avg".into(), format!("{:.2}", self.cloc_average)),
            ("Blank Avg".into(), format!("{:.2}", self.blank_average)),
            ("SLOC Min".into(), format!("{:.2}", self.sloc_min)),
            ("SLOC Max".into(), format!("{:.2}", self.sloc_max)),
            ("CLOC Min".into(), format!("{:.2}", self.cloc_min)),
            ("CLOC Max".into(), format!("{:.2}", self.cloc_max)),
            ("PLOC Min".into(), format!("{:.2}", self.ploc_min)),
            ("PLOC Max".into(), format!("{:.2}", self.ploc_max)),
            ("LLOC Min".into(), format!("{:.2}", self.lloc_min)),
            ("LLOC Max".into(), format!("{:.2}", self.lloc_max)),
            ("Blank Min".into(), format!("{:.2}", self.blank_min)),
            ("Blank Max".into(), format!("{:.2}", self.blank_max)),
        ]
    }
}

impl Detailed for NomSummary {
    fn details(&self) -> Vec<(String, String)> {
        vec![
            ("Functions".into(), format!("{:.2}", self.functions)),
            ("Closures".into(), format!("{:.2}", self.closures)),
            ("Total".into(), format!("{:.2}", self.total)),
            ("Count".into(), self.count.to_string()),
        ]
    }
}

impl Detailed for MiSummary {
    fn details(&self) -> Vec<(String, String)> {
        vec![
            ("MI Original".into(), format!("{:.2}", self.mi_original)),
            ("MI SEI".into(), format!("{:.2}", self.mi_sei)),
            ("MI VS".into(), format!("{:.2}", self.mi_visual_studio)),
            ("Count".into(), self.count.to_string()),
        ]
    }
}

impl Detailed for AbcSummary {
    fn details(&self) -> Vec<(String, String)> {
        vec![
            ("Assignments".into(), format!("{:.2}", self.assignments)),
            ("Branches".into(), format!("{:.2}", self.branches)),
            ("Conditions".into(), format!("{:.2}", self.conditions)),
            ("Count".into(), self.count.to_string()),
        ]
    }
}
