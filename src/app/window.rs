use iced::widget::{container, text_editor};
use iced::window;
use iced::{ Element, Theme };
use serde_json::Value;
use crate::app::message::{WindowCategory, Message, WindowContentMessage};

#[derive(Debug)]
pub struct Window {
    pub category: WindowCategory,
    pub title: String,
    // pub scale_input: String,
    pub current_scale: f64,
    pub theme: Theme,
    pub input: String,
    pub output: String,
    pub content: text_editor::Content,
}



impl Window {
    pub fn new(count: usize) -> Self {
        Self {
            category: WindowCategory::Homepage,
            title: format!("Window_{}", count),
            // scale_input: "1.0".to_string(),
            current_scale: 1.0,
            theme: Theme::ALL[count % Theme::ALL.len()].clone(),
            input: String::new(),
            output: String::new(),
            content: text_editor::Content::new(),
        }
    }

    pub fn view(&self, id: window::Id) -> Element<Message> {
        let content = match self.category {
            WindowCategory::Homepage => {
               super::homepage::view()
            }
            WindowCategory::ToolsTime => {
                super::time::view()
            },
            WindowCategory::ToolsJson2Csv => {
                super::json2csv::view()
            },
            WindowCategory::ToolsJson2Query => {
                let view_content = super::json2query::view(id, self.input.clone(), self.output.clone(), &self.content);
                view_content
            }
        };

        container(content).into()
    }

    pub fn update(&mut self, message: WindowContentMessage) {
        match message {
            WindowContentMessage::None => {}
            WindowContentMessage::Json2QueryInputAction(input) => {
                  // 
                  self.content.perform(input);
                // self.content = iced::widget::text_editor::Content::with_text(&input);
                self.input = self.content.text();
            }
            WindowContentMessage::Json2QueryTransfer() => {
                if let Ok(json_value) = serde_json::from_str(&self.input) {
                    self.output = self.json_to_query(&json_value);
                } else {
                    self.output = "Invalid JSON input".to_string();
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
