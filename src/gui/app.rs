//! 应用全局状态

use std::path::PathBuf;
use crossbeam_channel::{Receiver, Sender};

use crate::config::{FormulaMode, SplitSheetConfig};

/// 单个待拆分 Sheet 的 UI 配置状态
#[derive(Debug, Clone)]
pub struct SheetConfigState {
    pub name: String,           // Sheet 名称（从文件读取）
    pub enabled: bool,          // 是否参与拆分
    pub split_column: String,   // 用户输入的拆分列
    pub header_row: String,     // 用户输入的表头行号（字符串，便于编辑）
    pub is_common: bool,        // 是否作为公共 Sheet
    pub detected_columns: Vec<String>, // 从文件中检测到的列名
}

impl SheetConfigState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            enabled: false,
            split_column: String::new(),
            header_row: "1".to_string(),
            is_common: false,
            detected_columns: vec![],
        }
    }

    /// 转换为 SplitSheetConfig（仅当 enabled 且配置有效时）
    pub fn to_split_config(&self) -> Option<SplitSheetConfig> {
        if !self.enabled || self.split_column.trim().is_empty() {
            return None;
        }
        let header_row = self.header_row.parse::<usize>().unwrap_or(1).max(1);
        Some(SplitSheetConfig {
            name: self.name.clone(),
            split_column: self.split_column.trim().to_string(),
            header_row,
        })
    }
}

/// 任务状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    Idle,
    Running,
    Success { file_count: usize },
    Failed { error: String },
}

/// 后台任务消息
#[derive(Debug)]
pub enum TaskMessage {
    Log(String),
    Progress { current: usize, total: usize },
    Done { file_count: usize },
    Error(String),
}

/// 主应用状态
pub struct AppState {
    // 输入文件
    pub input_path: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,

    // Sheet 配置
    pub sheet_configs: Vec<SheetConfigState>,
    pub sheets_loaded: bool,

    // 输出选项
    pub force_overwrite: bool,
    pub empty_key_label: String,
    pub create_empty_sheet: bool,
    pub formula_mode: FormulaMode,

    // 任务状态
    pub task_state: TaskState,
    pub progress: (usize, usize), // (current, total)
    pub log_messages: Vec<String>,

    // 通信通道
    pub task_sender: Option<Sender<()>>,    // 发送取消信号
    pub msg_receiver: Option<Receiver<TaskMessage>>,

    // UI 状态
    pub active_tab: Tab,
    pub show_about: bool,
    pub error_popup: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Config,   // 配置页
    Log,      // 日志页
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            input_path: None,
            output_dir: None,
            sheet_configs: vec![],
            sheets_loaded: false,
            force_overwrite: false,
            empty_key_label: "空值".to_string(),
            create_empty_sheet: true,
            formula_mode: FormulaMode::Value,
            task_state: TaskState::Idle,
            progress: (0, 0),
            log_messages: vec![],
            task_sender: None,
            msg_receiver: None,
            active_tab: Tab::Config,
            show_about: false,
            error_popup: None,
        }
    }
}

impl AppState {
    /// 从已加载的 Sheet 中收集有效配置
    pub fn collect_split_configs(&self) -> Vec<SplitSheetConfig> {
        self.sheet_configs
            .iter()
            .filter_map(|s| s.to_split_config())
            .collect()
    }

    pub fn collect_common_sheets(&self) -> Vec<String> {
        self.sheet_configs
            .iter()
            .filter(|s| s.is_common)
            .map(|s| s.name.clone())
            .collect()
    }

    /// 验证当前配置是否可以执行
    pub fn validate(&self) -> Result<(), String> {
        if self.input_path.is_none() {
            return Err("请先选择输入文件".into());
        }
        let splits = self.collect_split_configs();
        if splits.is_empty() {
            return Err("至少需要配置一个待拆分 Sheet（勾选「拆分」并填写拆分列）".into());
        }
        for cfg in &splits {
            if cfg.split_column.is_empty() {
                return Err(format!("Sheet「{}」的拆分列不能为空", cfg.name));
            }
        }
        Ok(())
    }

    pub fn add_log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        // 保留最近 1000 条
        if self.log_messages.len() >= 1000 {
            self.log_messages.remove(0);
        }
        self.log_messages.push(msg);
    }

    pub fn clear_log(&mut self) {
        self.log_messages.clear();
    }
}
