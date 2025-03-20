use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Fill, Theme, Renderer, Left};
use iced::widget::Scrollable;
use crate::app::message::{Message, WindowCategory};

pub fn view<'a>() -> Scrollable<'a, Message, Theme, Renderer> {
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
                row!(
                    column![btn].padding(10),
                    column![btn2].padding(10),
                    column![btn3].padding(10)
                ),
            ]
            .spacing(10),
        )
        .width(Fill)
        .align_x(Left),
    )
}