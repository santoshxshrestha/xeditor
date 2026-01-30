use iced::Element;
use iced::Length::Fill;
use iced::Length::FillPortion;
use iced::task::Task;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text_editor;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

struct Xeditor {
    content: text_editor::Content,
}

#[allow(unused)]
#[derive(Debug, Clone)]
enum Message {
    ActionPerformed(text_editor::Action),
    OpenFile,
    OpenedFile(Result<Arc<String>, ErrorKind>),
    NewFile,
    OpenDirectory,
    SaveFile,
}

impl Xeditor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                content: text_editor::Content::with_text(include_str!("./main.rs")),
            },
            Task::perform(
                read_file(format!(
                    "{}/src/main.rs",
                    std::env::var("CARGO_MANIFEST_DIR").unwrap()
                )),
                Message::OpenedFile,
            ),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(content) => {
                self.content.perform(content);
            }
            Message::OpenedFile(content) => {
                if let Ok(file_content) = content {
                    self.content = text_editor::Content::with_text(&file_content);
                }
            }
            _ => println!("Not implemented yet"),
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let editor_area = text_editor(&self.content)
            .placeholder("Type some thing bruth")
            .height(Fill)
            .on_action(Message::ActionPerformed);

        let editor_container = container(editor_area).width(FillPortion(9));

        let tree_area = text("File Tree Area").height(Fill).width(FillPortion(1));

        container(row![tree_area, editor_container])
            .padding(10)
            .center(Fill)
            .into()
    }
}

fn main() -> iced::Result {
    iced::application(Xeditor::new, Xeditor::update, Xeditor::view).run()
}

pub async fn read_file(path: impl AsRef<Path>) -> Result<Arc<String>, ErrorKind> {
    fs::read_to_string(path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
}
