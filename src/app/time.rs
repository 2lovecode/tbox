use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Fill, Theme, Renderer, Center};
use iced::widget::Scrollable;
use crate::app::message::{Message, WindowCategory};

pub fn view<'a>() -> Scrollable<'a, Message, Theme, Renderer> {
    scrollable(
        column![text("时间工具").shaping(text::Shaping::Advanced)]
            .spacing(50)
            .width(Fill)
            .align_x(Center),
    )
}