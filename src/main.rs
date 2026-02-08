use iced::Alignment;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::Length::Fill;
use iced::Length::FillPortion;
use iced::Settings;
use iced::border;
use iced::highlighter;
use iced::keyboard;
use iced::task::Task;
use iced::theme::Base;
use iced::theme::Theme;
use iced::widget::Space;
use iced::widget::button;
use iced::widget::container;
use iced::widget::pane_grid;
use iced::widget::text;
use iced::widget::text_editor;
use iced::widget::text_editor::Position;
use iced::widget::{column, row};
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

#[derive(Debug, Clone)]
pub enum FileNode {
    File {
        name: String,
        path: Option<PathBuf>,
    },
    Directory {
        name: String,
        path: PathBuf,
        expanded: bool,
        children_nodes: Box<Option<Vec<FileNode>>>,
    },
}

struct Xeditor {
    content: text_editor::Content,
    tree_content: Vec<FileNode>,
    panes: pane_grid::State<PaneKind>,
    error: Option<Error>,
    path: Option<PathBuf>,
    is_dirty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneKind {
    Explorer,
    Editor,
}

#[allow(unused)]
#[derive(Debug, Clone)]
enum Message {
    ActionPerformed(text_editor::Action),
    PaneResized(pane_grid::ResizeEvent),
    OpenFile,
    OpenedFile(Result<(Arc<String>, PathBuf), Error>),
    OpenedTreeFile(Result<(Arc<String>, PathBuf), Error>),
    NewFile,
    OpenDirectory,
    OpenedDirectory(Result<Vec<FileNode>, Error>),
    OpenChildDirectory(PathBuf),
    OpenedChildDirectory(Result<(Vec<FileNode>, PathBuf), Error>),
    OpenTreeFile(PathBuf),
    SaveFile,
    SavedFile(Result<PathBuf, Error>),
}

impl Xeditor {
    fn new() -> (Self, Task<Message>) {
        let (mut panes, explorer) = pane_grid::State::new(PaneKind::Explorer);
        if let Some((_editor, split)) =
            panes.split(pane_grid::Axis::Vertical, explorer, PaneKind::Editor)
        {
            // Give the explorer a sensible starting width.
            panes.resize(split, 0.22);
        }

        (
            Self {
                content: text_editor::Content::new(),
                tree_content: vec![FileNode::File {
                    name: String::from("New File"),
                    path: None,
                }],
                panes,
                error: None,
                path: None,
                is_dirty: true,
            },
            Task::perform(
                read_directory(default_directory()),
                Message::OpenedDirectory,
            ),
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

            Message::PaneResized(event) => {
                self.panes.resize(event.split, event.ratio);
                Task::none()
            }

            Message::OpenedFile(content) => match content {
                Ok(content) => {
                    self.content = text_editor::Content::with_text(&content.0);
                    self.path = Some(content.1);
                    self.is_dirty = false;

                    let file_name = self
                        .path
                        .as_ref()
                        .and_then(|path| {
                            path.file_name()
                                .map(|name| String::from(name.to_string_lossy()))
                        })
                        .unwrap_or_else(|| String::from("Default"));

                    self.tree_content = vec![FileNode::File {
                        name: file_name,
                        path: self.path.clone(),
                    }];

                    Task::none()
                }
                Err(e) => {
                    self.error = Some(e);
                    Task::none()
                }
            },

            Message::OpenedTreeFile(content) => match content {
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

            Message::OpenTreeFile(path) => Task::perform(read_file(path), Message::OpenedTreeFile),

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
                self.tree_content = vec![FileNode::File {
                    name: String::from("New File"),
                    path: None,
                }];
                Task::none()
            }

            Message::OpenDirectory => Task::perform(pick_directory(), Message::OpenedDirectory),

            Message::OpenedDirectory(dir_list) => match dir_list {
                Ok(contents) => {
                    self.tree_content = contents;
                    Task::none()
                }
                Err(e) => {
                    self.error = Some(e);
                    Task::none()
                }
            },
            Message::OpenChildDirectory(path) => {
                if toggle_dir_expanded(&mut self.tree_content, &path) {
                    Task::none()
                } else {
                    Task::perform(read_child_directory(path), Message::OpenedChildDirectory)
                }
            }

            Message::OpenedChildDirectory(Ok((child_node, path))) => {
                set_dir_children(&mut self.tree_content, &path, child_node);
                Task::none()
            }

            Message::OpenedChildDirectory(Err(error)) => {
                self.error = Some(error);
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

        let grid = pane_grid(&self.panes, |_, kind, _is_maximized| match kind {
            PaneKind::Explorer => {
                let mut tree_column = column![text("EXPLORER").size(12)];
                tree_column = tree_column.spacing(4);
                tree_column = tree_column.extend(render_tree_nodes(&self.tree_content, 0));

                let border = border;
                let tree_area = container(column![tree_column])
                    .width(Fill)
                    .padding(10)
                    .height(Fill)
                    .clip(true)
                    .style(move |_theme| container::Style {
                        text_color: Some(Color::WHITE),
                        background: Some(Theme::CatppuccinMocha.base().background_color.into()),
                        border,
                        shadow: iced::Shadow {
                            color: Color::from_rgb8(30, 32, 48),
                            offset: iced::Vector { x: 0.5, y: 1.0 },
                            blur_radius: 3.0,
                        },
                        snap: false,
                    });

                pane_grid::Content::new(tree_area)
            }
            PaneKind::Editor => {
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
                    )
                    .key_binding(|key_press| match key_press.key.as_ref() {
                        keyboard::Key::Character("s") if key_press.modifiers.command() => {
                            Some(text_editor::Binding::Custom(Message::SaveFile))
                        }
                        keyboard::Key::Character("o") if key_press.modifiers.command() => {
                            if key_press.modifiers.shift() {
                                Some(text_editor::Binding::Custom(Message::OpenDirectory))
                            } else {
                                Some(text_editor::Binding::Custom(Message::OpenFile))
                            }
                        }
                        keyboard::Key::Character("n") if key_press.modifiers.command() => {
                            Some(text_editor::Binding::Custom(Message::NewFile))
                        }
                        _ => text_editor::Binding::from_key_press(key_press),
                    });

                let editor_container = container(editor_area).width(Fill).height(Fill);

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

                pane_grid::Content::new(column![editor_container, status_bar].height(Fill))
            }
        })
        .spacing(6)
        .min_size(140)
        .on_resize(12, Message::PaneResized);

        let border = border;
        container(grid)
            .padding(10)
            .center(Fill)
            .style(move |_theme| container::Style {
                text_color: Some(Color::WHITE),
                background: Some(Theme::CatppuccinMocha.base().background_color.into()),
                border,
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

fn icon<'a>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONTS: Font = Font::with_name("xeditor");
    text(codepoint).font(ICON_FONTS).into()
}

fn directory_icon<'a>() -> Element<'a, Message> {
    icon('\u{E001}')
}

fn file_icon<'a>() -> Element<'a, Message> {
    icon('\u{E002}')
}

fn closed_chevron<'a>() -> Element<'a, Message> {
    icon('\u{F001}')
}

fn opened_chevron<'a>() -> Element<'a, Message> {
    icon('\u{F002}')
}

fn render_tree_nodes<'a>(nodes: &'a [FileNode], depth: usize) -> Vec<Element<'a, Message>> {
    let mut out: Vec<Element<'a, Message>> = Vec::new();
    let indent = (depth as f32) * 14.0;

    for node in nodes {
        match node {
            FileNode::File { name, path } => {
                let chevron = text("").width(Length::Fixed(10.0));
                let label = text(name);
                let row_content = row![chevron, file_icon(), label].spacing(6);

                if let Some(path) = path {
                    out.push(
                        row![
                            Space::new().width(Length::Fixed(indent)),
                            button(container(row_content).width(Fill).align_x(Alignment::Start))
                                .on_press(Message::OpenTreeFile(path.clone()))
                                .padding([2, 6])
                                .width(Fill)
                                .style(button::text)
                        ]
                        .into(),
                    );
                } else {
                    out.push(
                        row![
                            Space::new().width(Length::Fixed(indent)),
                            container(container(row_content).width(Fill).align_x(Alignment::Start))
                                .padding([2, 6])
                                .width(Fill)
                        ]
                        .into(),
                    );
                }
            }
            FileNode::Directory {
                name,
                path,
                expanded,
                children_nodes,
            } => {
                let chevron = if *expanded {
                    // text("v").width(Length::Fixed(4.0))
                    opened_chevron()
                } else {
                    // text(">").width(Length::Fixed(4.0))
                    closed_chevron()
                };
                let label = text(name);
                out.push(
                    row![
                        Space::new().width(Length::Fixed(indent)),
                        button(
                            container(row![chevron, label].spacing(6))
                                .width(Fill)
                                .align_x(Alignment::Start)
                        )
                        .on_press(Message::OpenChildDirectory(path.clone()))
                        .padding([2, 6])
                        .width(Fill)
                        .style(button::text)
                    ]
                    .into(),
                );

                if *expanded
                    && let Some(children) = children_nodes.as_deref() {
                        out.extend(render_tree_nodes(children, depth + 1));
                    }
            }
        }
    }

    out
}

fn toggle_dir_expanded(nodes: &mut [FileNode], target: &PathBuf) -> bool {
    for node in nodes {
        if let FileNode::Directory {
            path,
            expanded,
            children_nodes,
            ..
        } = node
        {
            if path == target {
                if *expanded {
                    *expanded = false;
                    return true;
                }

                if children_nodes.is_some() {
                    *expanded = true;
                    return true;
                }

                // Not loaded yet -> caller should load it.
                *expanded = true;
                return false;
            }

            if let Some(children) = children_nodes.as_deref_mut()
                && toggle_dir_expanded(children, target) {
                    return true;
                }
        }
    }

    false
}

fn set_dir_children(nodes: &mut [FileNode], target: &PathBuf, children: Vec<FileNode>) -> bool {
    let mut children = Some(children);
    set_dir_children_inner(nodes, target, &mut children)
}

fn set_dir_children_inner(
    nodes: &mut [FileNode],
    target: &PathBuf,
    children: &mut Option<Vec<FileNode>>,
) -> bool {
    for node in nodes {
        if let FileNode::Directory {
            path,
            children_nodes,
            ..
        } = node
        {
            if path == target {
                **children_nodes = children.take();
                return true;
            }

            if let Some(existing) = children_nodes.as_deref_mut()
                && set_dir_children_inner(existing, target, children) {
                    return true;
                }
        }
    }

    false
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

// This is just read the content of the directory and return the vector  fo the fielNone
async fn read_directory(path: PathBuf) -> Result<Vec<FileNode>, Error> {
    let mut read_dir = fs::read_dir(&path)
        .await
        .map_err(|error| error.kind())
        .map_err(Error::IoError)?;

    let mut childrens: Vec<FileNode> = Vec::new();
    while let Some(entry) = read_dir.next_entry().await.unwrap() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if path.is_dir() {
            childrens.push(FileNode::Directory {
                name,
                path,
                expanded: false,
                children_nodes: Box::new(None),
            });
        } else {
            childrens.push(FileNode::File {
                name,
                path: Some(path),
            });
        }
    }
    Ok(childrens)
}

// Repeated for a reason but need to fix this
async fn read_child_directory(path: PathBuf) -> Result<(Vec<FileNode>, PathBuf), Error> {
    let mut read_dir = fs::read_dir(&path)
        .await
        .map_err(|error| error.kind())
        .map_err(Error::IoError)?;

    let mut childrens: Vec<FileNode> = Vec::new();
    while let Some(entry) = read_dir.next_entry().await.unwrap() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if path.is_dir() {
            childrens.push(FileNode::Directory {
                name,
                path,
                expanded: false,
                children_nodes: Box::new(None),
            });
        } else {
            childrens.push(FileNode::File {
                name,
                path: Some(path),
            });
        }
    }
    Ok((childrens, path))
}

async fn pick_file() -> Result<(Arc<String>, PathBuf), Error> {
    let path = rfd::AsyncFileDialog::new()
        .set_title("Choose a file")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?
        .path()
        .to_owned();

    read_file(path).await
}

async fn pick_directory() -> Result<Vec<FileNode>, Error> {
    let path = rfd::AsyncFileDialog::new()
        .set_title("Choose a directory")
        .pick_folder()
        .await
        .ok_or(Error::DialogClosed)?
        .path()
        .to_owned();
    read_directory(path).await
}

fn default_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
