use iced::Alignment;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Font;
use iced::Length::Fill;
use iced::Length::FillPortion;
use iced::Settings;
use iced::border;
use iced::highlighter;
use iced::task::Task;
use iced::theme::Base;
use iced::theme::Theme;
use iced::widget::Tooltip;
use iced::widget::button;
use iced::widget::container;
use iced::widget::text;
use iced::widget::text_editor;
use iced::widget::text_editor::Position;
use iced::widget::tooltip;
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
    is_dirty: bool,
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
    SavedFile(Result<PathBuf, Error>),
}

impl Xeditor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                error: None,
                path: None,
                is_dirty: true,
            },
            Task::perform(read_file(default_file()), Message::OpenedFile),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(content) => {
                self.is_dirty = self.is_dirty || content.is_edit();

                self.error = None;

                self.content.perform(content);

                Task::none()
            }

            Message::OpenedFile(content) => match content {
                Ok(content) => {
                    self.content = text_editor::Content::with_text(&content.0);
                    self.path = Some(content.1);
                    self.is_dirty = false;

                    Task::none()
                }
                Err(e) => {
                    self.error = Some(e);
                    Task::none()
                }
            },
            Message::OpenFile => Task::perform(pick_file(), Message::OpenedFile),

            Message::SaveFile => {
                let text = self.content.text();
                Task::perform(save_file(self.path.clone(), text), Message::SavedFile)
            }

            Message::SavedFile(Ok(path)) => {
                self.path = Some(path);
                self.is_dirty = false;

                Task::none()
            }
            Message::SavedFile(Err(error)) => {
                self.error = Some(error);
                Task::none()
            }

            Message::NewFile => {
                self.content = text_editor::Content::new();
                self.path = None;
                self.is_dirty = true;
                Task::none()
            }

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
        let open_button = create_button("open a file", Some(Message::OpenFile), open_icon());
        let save_button = create_button(
            "save file",
            self.is_dirty.then_some(Message::SaveFile),
            save_icon(),
        );
        let new_file_button = create_button("Create new file", Some(Message::NewFile), new_icon());

        let controls = row![open_button, save_button, new_file_button]
            .height(30)
            .width(100)
            .padding(10)
            .spacing(10);

        let editor_area = text_editor(&self.content)
            .placeholder("Type some thing bruth")
            .height(Fill)
            .on_action(Message::ActionPerformed)
            .highlight(
                self.path
                    .as_ref()
                    .and_then(|path| path.extension())
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("rs"),
                highlighter::Theme::Base16Mocha,
            );

        let editor_container = container(editor_area).width(FillPortion(9));

        // TODO: Need to parse the the path and then get the name of the file and directory and
        // read recursively
        let tree_area = container(column![text("file_name")])
            .width(FillPortion(1))
            .padding(10)
            .height(Fill)
            .style(move |_theme| container::Style {
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

        let status_bar = {
            let status = if let Some(Error::IoError(error)) = self.error {
                text(error.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) => text(path).size(14),
                    None => text("New File"),
                }
            };

            let position = {
                let Position { line, column } = self.content.cursor().position;
                text(format!("Ln {}, Col {}", line + 1, column + 1))
                    .width(FillPortion(1))
                    .size(16)
                    .align_x(Alignment::End)
            };
            row![status, position]
        };

        container(row![
            tree_area,
            column![controls, editor_container, status_bar]
        ])
        .padding(10)
        .center(Fill)
        .style(move |_theme| container::Style {
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

fn create_button<'a>(
    tip: &'a str,
    message: Option<Message>,
    icon: Element<'a, Message>,
) -> Tooltip<'a, Message> {
    let is_disabled = message.is_none();
    tooltip(
        button(icon)
            .on_press_maybe(message)
            .padding([5, 10])
            .style(move |theme, status| {
                if is_disabled {
                    button::secondary(theme, status)
                } else {
                    button::primary(theme, status)
                }
            }),
        tip,
        tooltip::Position::Bottom,
    )
}

fn new_icon<'a>() -> Element<'a, Message> {
    icon('\u{E800}')
}

fn save_icon<'a>() -> Element<'a, Message> {
    icon('\u{E801}')
}

fn open_icon<'a>() -> Element<'a, Message> {
    icon('\u{F115}')
}

fn icon<'a>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONTS: Font = Font::with_name("xeditor-icons");
    text(codepoint).font(ICON_FONTS).into()
}

fn main() -> iced::Result {
    iced::application(Xeditor::new, Xeditor::update, Xeditor::view)
        .settings(Settings {
            default_font: Font::MONOSPACE,
            fonts: vec![include_bytes!("../fonts/xeditor.ttf").as_slice().into()],
            ..Settings::default()
        })
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

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    fs::write(&path, text)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok(path)
}
