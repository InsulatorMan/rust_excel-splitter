use std::collections::HashMap;

use calamine::Data;

use crate::config::AppConfig;
use crate::error::{AppError, Result};
use crate::reader::{get_column_index, WorkbookData};

/// 输出分组
#[derive(Debug, Clone)]
pub struct OutputGroup {
    pub key: String,
    pub safe_key: String,
    pub sheets: HashMap<String, Vec<usize>>, // sheet_name -> row_indices
}

/// 规划输出分组
pub fn plan_outputs(config: &AppConfig, wb: &WorkbookData) -> Result<Vec<OutputGroup>> {
    let mut groups: HashMap<String, OutputGroup> = HashMap::new();
    
    for split_config in &config.split_sheets {
        let sheet = wb.sheets.get(&split_config.name)
            .ok_or_else(|| AppError::Config(format!("Sheet '{}' not found", split_config.name)))?;
        
        let col_idx = get_column_index(sheet, &split_config.split_column)
            .ok_or_else(|| AppError::Config(
                format!("Column '{}' not found in sheet '{}'", split_config.split_column, split_config.name)
            ))?;
        
        for (row_idx, row) in sheet.rows.iter().enumerate() {
            let key = if col_idx < row.len() {
                match &row[col_idx] {
                    Data::String(s) if !s.is_empty() => s.clone(),
                    Data::Float(f) => f.to_string(),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    _ => config.empty_key_label.clone(),
                }
            } else {
                config.empty_key_label.clone()
            };
            
            let safe_key = make_safe_filename(&key);
            
            groups.entry(safe_key.clone())
                .or_insert_with(|| OutputGroup {
                    key: key.clone(),
                    safe_key: safe_key.clone(),
                    sheets: HashMap::new(),
                })
                .sheets
                .entry(split_config.name.clone())
                .or_insert_with(Vec::new)
                .push(row_idx);
        }
    }
    
    let mut result: Vec<OutputGroup> = groups.into_values().collect();
    result.sort_by(|a, b| a.key.cmp(&b.key));
    
    Ok(result)
}

/// 创建安全的文件名
fn make_safe_filename(key: &str) -> String {
    let mut result = String::new();
    for ch in key.chars() {
        match ch {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => result.push('_'),
            c => result.push(c),
        }
    }
    if result.is_empty() {
        result = "empty".to_string();
    }
    if result.len() > 200 {
        result = result[..200].to_string();
    }
    result
}
