use std::collections::HashMap;
use eframe::egui;
use egui::Color32;
use crate::tbox::Page::{HomePage, PushBoxGame};

// 看我丑陋的代码。。。。


#[derive(Eq, PartialEq, Hash)]
enum Page {
    HomePage,
    PushBoxGame
}

pub struct App {
    state: GlobalState,
    text: String,
    info: String,
    current_map: [[i32; 10]; 10],
}

pub struct GlobalState {
    current_page: Page,
    page_state_map: HashMap<Page, PageState>,
}

pub struct PageState {
    id: Page,
    title: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        setup_custom_fonts(&cc.egui_ctx);
        let game_map: [[i32; 10]; 10]  = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 2, 2, 2, 2, 1, 1, 1],
            [1, 1, 2, 2, 4, 2, 2, 2, 1, 1],
            [1, 1, 2, 3, 2, 2, 2, 2, 1, 1], 
            [1, 1, 2, 2, 2, 1, 1, 1, 1, 1],
            [1, 1, 2, 2, 2, 1, 1, 1, 1, 1],
            [1, 1, 2, 2, 2, 5, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        let mut state_map = HashMap::new();
        state_map.insert(HomePage, PageState{id:HomePage, title:String::from("首页")});
        state_map.insert(PushBoxGame, PageState{id:PushBoxGame, title:String::from("推箱子")});

        Self {
            state: GlobalState{current_page:HomePage, page_state_map: state_map},
            text: String::from("操作："),
            info: String::from("结果："),
            current_map: game_map,
        }
    }

    fn get_current_page_title(&mut self) -> String {
        match self.state.page_state_map.get(&(self.state.current_page)) {
            Some(x) => String::from(&(x.title)),
            None => String::from(""),
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        custom_window_frame(ctx, frame, self.get_current_page_title().as_str() , |ui| {
            match self.state.current_page {
                HomePage => {
                    if ui.button("推箱子").clicked() {
                        self.state.current_page = PushBoxGame;
                        ui.heading("text");
                    }
                },
                PushBoxGame => {
                    if ui.button("主界面").clicked() {
                        self.state.current_page = HomePage;
                    }

                    ui.label(&self.text);
                    ui.label(&self.info);
                    

                    // 绘制地图
                    let rows = self.current_map.len();
                    let cols = self.current_map[0].len();

                    let frows: f32 = rows as f32;
                    let fcols: f32 = cols as f32;
                    
                   // 计算总的容器大小
                    let grid_rect = ui.available_rect_before_wrap();
                    let total_size = grid_rect.width().min(grid_rect.height());
                    // 计算每个格子的大小
                    let cell_size = total_size / f32::from(frows).min(fcols);
                    
                    
                    let dark_mode = ui.visuals().dark_mode;
                    let faded_color = ui.visuals().window_fill();
                    let faded_color = |color: Color32| -> Color32 {
                        use egui::Rgba;
                        let t = if dark_mode { 0.7 } else { 0.55 };
                        egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
                    };

                    
                    // 绘制格子
                    for row in 0..rows {
                        ui.horizontal(|ui| {
                            for col in 0..cols {
                                let x0 = grid_rect.left() + (col+1) as f32 * cell_size ;
                                let x1 = x0 + cell_size;
                                let y0 = grid_rect.top() + row as f32 * cell_size;
                                let y1 = y0 + cell_size;

                                // 绘制一个格子
                                // let mut rect = ui.available_rect_before_wrap();
                                // rect.set_height(cell_size);
                                // rect.set_width(cell_size);
                                let mut rect_color = faded_color(Color32::GOLD);
                                if self.current_map[row][col] == 2 {
                                    rect_color = faded_color(Color32::WHITE);
                                } else if self.current_map[row][col] == 3 {
                                    rect_color = faded_color(Color32::RED);
                                } else if self.current_map[row][col] == 4 {
                                    rect_color = faded_color(Color32::GREEN);
                                } else if self.current_map[row][col] == 5 {
                                    rect_color = faded_color(Color32::BLUE);
                                }
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1)),
                                    0.0,
                                    rect_color,
                                )
                            }
                        });
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::W)) {
                        self.text.push_str("上");
                        let mut change = false;
                        for row in 0..rows {
                            for col in 0..cols {
                                if self.current_map[row][col] == 3 {
                                    if self.current_map[row-1][col] != 1 {
                                        if self.current_map[row-1][col] == 4 {
                                            if row - 1 >= 1 && self.current_map[row-2][col] != 1{
                                                self.current_map[row-2][col] = 4;
                                                self.current_map[row-1][col] = 3;
                                                self.current_map[row][col] = 2;
                                            }
                                        } else {
                                            self.current_map[row-1][col] = 3;
                                            self.current_map[row][col] = 2;
                                        }
                                    }
                                    
                                    change = true;
                                    break;
                                }
                            }
                            if change {
                                break;
                            }
                        }
                    }

                    if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                        self.text.push_str("下");
                        let mut change = false;
                        for row in 0..rows {
                            for col in 0..cols {
                                if self.current_map[row][col] == 3 {
                                    if (row + 1) <= 10 && self.current_map[row+1][col] != 1{
                                        if self.current_map[row+1][col] == 4 {
                                            if (row + 2) <= 10 && self.current_map[row+2][col] != 1 {
                                                self.current_map[row+2][col] = 4;
                                                self.current_map[row+1][col] = 3;
                                                self.current_map[row][col] = 2;
                                            }
                                        } else {
                                            self.current_map[row+1][col] = 3;
                                            self.current_map[row][col] = 2;
                                        }
                                        
                                    }
                                    
                                    change = true;
                                    break;
                                }
                            }
                            if change {
                                break;
                            }
                        }
                    }

                    if ctx.input(|i| i.key_released(egui::Key::A)) {
                        self.text.push_str("左");
                        let mut change = false;
                        for row in 0..rows {
                            for col in 0..cols {
                                if self.current_map[row][col] == 3 {
                                    if self.current_map[row][col-1] != 1 {
                                        if self.current_map[row][col-1] == 4 {
                                            if col - 1 >= 1 && self.current_map[row][col-2] != 1 {
                                                self.current_map[row][col-2] = 4;
                                                self.current_map[row][col-1] = 3;
                                                self.current_map[row][col] = 2;
                                            }
                                        } else {
                                            self.current_map[row][col-1] = 3;
                                            self.current_map[row][col] = 2;
                                        }
                                        
                                    }
                                    
                                    change = true;
                                    break;
                                }
                            }
                            if change {
                                break;
                            }
                        }
                    }

                    if ctx.input(|i| i.key_released(egui::Key::D)) {
                        self.text.push_str("右");
                        let mut change = false;
                        for row in 0..rows {
                            for col in 0..cols {
                                if self.current_map[row][col] == 3 {
                                    if (col+1) <= 10 && self.current_map[row][col+1] != 1 {
                                        if  self.current_map[row][col+1] == 4 {
                                            if col + 2 <= 10 && self.current_map[row][col+2] != 1 {
                                                self.current_map[row][col+2] = 4;
                                                self.current_map[row][col+1] = 3;
                                                self.current_map[row][col] = 2;
                                            }
                                        } else {
                                            self.current_map[row][col+1] = 3;
                                            self.current_map[row][col] = 2;
                                        }
                                       
                                    }
                                    
                                    change = true;
                                    break;
                                }
                            }
                            if change {
                                break;
                            }
                        }
                    }

                    let mut has_target = false;

                    for row in 0..rows {
                        for col in 0..cols {
                            if self.current_map[row][col] == 5 {
                                has_target = true;
                                break;
                            }
                        }
                        if has_target {
                            break;
                        }
                    }
                    if !has_target {
                        self.info = String::from("结果：过关！");
                    }
                }
            }
        });
    }
}

