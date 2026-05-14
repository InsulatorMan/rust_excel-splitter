//! Sheet 配置面板

use egui::{Color32, ComboBox, Grid, RichText, ScrollArea};

use crate::gui::app::{AppState, SheetConfigState, TaskState};
use crate::gui::task::load_sheet_preview;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    // 加载 Sheet 按钮
    let can_load = state.input_path.is_some()
        && !state.sheets_loaded
        && state.task_state == TaskState::Idle;

    ui.horizontal(|ui| {
        ui.heading(RichText::new("Sheet 配置").size(15.0));
        ui.add_space(12.0);

        if can_load {
            if ui
                .button(RichText::new("加载 Sheet 列表").size(13.0))
                .clicked()
            {
                load_sheets(state);
            }
        } else if state.sheets_loaded {
            ui.label(
                RichText::new(format!("共 {} 个 Sheet", state.sheet_configs.len()))
                    .size(12.0)
                    .color(Color32::from_gray(120)),
            );
            if ui.small_button("重新加载").clicked() {
                state.sheets_loaded = false;
                state.sheet_configs.clear();
            }
        }
    });

    if !state.sheets_loaded {
        ui.add_space(8.0);
        ui.label(
            RichText::new("请先选择输入文件，然后点击「加载 Sheet 列表」")
                .size(12.0)
                .color(Color32::from_gray(140)),
        );
        return;
    }

    ui.add_space(6.0);

    // 表格列头
    render_header(ui);

    // Sheet 列表
    ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            for config in state.sheet_configs.iter_mut() {
                render_sheet_row(ui, config);
            }
        });

    ui.add_space(4.0);
    ui.separator();

    // 快速操作
    ui.horizontal(|ui| {
        if ui.small_button("全选为公共").clicked() {
            for c in state.sheet_configs.iter_mut() {
                c.is_common = true;
                c.enabled = false;
            }
        }
        if ui.small_button("清除所有").clicked() {
            for c in state.sheet_configs.iter_mut() {
                c.enabled = false;
                c.is_common = false;
                c.split_column.clear();
            }
        }
    });
}

fn render_header(ui: &mut egui::Ui) {
    Grid::new("sheet_header")
        .num_columns(5)
        .min_col_width(60.0)
        .striped(false)
        .show(ui, |ui| {
            ui.label(RichText::new("Sheet 名称").strong().size(12.0));
            ui.label(RichText::new("拆分").strong().size(12.0));
            ui.label(RichText::new("拆分列（列名或字母）").strong().size(12.0));
            ui.label(RichText::new("表头行").strong().size(12.0));
            ui.label(RichText::new("公共").strong().size(12.0));
            ui.end_row();
        });
    ui.separator();
}

fn render_sheet_row(ui: &mut egui::Ui, config: &mut SheetConfigState) {
    let row_color = if config.enabled {
        Color32::from_rgba_unmultiplied(40, 120, 255, 15)
    } else if config.is_common {
        Color32::from_rgba_unmultiplied(40, 180, 80, 15)
    } else {
        Color32::TRANSPARENT
    };

    egui::Frame::none()
        .fill(row_color)
        .inner_margin(egui::Margin::symmetric(4.0, 2.0))
        .show(ui, |ui| {
            Grid::new(format!("row_{}", config.name))
                .num_columns(5)
                .min_col_width(60.0)
                .show(ui, |ui| {
                    // Sheet 名称
                    ui.label(
                        RichText::new(&config.name)
                            .size(13.0)
                            .color(if config.enabled {
                                Color32::from_rgb(30, 90, 200)
                            } else {
                                Color32::DARK_GRAY
                            }),
                    );

                    // 拆分开关
                    let split_changed = ui.checkbox(&mut config.enabled, "").changed();
                    if split_changed && config.enabled {
                        config.is_common = false; // 互斥
                    }

                    // 拆分列输入（含下拉建议）
                    ui.add_enabled_ui(config.enabled, |ui| {
                        if config.detected_columns.is_empty() {
                            ui.add(
                                egui::TextEdit::singleline(&mut config.split_column)
                                    .desired_width(160.0)
                                    .hint_text("列名或列字母(如 A)"),
                            );
                        } else {
                            // 带下拉建议的列选择
                            ComboBox::from_id_salt(format!("col_{}", config.name))
                                .selected_text(if config.split_column.is_empty() {
                                    "选择或输入列名"
                                } else {
                                    &config.split_column
                                })
                                .width(160.0)
                                .show_ui(ui, |ui| {
                                    for col_name in &config.detected_columns {
                                        ui.selectable_value(
                                            &mut config.split_column,
                                            col_name.clone(),
                                            col_name,
                                        );
                                    }
                                });
                        }
                    });

                    // 表头行号
                    ui.add_enabled_ui(config.enabled, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut config.header_row)
                                .desired_width(50.0),
                        );
                    });

                    // 公共 Sheet 开关
                    let common_changed = ui.checkbox(&mut config.is_common, "").changed();
                    if common_changed && config.is_common {
                        config.enabled = false; // 互斥
                        config.split_column.clear();
                    }

                    ui.end_row();
                });
        });
}

fn load_sheets(state: &mut AppState) {
    let Some(path) = &state.input_path else { return };

    match load_sheet_preview(path) {
        Ok(sheets) => {
            state.sheet_configs = sheets
                .into_iter()
                .map(|(name, columns)| {
                    let mut cfg = SheetConfigState::new(name);
                    cfg.detected_columns = columns;
                    cfg
                })
                .collect();
            state.sheets_loaded = true;
        }
        Err(e) => {
            state.error_popup = Some(format!("加载 Sheet 失败: {}", e));
        }
    }
}
