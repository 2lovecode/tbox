use iced::widget::{column, scrollable, text};
use iced::{Fill, Theme, Renderer, Center};
use iced::widget::Scrollable;
use crate::app::message::Message;

pub fn view<'a>() -> Scrollable<'a, Message, Theme, Renderer> {
    scrollable(
        column![text("json2csv")]
            .spacing(50)
            .width(Fill)
            .align_x(Center),
    )
}