fn custom_window_frame(
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    use egui::*;

    let panel_frame = egui::Frame {
        fill: ctx.style().visuals.window_fill(),
        rounding: 10.0.into(),
        stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
        outer_margin: 0.5.into(),
        ..Default::default()
    };

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let title_bar_height = 32.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };
        title_bar_ui(ui, frame, title_bar_rect, title);

        // Add the contents:
        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        }
        .shrink(4.0);
        let mut content_ui = ui.child_ui(content_rect, *ui.layout());
        add_contents(&mut content_ui);
    });
}

fn title_bar_ui(
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
    title_bar_rect: eframe::epaint::Rect,
    title: &str,
) {
    use egui::*;

    let painter = ui.painter();

    let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());

    

    // Paint the title:
    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );

    // Paint the line under the title:
    painter.line_segment(
        [
            title_bar_rect.left_bottom() + vec2(1.0, 0.0),
            title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
        ],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    // Interact with the title bar (drag to move window):
    if title_bar_response.double_clicked() {
        frame.set_maximized(!frame.info().window_info.maximized);
    } else if title_bar_response.is_pointer_button_down_on() {
        frame.drag_window();
    }

    ui.allocate_ui_at_rect(title_bar_rect, |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            egui::widgets::global_dark_light_mode_switch(ui);
        });
    });

    ui.allocate_ui_at_rect(title_bar_rect, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui, frame);
        });
    });
}

/// 展示最大化，最小化，关闭按钮
fn close_maximize_minimize(ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    use egui::{Button, RichText};

    let button_height = 12.0;

    let close_response = ui
        .add(Button::new(RichText::new("❌").size(button_height)))
        .on_hover_text("关闭");
    if close_response.clicked() {
        frame.close();
    }

    if frame.info().window_info.maximized {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("恢复");
        if maximized_response.clicked() {
            frame.set_maximized(false);
        }
    } else {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("最大化");
        if maximized_response.clicked() {
            frame.set_maximized(true);
        }
    }

    let minimized_response = ui
        .add(Button::new(RichText::new("🗕").size(button_height)))
        .on_hover_text("最小化");
    if minimized_response.clicked() {
        frame.set_minimized(true);
    }
}


fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "fonts/sanjiheisongti.ttf"
        )),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}