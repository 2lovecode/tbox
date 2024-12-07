use iced::window;
use iced::widget::{
    button, column, row, container, scrollable, text,
};
use iced::{Center, Element, Fill, Theme};

#[derive(Debug)]
pub struct Window {
    pub category: WindowCategory,
    pub title: String,
    pub scale_input: String,
    pub current_scale: f64,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub enum WindowCategory {
    Main,
    ToolsJson2Csv,
}


#[derive(Debug, Clone)]
pub enum Message {
    OpenWindow(WindowCategory),
    WindowOpened(window::Id, WindowCategory),
    WindowClosed(window::Id),
    ScaleInputChanged(window::Id, String),
    ScaleChanged(window::Id, String),
    TitleChanged(window::Id, String),
}




impl Window {
    pub fn new(count: usize) -> Self {
        Self {
            category: WindowCategory::Main,
            title: format!("Window_{}", count),
            scale_input: "1.0".to_string(),
            current_scale: 1.0,
            theme: Theme::ALL[count % Theme::ALL.len()].clone(),
        }
    }

    pub fn view(&self, id: window::Id) -> Element<Message> {
        
        let content = match self.category {
            WindowCategory::Main => {
                let btn = button (text("Json转Csv").shaping(text::Shaping::Advanced))
                    .on_press(Message::OpenWindow(WindowCategory::ToolsJson2Csv));
                scrollable(
                    column![
                        btn,
                        row!["Left", "Right"].spacing(10),
                        "Bottom"
                    ].spacing(50)
                        .width(Fill)
                        .align_x(Center),
                )
            }
            WindowCategory::ToolsJson2Csv => {
                // Message::TitleChanged(id, String::from("aaa"));
                scrollable(
                    column![text("json2csv")]
                        .spacing(50)
                        .width(Fill)
                        .align_x(Center),
                )
            }
        };

        container(content).center_x(200).into()
    }
}