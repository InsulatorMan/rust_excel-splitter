//! 输出选项面板

use egui::{Grid, RichText};
use crate::config::FormulaMode;
use crate::gui::app::AppState;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading(RichText::new("输出选项").size(15.0));
    ui.add_space(6.0);

    Grid::new("output_options")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .show(ui, |ui| {
            // 强制覆盖
            ui.label("强制覆盖已存在文件:");
            ui.checkbox(&mut state.force_overwrite, "");
            ui.end_row();

            // 空键标签
            ui.label("空分组键标签:");
            ui.add(
                egui::TextEdit::singleline(&mut state.empty_key_label)
                    .desired_width(120.0),
            );
            ui.end_row();

            // 空 Sheet
            ui.label("无数据时创建空 Sheet:");
            ui.checkbox(&mut state.create_empty_sheet, "");
            ui.end_row();

            // 公式处理
            ui.label("公式处理方式:");
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut state.formula_mode,
                    FormulaMode::Value,
                    "替换为计算值",
                );
                ui.selectable_value(
                    &mut state.formula_mode,
                    FormulaMode::Keep,
                    "保留公式（可能失效）",
                );
            });
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.separator();
}
