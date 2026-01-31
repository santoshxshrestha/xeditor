#![allow(unused)]
use iced::Alignment;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length::Fill;
use iced::Length::FillPortion;
use iced::border;
use iced::border::color;
use iced::task::Task;
use iced::theme;
use iced::theme::Base;
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
    SavedFile(Result<(), Error>),
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
        let border = Border {
            width: 1.0,
            color: Color::from_rgb8(69, 71, 90),
            radius: border::Radius {
                top_left: 5.0,
                top_right: 5.0,
                bottom_right: 5.0,
                bottom_left: 5.0,
            },
        };
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

        let parent_directory = match &self.path {
            Some(path) => path.parent().unwrap(),
            None => Path::new(""),
        };

        let entries = match parent_directory.read_dir() {
            Ok(entries) => entries,
            Err(_) => return text("Could not read directory").into(),
        };

        let tree_content = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                let file_name = entry.file_name().into_string().unwrap_or_default();
                if entry.path().is_dir() {
                    format!(" {}/", file_name)
                } else {
                    format!(" {}", file_name)
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        let tree_area = container(column![text(tree_content)])
            .width(FillPortion(1))
            .padding(10)
            .height(Fill)
            .style(move |theme| container::Style {
                text_color: Some(Color::WHITE),
                background: Some(Theme::CatppuccinMocha.base().background_color.into()),
                border: border,
                shadow: iced::Shadow {
                    color: Color::from_rgb8(30, 32, 48),
                    offset: iced::Vector { x: 0.5, y: 1.0 },
                    blur_radius: 3.0,
                },
                snap: false,
            });

        let position = {
            let Position { line, column } = self.content.cursor().position;
            text(format!("Ln {}, Col {}", line + 1, column + 1))
                .width(FillPortion(1))
                .size(16)
                .align_x(Alignment::End)
        };

        let bottom_bar = row![
            text(parent_directory.to_string_lossy().to_string()).align_x(Alignment::Start),
            position
        ];

        container(row![
            tree_area,
            column![controls, editor_container, bottom_bar]
        ])
        .padding(10)
        .center(Fill)
        .style(move |theme| container::Style {
            text_color: Some(Color::WHITE),
            background: Some(Theme::CatppuccinMocha.base().background_color.into()),
            border: border,
            shadow: iced::Shadow {
                color: Color::from_rgb8(30, 32, 48),
                offset: iced::Vector { x: 0.5, y: 1.0 },
                blur_radius: 3.0,
            },
            snap: false,
        })
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
