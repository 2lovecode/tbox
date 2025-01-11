use iced::alignment::Horizontal::Left;
use iced::widget::{button, column, container, row, scrollable, text, text_editor, text_input};
use iced::window;
use iced::{Center, Element, Fill, Theme};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Window {
    pub category: WindowCategory,
    pub title: String,
    pub scale_input: String,
    pub current_scale: f64,
    pub theme: Theme,
    pub json_input: String,
    pub query_output: String,
}

#[derive(Debug, Clone)]
pub enum WindowCategory {
    Main,
    ToolsTime,
    ToolsJson2Csv,
    ToolsJson2Query,
}

#[derive(Debug, Clone)]
pub enum WindowContentMessage {
    Json2QueryInputChange(String),
    Json2QueryTransfer(),
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenWindow(WindowCategory),
    WindowOpened(window::Id, WindowCategory),
    WindowClosed(window::Id),
    ScaleInputChanged(window::Id, String),
    ScaleChanged(window::Id, String),
    TitleChanged(window::Id, String),
    ContentChanged(window::Id, WindowContentMessage),
}

impl Window {
    pub fn new(count: usize) -> Self {
        Self {
            category: WindowCategory::Main,
            title: format!("Window_{}", count),
            scale_input: "1.0".to_string(),
            current_scale: 1.0,
            theme: Theme::ALL[count % Theme::ALL.len()].clone(),
            json_input: String::new(),
            query_output: String::new(),
        }
    }

    pub fn view(&self, id: window::Id) -> Element<Message> {
        let content = match self.category {
            WindowCategory::Main => {
                let btn = button(text("Json转Csv").shaping(text::Shaping::Advanced))
                    .on_press(Message::OpenWindow(WindowCategory::ToolsJson2Csv));
                let btn2 = button(text("时间转换").shaping(text::Shaping::Advanced))
                    .on_press(Message::OpenWindow(WindowCategory::ToolsTime));
                let btn3 = button(text("Json转Query").shaping(text::Shaping::Advanced))
                    .on_press(Message::OpenWindow(WindowCategory::ToolsJson2Query));
                scrollable(
                    container(
                        column![
                            row!(text("功能列表").shaping(text::Shaping::Advanced)),
                            row!(column![btn], column![btn2]),
                            row!(column![btn3])
                        ]
                        .spacing(10),
                    )
                    .width(Fill)
                    .align_x(Left),
                )
            }
            WindowCategory::ToolsTime => scrollable(
                column![text("时间工具").shaping(text::Shaping::Advanced)]
                    .spacing(50)
                    .width(Fill)
                    .align_x(Center),
            ),
            WindowCategory::ToolsJson2Csv => scrollable(
                column![text("json2csv")]
                    .spacing(50)
                    .width(Fill)
                    .align_x(Center),
            ),
            WindowCategory::ToolsJson2Query => {
                let input = text_input("输入JSON", &self.json_input)
                    .id("json-input")
                    .on_input(move |s| {
                        Message::ContentChanged(id, WindowContentMessage::Json2QueryInputChange(s))
                    })
                    .padding(20) // Increased padding
                    .size(20)
                    .width(iced::Length::Fill)
                    .align_x(Center);

                let convert_button = button(text("转换").shaping(text::Shaping::Advanced))
                    .on_press(Message::ContentChanged(
                        id,
                        WindowContentMessage::Json2QueryTransfer(),
                    ))
                    .padding(10)
                    .width(iced::Length::Fixed(100.0));

                let output = text_input("", &self.query_output)
                    .padding(15)
                    .size(20) // Increased the size of the input text
                    .width(iced::Length::Fill) // Make the input box fill the available width
                    .align_x(Center);
                scrollable(
                    column![
                        row![input].width(iced::Length::Fill).padding(10), // Center the input row
                        row![convert_button]
                            .width(iced::Length::Fill)
                            .padding(10)
                            .align_y(Center),
                        row![output].width(iced::Length::Fill).padding(10),
                    ]
                    .spacing(20)
                    .width(iced::Length::Fill)
                    .align_x(Center),
                )
            }
        };

        container(content).into()
    }

    pub fn update(&mut self, message: WindowContentMessage) {
        match message {
            WindowContentMessage::Json2QueryInputChange(input) => {
                self.json_input = input;
            }
            WindowContentMessage::Json2QueryTransfer() => {
                if let Ok(json_value) = serde_json::from_str(&self.json_input) {
                    self.query_output = self.json_to_query(&json_value);
                } else {
                    self.query_output = "Invalid JSON input".to_string();
                }
            }
        }
    }

    fn json_to_query(&self, json: &Value) -> String {
        let mut query_pairs = Vec::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if let Some(val_str) = value.as_str() {
                    query_pairs.push(format!("{}={}", key, val_str));
                } else if let Some(val_num) = value.as_f64() {
                    query_pairs.push(format!("{}={}", key, val_num));
                } else if let Some(val_bool) = value.as_bool() {
                    query_pairs.push(format!("{}={}", key, val_bool));
                }
            }
        }
        query_pairs.join("&")
    }
}

