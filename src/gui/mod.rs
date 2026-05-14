//! GUI 主体：eframe App 实现

pub mod app;
pub mod panels;
pub mod task;

use eframe::egui;
use egui::{Button, Color32, RichText, TopBottomPanel, CentralPanel};

use crate::gui::app::{AppState, TaskState};
use crate::gui::task::{run_task, TaskParams};

pub struct ExcelSplitterApp {
    state: AppState,
}

impl ExcelSplitterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 设置中文字体
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for ExcelSplitterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理后台消息
        self.poll_task_messages(ctx);

        // 顶部工具栏
        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            render_toolbar(ui, &mut self.state);
        });

        // 底部操作栏
        TopBottomPanel::bottom("action_bar").show(ctx, |ui| {
            render_action_bar(ui, &mut self.state);
        });

        // 主内容区
        CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                panels::input_panel::render(ui, &mut self.state);
                ui.add_space(6.0);
                panels::sheet_panel::render(ui, &mut self.state);
                ui.add_space(6.0);
                panels::output_panel::render(ui, &mut self.state);
                ui.add_space(6.0);
                panels::log_panel::render(ui, &mut self.state);
            });
        });

        // 错误弹窗
        if let Some(err) = self.state.error_popup.clone() {
            egui::Window::new("错误")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new(&err).color(Color32::from_rgb(180, 50, 50)));
                    ui.add_space(8.0);
                    if ui.button("确定").clicked() {
                        self.state.error_popup = None;
                    }
                });
        }
    }
}

impl ExcelSplitterApp {
    fn poll_task_messages(&mut self, ctx: &egui::Context) {
        let Some(receiver) = &self.state.msg_receiver else { return };

        // 批量收取，避免阻塞
        for _ in 0..50 {
            match receiver.try_recv() {
                Ok(crate::gui::app::TaskMessage::Log(msg)) => {
                    self.state.add_log(msg);
                    ctx.request_repaint();
                }
                Ok(crate::gui::app::TaskMessage::Progress { current, total }) => {
                    self.state.progress = (current, total);
                    ctx.request_repaint();
                }
                Ok(crate::gui::app::TaskMessage::Done { file_count }) => {
                    self.state.task_state = TaskState::Success { file_count };
                    self.state.msg_receiver = None;
                    ctx.request_repaint();
                }
                Ok(crate::gui::app::TaskMessage::Error(e)) => {
                    self.state.task_state = TaskState::Failed { error: e.clone() };
                    self.state.add_log(format!("错误: {}", e));
                    self.state.msg_receiver = None;
                    ctx.request_repaint();
                }
                Err(_) => break,
            }
        }
    }
}

fn render_toolbar(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Excel 拆分工具")
                .size(16.0)
                .strong(),
        );
        ui.separator();
        ui.label(
            RichText::new("v0.1.0")
                .size(11.0)
                .color(Color32::from_gray(150)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("关于").clicked() {
                state.show_about = true;
            }
        });
    });
}

fn render_action_bar(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        // 执行按钮
        let is_running = state.task_state == TaskState::Running;
        let btn_text = if is_running { "处理中..." } else { "开始拆分" };

        let btn = Button::new(RichText::new(btn_text).size(14.0))
            .min_size(egui::vec2(120.0, 32.0))
            .fill(if is_running {
                Color32::from_gray(180)
            } else {
                Color32::from_rgb(40, 120, 220)
            });

        if ui.add_enabled(!is_running, btn).clicked() {
            start_task(state);
        }

        // 验证提示
        if let Err(msg) = state.validate() {
            ui.label(
                RichText::new(format!("{}", msg))
                    .size(12.0)
                    .color(Color32::from_rgb(180, 100, 30)),
            );
        }

        // 输出目录快速打开
        if let Some(dir) = &state.output_dir.clone() {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("打开输出目录").clicked() {
                    let _ = open::that(dir);
                }
            });
        }
    });
    ui.add_space(4.0);
}

fn start_task(state: &mut AppState) {
    if let Err(e) = state.validate() {
        state.error_popup = Some(e);
        return;
    }

    let (tx, rx) = crossbeam_channel::unbounded();
    state.msg_receiver = Some(rx);
    state.task_state = TaskState::Running;
    state.progress = (0, 0);
    state.add_log("开始执行...".into());

    let params = TaskParams {
        input: state.input_path.clone().unwrap(),
        output_dir: state.output_dir.clone().unwrap_or_else(|| {
            state.input_path.as_ref().unwrap()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        }),
        split_sheets: state.collect_split_configs(),
        common_sheets: state.collect_common_sheets(),
        force: state.force_overwrite,
        empty_key_label: state.empty_key_label.clone(),
        create_empty_sheet: state.create_empty_sheet,
        formula_mode: state.formula_mode.clone(),
    };

    run_task(params, tx);
}

/// 设置支持中文的字体
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 尝试加载系统字体（Windows 适配）
    let font_paths = [
        "C:/Windows/Fonts/msyh.ttc",                       // Windows 微软雅黑
        "C:/Windows/Fonts/simsun.ttc",                     // Windows 宋体
        "C:/Windows/Fonts/msyhbd.ttc",                     // Windows 微软雅黑粗体
    ];

    for path in &font_paths {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "chinese".to_owned(),
                egui::FontData::from_owned(data).into(),
            );
            // 插入到所有字体族的首位
            for family in fonts.families.values_mut() {
                family.insert(0, "chinese".to_owned());
            }
            break;
        }
    }

    ctx.set_fonts(fonts);
}
