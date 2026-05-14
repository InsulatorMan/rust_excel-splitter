//! 后台任务：在独立线程中执行 Excel 处理，通过 channel 回传进度

use std::path::PathBuf;
use crossbeam_channel::Sender;

use crate::config::{AppConfig, FormulaMode, SplitSheetConfig};
use crate::gui::app::TaskMessage;

pub struct TaskParams {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub split_sheets: Vec<SplitSheetConfig>,
    pub common_sheets: Vec<String>,
    pub force: bool,
    pub empty_key_label: String,
    pub create_empty_sheet: bool,
    pub formula_mode: FormulaMode,
}

/// 在后台线程执行处理，通过 sender 回传消息
pub fn run_task(params: TaskParams, sender: Sender<TaskMessage>) {
    std::thread::spawn(move || {
        let send = |msg: TaskMessage| {
            let _ = sender.send(msg);
        };

        send(TaskMessage::Log("开始读取文件...".into()));

        // 读取工作簿
        let wb_data = match crate::reader::load_workbook(
            &params.input,
            &params.split_sheets,
        ) {
            Ok(d) => d,
            Err(e) => {
                send(TaskMessage::Error(format!("读取失败: {}", e)));
                return;
            }
        };

        send(TaskMessage::Log(format!(
            "文件读取完成，共 {} 个 Sheet",
            wb_data.sheet_order.len()
        )));

        // 构建配置
        let config = AppConfig {
            input: params.input,
            output_dir: params.output_dir.clone(),
            force: params.force,
            empty_key_label: params.empty_key_label,
            create_empty_sheet: params.create_empty_sheet,
            formula_mode: params.formula_mode,
            split_sheets: params.split_sheets,
            common_sheets: params.common_sheets,
        };

        // 确保输出目录存在
        if let Err(e) = std::fs::create_dir_all(&config.output_dir) {
            send(TaskMessage::Error(format!("创建输出目录失败: {}", e)));
            return;
        }

        // 规划输出
        send(TaskMessage::Log("分析分组结构...".into()));
        let outputs = match crate::processor::plan_outputs(&config, &wb_data) {
            Ok(o) => o,
            Err(e) => {
                send(TaskMessage::Error(format!("分析失败: {}", e)));
                return;
            }
        };

        if outputs.is_empty() {
            send(TaskMessage::Log("未找到任何分组键，无文件输出。".into()));
            send(TaskMessage::Done { file_count: 0 });
            return;
        }

        let total = outputs.len();
        send(TaskMessage::Log(format!("共 {} 个分组，开始写出...", total)));
        send(TaskMessage::Progress { current: 0, total });

        // 逐个写出（带进度）
        let mut success_count = 0;
        for (i, group) in outputs.iter().enumerate() {
            send(TaskMessage::Log(format!(
                "  [{}/{}] 写出: {}",
                i + 1, total, group.safe_key
            )));

            if let Err(e) = crate::writer::write_all_outputs(&config, &wb_data, &[group.clone()]) {
                send(TaskMessage::Log(format!(
                    "  分组「{}」写出失败: {}",
                    group.key, e
                )));
            } else {
                success_count += 1;
            }

            send(TaskMessage::Progress {
                current: i + 1,
                total,
            });
        }

        send(TaskMessage::Log(format!(
            "完成！成功写出 {}/{} 个文件，输出目录: {}",
            success_count,
            total,
            params.output_dir.display()
        )));
        send(TaskMessage::Done { file_count: success_count });
    });
}

/// 从源文件快速提取 Sheet 列表及其列名（用于 UI 预览）
pub fn load_sheet_preview(
    path: &std::path::Path,
) -> crate::error::Result<Vec<(String, Vec<String>)>> {
    use calamine::{Data, Reader, Xlsx};

    let mut wb: Xlsx<_> = calamine::open_workbook(path)
        .map_err(|e: calamine::XlsxError| crate::error::AppError::Calamine(e.to_string()))?;
    let names = wb.sheet_names().to_vec();

    let mut result = Vec::new();
    for name in &names {
        let range = wb
            .worksheet_range(name)
            .map_err(|e: calamine::XlsxError| crate::error::AppError::Calamine(e.to_string()))?;

        // 取第一行作为列名预览
        let columns: Vec<String> = if range.height() > 0 {
            (0..range.width())
                .map(|c| {
                    range
                        .get_value((0, c as u32))
                        .map(|v| match v {
                            Data::String(s) => s.clone(),
                            Data::Float(f) => f.to_string(),
                            Data::Int(i) => i.to_string(),
                            _ => String::new(),
                        })
                        .unwrap_or_default()
                })
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![]
        };

        result.push((name.clone(), columns));
    }

    Ok(result)
}
