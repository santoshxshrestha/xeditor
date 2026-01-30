#![allow(unused)]
use iced::Element;
use iced::task::Task;
use iced::Theme;
use std::path::PathBuf;
use iced::widget::text_editor;
use iced::widget::text;

struct Xeditor{
    content: text_editor::Content,
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
        Self{
            content: text_editor::Content::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ActionPerformed(content) => {
                self.content.perform(content);
            }
            _ => println!("Not implemented yet"),
        }
    }

    fn view(&self)-> Element<'_ , Message>{
        text_editor(&self.content)
            .placeholder("Type some thing bruth")
            .on_action(Message::ActionPerformed)
            .into()
    }
}



fn main()-> iced::Result {
    iced::application(Xeditor::new, Xeditor::update, Xeditor::view).run()
}
