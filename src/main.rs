#![allow(unused)]
use iced::Element;
use iced::task::Task;
use iced::Theme;
use std::path::PathBuf;
use iced::widget::text_editor;
use iced::widget::text;

struct Xeditor;


#[derive(Debug, Clone)]
enum Message {
    ActionPerformed(text_editor::Action),
    NewFile,
    OpenDirectory,
    OpenFile,
    SaveFile,
}

impl Xeditor {
    fn new() -> Self {
        Self
    }

    fn update(&mut self, message: Message) {
        match message {
            _=> println!("Matched some thing")
        }
    }

    fn view(&self)-> Element<'_ , Message>{
        text("Hello, iced").into()
    }
}



fn main()-> iced::Result {
    iced::application(Xeditor::new, Xeditor::update, Xeditor::view).run()
}
