#![allow(unused)]
use iced::task::Task;
use iced::Theme;
use std::path::PathBuf;
use iced::widget::text_editor;
use iced::widget::text;

struct Xeditor{
    directory: Option<PathBuf>,
    file: Option<PathBuf>,
    theme: Theme,
    content: text_editor::Content,
    is_loading: bool,
}

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
        Self {
            directory: None,
            file: None,
            theme: Theme::CatppuccinMocha,
            content: text_editor::Content::new(),
            is_loading: false,
        }
    }
    fn update(&mut self, message: Message) {
        match message {
            _ => println!("Did some update")
        }
    }
    fn view(self){
        println!("This is the view")
    }
}



fn main() {
    println!("Hello iced");
}
