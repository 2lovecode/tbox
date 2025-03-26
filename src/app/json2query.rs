use iced::widget::{button, column, row, scrollable, text, text_editor, text_input};
use iced::{Theme, Renderer, Center};
use iced::widget::Scrollable;
use iced::window::Id;
use crate::app::message::{Message, WindowContentMessage};

pub fn view<'a>(id:Id, input: String, output: String, editor_content: &'a text_editor::Content,) -> Scrollable<'a, Message, Theme, Renderer> {
    
    let _input_value = input.clone();
    let output_value = output.clone();
    // let input = text_input("输入JSON", &input_value)
    //     .id("json-input")
    //     .on_input(move |s| {
    //         Message::ContentChanged(id, WindowContentMessage::Json2QueryInputChange(s))
    //     })
    //     .padding(5) // Increased padding
    //     .size(20)
    //     .width(iced::Length::Fill);


    let input = text_editor(editor_content)
    .on_action(move |action| {
        match action {
            text_editor::Action::Edit(edit) => {
                Message::ContentChanged(id, WindowContentMessage::Json2QueryInputAction(text_editor::Action::Edit(edit.clone())))
            },
            text_editor::Action::Move(motion) => {
                Message::ContentChanged(id, WindowContentMessage::Json2QueryInputAction(text_editor::Action::Move(motion.clone())))
            },
            text_editor::Action::SelectAll => {
                Message::ContentChanged(id, WindowContentMessage::Json2QueryInputAction(text_editor::Action::SelectAll))
            },
            _ => {
                Message::ContentChanged(id, WindowContentMessage::None)
            },
        }
    })
    .size(20)
    // .width(iced::Length::Fixed(200.0))
    .height(iced::Length::Fixed(300.0));
    let convert_button = button(text("转换").shaping(text::Shaping::Advanced))
        .on_press(Message::ContentChanged(
            id,
            WindowContentMessage::Json2QueryTransfer(),
        ))
        .padding(10)
        .width(iced::Length::Fixed(100.0));

    let output = text_input("", &output_value)
        .padding(5)
        .size(20) // Increased the size of the input text
        .width(iced::Length::Fill);

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
        .width(iced::Length::Fill),
    )
    
}