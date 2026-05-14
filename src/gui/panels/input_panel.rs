//! 文件输入面板

use egui::{Color32, Sense, Stroke, Vec2};
use rfd::FileDialog;

use crate::gui::app::AppState;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(4.0);

    // 拖拽区域
    let drop_zone_height = 80.0;
    let available_width = ui.available_width();

    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(available_width, drop_zone_height),
        Sense::click_and_drag(),
    );

    // 处理文件拖放
    if !ui.ctx().input(|i| i.raw.dropped_files.is_empty()) {
        let files = ui.ctx().input(|i| i.raw.dropped_files.clone());
        if let Some(file) = files.first() {
            if let Some(path) = &file.path {
                if path.extension().map(|e| e == "xlsx").unwrap_or(false) {
                    state.input_path = Some(path.clone());
                    state.sheets_loaded = false;
                    state.sheet_configs.clear();
                }
            }
        }
    }

    // 绘制拖拽区
    let is_hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    let stroke_color = if is_hovering {
        Color32::from_rgb(64, 128, 255)
    } else {
        Color32::from_gray(160)
    };
    let bg_color = if is_hovering {
        Color32::from_rgba_unmultiplied(64, 128, 255, 20)
    } else {
        Color32::from_gray(245)
    };

    ui.painter().rect_filled(rect, 8.0, bg_color);
    ui.painter().rect_stroke(
        rect,
        8.0,
        Stroke::new(1.5, stroke_color),
    );

    let center = rect.center();
    if state.input_path.is_some() {
        let name = state
            .input_path
            .as_ref()
            .unwrap()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知文件");
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("{}", name),
            egui::FontId::proportional(14.0),
            Color32::from_rgb(30, 120, 60),
        );
    } else {
        ui.painter().text(
            egui::pos2(center.x, center.y - 10.0),
            egui::Align2::CENTER_CENTER,
            "将 .xlsx 文件拖放到此处",
            egui::FontId::proportional(14.0),
            Color32::from_gray(120),
        );
        ui.painter().text(
            egui::pos2(center.x, center.y + 12.0),
            egui::Align2::CENTER_CENTER,
            "或点击下方按钮选择文件",
            egui::FontId::proportional(11.0),
            Color32::from_gray(160),
        );
    }

    if response.clicked() {
        open_file_dialog(state);
    }

    ui.add_space(8.0);

    // 按钮行
    ui.horizontal(|ui| {
        if ui.button("选择 Excel 文件").clicked() {
            open_file_dialog(state);
        }

        ui.separator();

        // 输出目录
        let out_label = state
            .output_dir
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("（与输入文件同目录）");

        ui.label(egui::RichText::new("输出目录:").size(13.0));
        ui.add(
            egui::Label::new(
                egui::RichText::new(out_label)
                    .size(12.0)
                    .color(Color32::from_gray(100)),
            )
            .truncate(),
        );

        if ui.button("更改").clicked() {
            open_dir_dialog(state);
        }

        if state.output_dir.is_some() && ui.button("清除").clicked() {
            state.output_dir = None;
        }
    });

    ui.add_space(4.0);
    ui.separator();
}

fn open_file_dialog(state: &mut AppState) {
    if let Some(path) = FileDialog::new()
        .add_filter("Excel 文件", &["xlsx"])
        .set_title("选择输入 Excel 文件")
        .pick_file()
    {
        // 默认输出目录与输入文件同级
        if state.output_dir.is_none() {
            state.output_dir = path.parent().map(|p| p.to_path_buf());
        }
        state.input_path = Some(path);
        state.sheets_loaded = false;
        state.sheet_configs.clear();
    }
}

fn open_dir_dialog(state: &mut AppState) {
    if let Some(path) = FileDialog::new()
        .set_title("选择输出目录")
        .pick_folder()
    {
        state.output_dir = Some(path);
    }
}
