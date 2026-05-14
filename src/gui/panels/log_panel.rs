//! 日志与进度面板

use egui::{Color32, ProgressBar, RichText, ScrollArea};
use crate::gui::app::{AppState, TaskState};

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    // 进度条（任务运行时显示）
    if state.task_state == TaskState::Running {
        let (cur, total) = state.progress;
        let progress = if total > 0 {
            cur as f32 / total as f32
        } else {
            0.0
        };

        ui.add(
            ProgressBar::new(progress)
                .show_percentage()
                .desired_width(f32::INFINITY)
                .text(format!("{}/{} 个文件", cur, total)),
        );
        ui.add_space(4.0);
    }

    // 状态提示
    match &state.task_state {
        TaskState::Success { file_count } => {
            ui.label(
                RichText::new(format!("成功输出 {} 个文件", file_count))
                    .size(14.0)
                    .color(Color32::from_rgb(30, 140, 60)),
            );
        }
        TaskState::Failed { error } => {
            ui.label(
                RichText::new(format!("处理失败: {}", error))
                    .size(13.0)
                    .color(Color32::from_rgb(200, 50, 50)),
            );
        }
        _ => {}
    }

    // 日志区域
    ui.horizontal(|ui| {
        ui.label(RichText::new("执行日志").size(13.0).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("清除").clicked() {
                state.clear_log();
            }
        });
    });

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .max_height(200.0)
        .show(ui, |ui| {
            for msg in &state.log_messages {
                let color = if msg.contains("失败") || msg.contains("错误") || msg.contains("警告") {
                    Color32::from_rgb(180, 80, 40)
                } else if msg.contains("完成") || msg.contains("成功") {
                    Color32::from_rgb(30, 140, 60)
                } else {
                    Color32::from_gray(50)
                };

                ui.label(RichText::new(msg).size(12.0).color(color).monospace());
            }
        });
}
