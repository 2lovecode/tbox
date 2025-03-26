use iced::{widget::text_editor, window};

#[derive(Debug, Clone)]
pub enum WindowCategory {
    Homepage,
    ToolsTime,
    ToolsJson2Csv,
    ToolsJson2Query,
}

#[derive(Debug, Clone)]
pub enum WindowContentMessage {
    Json2QueryInputAction(text_editor::Action),
    Json2QueryTransfer(),
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenWindow(WindowCategory),
    WindowOpened(window::Id, WindowCategory),
    WindowClosed(window::Id),
    // ScaleInputChanged(window::Id, String),
    // ScaleChanged(window::Id, String),
    // TitleChanged(window::Id, String),
    ContentChanged(window::Id, WindowContentMessage),
}