use crate::app::message::{Message, WindowCategory};
use iced_aw::{helpers::card, style};
use iced::{
    widget::{Column, Scrollable, Text, button, column, container, row, scrollable, text},
    Element, Fill, Theme, Renderer, Left,
};

pub fn view<'a>() -> Scrollable<'a, Message, Theme, Renderer> {
    let func1: Element<'_, Message> = 
        card(
        Text::new("Json转Csv").center().shaping(text::Shaping::Advanced),
            Column::new().push(Text::new("Json 转 CSV").shaping(text::Shaping::Advanced))
        )
        .foot(button(text("打开").shaping(text::Shaping::Advanced))
        .on_press(Message::OpenWindow(WindowCategory::ToolsJson2Csv)))
        .style(style::card::primary)
        .max_width(150.0)
        .into();
    
    let func2: Element<'_, Message> = 
        card(
        Text::new("时间转换").center().shaping(text::Shaping::Advanced),
            Column::new().push(Text::new("时间转换").shaping(text::Shaping::Advanced))
        )
        .foot(button(text("打开").shaping(text::Shaping::Advanced))
        .on_press(Message::OpenWindow(WindowCategory::ToolsTime)))
        .style(style::card::primary)
        .max_width(150.0)
        .into();

    let func3: Element<'_, Message> = 
        card(
        Text::new("Json转Query").center().shaping(text::Shaping::Advanced),
            Column::new().push(Text::new("Json 转 Query").shaping(text::Shaping::Advanced))
        )
        .foot(button(text("打开").shaping(text::Shaping::Advanced))
        .on_press(Message::OpenWindow(WindowCategory::ToolsJson2Query)))
        .style(style::card::primary)
        .max_width(150.0)
        .into();

    scrollable(
        container(
            column![
                row!(
                    column![func1].padding(10),
                    column![func2].padding(10),
                    column![func3].padding(10),
                )
            ]
            .spacing(10),
        )
        .width(Fill)
        .align_x(Left),
    )
}