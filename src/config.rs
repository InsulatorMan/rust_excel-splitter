use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormulaMode {
    Value,
    Keep,
}

impl Default for FormulaMode {
    fn default() -> Self {
        FormulaMode::Value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitSheetConfig {
    pub name: String,
    pub split_column: String,
    pub header_row: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub force: bool,
    pub empty_key_label: String,
    pub create_empty_sheet: bool,
    pub formula_mode: FormulaMode,
    pub split_sheets: Vec<SplitSheetConfig>,
    pub common_sheets: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input: PathBuf::new(),
            output_dir: PathBuf::from("output"),
            force: false,
            empty_key_label: "空值".to_string(),
            create_empty_sheet: true,
            formula_mode: FormulaMode::Value,
            split_sheets: vec![],
            common_sheets: vec![],
        }
    }
}
