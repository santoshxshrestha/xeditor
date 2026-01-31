#![allow(unused)]
use iced::Alignment;
use iced::Element;
use iced::Length::Fill;
use iced::Length::FillPortion;
use iced::task::Task;
use iced::theme::Theme;
use iced::widget::button;
use iced::widget::container;
use iced::widget::text;
use iced::widget::text_editor;
use iced::widget::text_editor::Position;
use iced::widget::{column, row};
use rfd;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

struct Xeditor {
    content: text_editor::Content,
    error: Option<Error>,
    path: Option<PathBuf>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
enum Message {
    ActionPerformed(text_editor::Action),
    OpenFile,
    OpenedFile(Result<(Arc<String>, PathBuf), Error>),
    NewFile,
    OpenDirectory,
    SaveFile,
}

impl Xeditor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                error: None,
                path: None,
            },
            Task::perform(read_file(default_file()), Message::OpenedFile),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(content) => {
                self.content.perform(content);

                Task::none()
            }

            Message::OpenedFile(content) => match content {
                Ok(content) => {
                    self.content = text_editor::Content::with_text(&content.0);
                    self.path = Some(content.1);

                    Task::none()
                }
                Err(e) => {
                    self.error = Some(e);
                    Task::none()
                }
            },
            Message::OpenFile => Task::perform(pick_file(), Message::OpenedFile),

            _ => {
                println!("Not implemented yet");
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let open_button = button("Open File")
            .on_press(Message::OpenFile)
            .height(30)
            .width(100);

        let save_button = button("Save File")
            .on_press(Message::SaveFile)
            .height(30)
            .width(100);

        let controls = row![open_button, save_button].padding(10).spacing(10);

        let editor_area = text_editor(&self.content)
            .placeholder("Type some thing bruth")
            .height(Fill)
            .on_action(Message::ActionPerformed);

        let editor_container = container(editor_area).width(FillPortion(9));

        let tree_area = text("File Tree Area").height(Fill).width(FillPortion(1));

        let position = {
            let Position { line, column } = self.content.cursor().position;
            text(format!("Ln {}, Col {}", line + 1, column + 1))
                .width(FillPortion(1))
                .size(16)
                .align_x(Alignment::End)
        };

        container(row![
            tree_area,
            column![controls, editor_container, position]
        ])
        .padding(10)
        .center(Fill)
        .into()
    }
}

fn main() -> iced::Result {
    iced::application(Xeditor::new, Xeditor::update, Xeditor::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IoError(ErrorKind),
}

async fn read_file(path: PathBuf) -> Result<(Arc<String>, PathBuf), Error> {
    let contents = fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IoError)?;

    Ok((contents, path))
}

async fn pick_file() -> Result<(Arc<String>, PathBuf), Error> {
    let path = rfd::AsyncFileDialog::new()
        .set_title("Choose a file")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    read_file(path.path().to_owned()).await
}

fn default_file() -> PathBuf {
    PathBuf::from(format!(
        "{}/src/main.rs",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
}
