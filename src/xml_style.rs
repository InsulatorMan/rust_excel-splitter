//! XML 样式处理模块（保留原始样式信息）

use std::collections::HashMap;

/// 单元格样式信息
#[derive(Debug, Clone, Default)]
pub struct CellStyle {
    pub font_name: Option<String>,
    pub font_size: Option<f64>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub number_format: Option<String>,
    pub horizontal_align: Option<String>,
    pub vertical_align: Option<String>,
}

/// 样式表
#[derive(Debug, Default)]
pub struct StyleSheet {
    pub styles: HashMap<String, CellStyle>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
        }
    }
    
    pub fn get_style(&self, style_id: &str) -> Option<&CellStyle> {
        self.styles.get(style_id)
    }
}
