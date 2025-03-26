use iced::window;

#[derive(Debug, Clone)]
pub enum WindowCategory {
    Homepage,
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
    // ScaleInputChanged(window::Id, String),
    // ScaleChanged(window::Id, String),
    // TitleChanged(window::Id, String),
    ContentChanged(window::Id, WindowContentMessage),
}