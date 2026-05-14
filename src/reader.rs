use std::collections::HashMap;
use std::path::Path;

use calamine::{open_workbook, Data, Reader, Xlsx};

use crate::config::SplitSheetConfig;
use crate::error::{AppError, Result};

/// 工作簿数据，包含所有 sheet 的数据
#[derive(Debug)]
pub struct WorkbookData {
    pub sheet_order: Vec<String>,
    pub sheets: HashMap<String, SheetData>,
}

/// 单个 sheet 的数据
#[derive(Debug)]
pub struct SheetData {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Data>>,
    pub column_index: HashMap<String, usize>,
}

/// 加载工作簿数据
pub fn load_workbook(
    path: &Path,
    split_configs: &[SplitSheetConfig],
) -> Result<WorkbookData> {
    let mut wb: Xlsx<_> = open_workbook(path)
        .map_err(|e: calamine::XlsxError| AppError::Calamine(e.to_string()))?;
    
    let sheet_names = wb.sheet_names().to_vec();
    let mut sheets = HashMap::new();
    
    for name in &sheet_names {
        let range = wb
            .worksheet_range(name)
            .map_err(|e: calamine::XlsxError| AppError::Calamine(e.to_string()))?;
        
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        let mut column_index = HashMap::new();
        
        // 获取该 sheet 的配置
        let config = split_configs.iter().find(|c| c.name == *name);
        let header_row = config.map(|c| c.header_row).unwrap_or(1);
        let header_row_idx = header_row.saturating_sub(1);
        
        // 读取表头
        if range.height() > header_row_idx {
            let width = range.width();
            for col in 0..width {
                let cell = range.get_value((header_row_idx as u32, col as u32));
                let header = match cell {
                    Some(Data::String(s)) => s.clone(),
                    Some(Data::Float(f)) => f.to_string(),
                    Some(Data::Int(i)) => i.to_string(),
                    _ => format!("Column{}", col + 1),
                };
                column_index.insert(header.clone(), col);
                headers.push(header);
            }
            
            // 读取数据行
            for row_idx in (header_row_idx + 1)..range.height() {
                let mut row = Vec::new();
                for col in 0..width {
                    let cell = range.get_value((row_idx as u32, col as u32));
                    row.push(cell.cloned().unwrap_or(Data::Empty));
                }
                rows.push(row);
            }
        }
        
        sheets.insert(
            name.clone(),
            SheetData {
                name: name.clone(),
                headers,
                rows,
                column_index,
            },
        );
    }
    
    Ok(WorkbookData {
        sheet_order: sheet_names,
        sheets,
    })
}

/// 从列名或列字母获取列索引
pub fn get_column_index(sheet: &SheetData, column: &str) -> Option<usize> {
    // 先尝试直接匹配列名
    if let Some(&idx) = sheet.column_index.get(column) {
        return Some(idx);
    }
    
    // 尝试解析列字母（如 A, B, AA）
    let col_letter = column.to_uppercase();
    let mut idx = 0usize;
    for ch in col_letter.chars() {
        if ch >= 'A' && ch <= 'Z' {
            idx = idx * 26 + (ch as usize - 'A' as usize + 1);
        } else {
            return None;
        }
    }
    if idx > 0 {
        Some(idx - 1)
    } else {
        None
    }
}
