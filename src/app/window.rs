use iced::alignment::Horizontal::{Left, Right};
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
    ToolsTime,
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
                let btn2 = button (text("时间转换").shaping(text::Shaping::Advanced))
                    .on_press(Message::OpenWindow(WindowCategory::ToolsTime));
                scrollable(
                    container(
                        column![
                            row!(text("功能列表").shaping(text::Shaping::Advanced)),
                            row!(column![btn], column![btn2]),
                        ]
                        .spacing(10),
                    ).width(Fill).align_x(Left),
                )
            }
            WindowCategory::ToolsTime => {
                scrollable(
                    column![text("时间工具").shaping(text::Shaping::Advanced)]
                        .spacing(50)
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