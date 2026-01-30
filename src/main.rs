#![allow(unused)]
use iced::widget::{column, row};
use iced::Length::Fill;
use iced::Length::FillPortion;
use iced::widget::container;
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
            content: text_editor::Content::with_text(include_str!("./main.rs")),
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
        let editor_area = text_editor(&self.content)
            .placeholder("Type some thing bruth")
            .height(Fill)
            .on_action(Message::ActionPerformed);

        let editor_container = container(editor_area).width(FillPortion(9));

        let tree_area = text("File Tree Area")
            .height(Fill)
            .width(FillPortion(1));

        container(row![tree_area, editor_container]).padding(10).center(Fill).into()
    }
}



fn main()-> iced::Result {
    iced::application(Xeditor::new, Xeditor::update, Xeditor::view).run()
}
