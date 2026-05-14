use std::collections::HashMap;

use calamine::DataType;
use rust_xlsxwriter::Workbook;

use crate::config::{AppConfig, FormulaMode};
use crate::error::{AppError, Result};
use crate::processor::OutputGroup;
use crate::reader::WorkbookData;

/// 写出所有输出文件
pub fn write_all_outputs(
    config: &AppConfig,
    wb: &WorkbookData,
    groups: &[OutputGroup],
) -> Result<()> {
    for group in groups {
        write_group(config, wb, group)?;
    }
    Ok(())
}

/// 写出一个分组文件
fn write_group(
    config: &AppConfig,
    wb: &WorkbookData,
    group: &OutputGroup,
) -> Result<()> {
    let output_path = config.output_dir.join(format!("{}.xlsx", group.safe_key));
    
    if output_path.exists() && !config.force {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("File already exists: {}", output_path.display()),
        )));
    }
    
    let mut workbook = Workbook::new();
    
    // 写入拆分 sheet
    for (sheet_name, row_indices) in &group.sheets {
        let sheet_data = wb.sheets.get(sheet_name)
            .ok_or_else(|| AppError::Config(format!("Sheet '{}' not found", sheet_name)))?;
        
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(sheet_name)
            .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
        
        // 写入表头
        for (col_idx, header) in sheet_data.headers.iter().enumerate() {
            worksheet.write_string(0, col_idx as u16, header)
                .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
        }
        
        // 写入数据行
        for (new_row_idx, &row_idx) in row_indices.iter().enumerate() {
            if row_idx < sheet_data.rows.len() {
                let row = &sheet_data.rows[row_idx];
                for (col_idx, cell) in row.iter().enumerate() {
                    write_cell(worksheet, (new_row_idx + 1) as u32, col_idx as u16, cell, &config.formula_mode)?;
                }
            }
        }
    }
    
    // 写入公共 sheet
    for common_name in &config.common_sheets {
        if let Some(sheet_data) = wb.sheets.get(common_name) {
            let worksheet = workbook.add_worksheet();
            worksheet.set_name(common_name)
                .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
            
            // 写入表头
            for (col_idx, header) in sheet_data.headers.iter().enumerate() {
                worksheet.write_string(0, col_idx as u16, header)
                    .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
            }
            
            // 写入所有数据行
            for (row_idx, row) in sheet_data.rows.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    write_cell(worksheet, (row_idx + 1) as u32, col_idx as u16, cell, &config.formula_mode)?;
                }
            }
        }
    }
    
    // 空 sheet 处理
    if config.create_empty_sheet && group.sheets.is_empty() {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Sheet1")
            .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
    }
    
    workbook.save(&output_path)
        .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
    
    Ok(())
}

/// 写入单元格
fn write_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    cell: &DataType,
    _formula_mode: &FormulaMode,
) -> Result<()> {
    match cell {
        DataType::String(s) => {
            worksheet.write_string(row, col, s)
                .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
        }
        DataType::Float(f) => {
            worksheet.write_number(row, col, *f)
                .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
        }
        DataType::Int(i) => {
            worksheet.write_number(row, col, *i as f64)
                .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
        }
        DataType::Bool(b) => {
            worksheet.write_boolean(row, col, *b)
                .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
        }
        DataType::DateTime(d) => {
            worksheet.write_number(row, col, *d)
                .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
        }
        DataType::Duration(d) => {
            worksheet.write_number(row, col, *d)
                .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
        }
        DataType::Error(e) => {
            worksheet.write_string(row, col, &format!("#ERROR: {:?}", e))
                .map_err(|e| AppError::XlsxWriter(e.to_string()))?;
        }
        DataType::Empty => {
            // 空单元格，不写入
        }
    }
    Ok(())
}
