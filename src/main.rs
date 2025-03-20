mod app;
use iced::widget::{
    center, horizontal_space, text_input,
};
use iced::window;
use iced::{Element, Subscription, Task, Theme, Vector};

use std::collections::BTreeMap;
use crate::app::message::{Message, WindowCategory};

fn main() -> iced::Result {
    iced::daemon(TBox::title, TBox::update, TBox::view)
        .subscription(TBox::subscription)
        .theme(TBox::theme)
        .scale_factor(TBox::scale_factor)
        .font(include_bytes!("../assets/fonts/sanjiheisongti.ttf").as_slice())
        .run_with(TBox::new)
}

struct TBox {
    windows: BTreeMap<window::Id, app::window::Window>,
}




impl TBox {
    fn new() -> (Self, Task<Message>) {
        let (_id, open) = window::open(window::Settings::default());

        (
            Self {
                windows: BTreeMap::new(),
            },
            open.map(|id| Message::WindowOpened(id, WindowCategory::Homepage)),
        )
    }

    fn title(&self, window: window::Id) -> String {
        self.windows
            .get(&window)
            .map(|window| window.title.clone())
            .unwrap_or_default()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenWindow(category) => {
                let Some(last_window) = self.windows.keys().last() else {
                    return Task::none();
                };

                window::get_position(*last_window)
                    .then(|last_position| {
                        let position = last_position.map_or(
                            window::Position::Default,
                            |last_position| {
                                window::Position::Specific(
                                    last_position + Vector::new(20.0, 20.0),
                                )
                            },
                        );

                        let (_id, open) = window::open(window::Settings {
                            position,
                            ..window::Settings::default()
                        });
                        println!("Window opened: {:?}", _id);
                        open
                    }).map(move |id| Message::WindowOpened(id, category.clone()))
            }
            Message::WindowOpened(id, category) => {
                let mut window = app::window::Window::new(self.windows.len() + 1);
                window.category = category;
                match window.category {
                    WindowCategory::Homepage => {
                        window.title = "主页".to_string();
                    },
                    WindowCategory::ToolsTime => {
                        window.title = "时间工具".to_string();
                    },
                    WindowCategory::ToolsJson2Csv => {
                        window.title = "Json转Csv".to_string();
                    },
                    WindowCategory::ToolsJson2Query => {
                        window.title = "Json转Query".to_string();
                    }
                }
                let focus_input = text_input::focus(format!("input-{id}"));

                self.windows.insert(id, window);

                focus_input
            }
            Message::WindowClosed(id) => {
                self.windows.remove(&id);

                if self.windows.is_empty() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Message::ScaleInputChanged(id, scale) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.scale_input = scale;
                }

                Task::none()
            }
            Message::ScaleChanged(id, scale) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.current_scale = scale
                        .parse::<f64>()
                        .unwrap_or(window.current_scale)
                        .clamp(0.5, 5.0);
                }

                Task::none()
            }
            Message::TitleChanged(id, title) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.title = title;
                }

                Task::none()
            }
            Message::ContentChanged(id, message) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.update(message);
                }

                Task::none()
            }

        }
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {
        if let Some(window) = self.windows.get(&window_id) {
            window.view(window_id)
        } else {
            horizontal_space().into()
        }
    }

    fn theme(&self, window: window::Id) -> Theme {
        if let Some(window) = self.windows.get(&window) {
            window.theme.clone()
        } else {
            Theme::default()
        }
    }

    fn scale_factor(&self, window: window::Id) -> f64 {
        self.windows
            .get(&window)
            .map(|window| window.current_scale)
            .unwrap_or(1.0)
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}

