use std::path::{PathBuf, Path};

use iced::{Application, Theme, executor, widget::{container, button, text, column as col, text_input, row, scrollable}, Command, Settings, Alignment, Length, Font, Color, theme::Custom};
use regex::Regex;

mod client;
mod server;
mod header;
mod parse_socket;

use server::{NoFTPServer, ServerSettings};
use parse_socket::{parse_socket, IPValidationWarning, IPValidationError, IPValidationMessage};


const DEFAULT_PORT: u16 = 24873;
const DEFAULT_DOWNLOADS_PATH: &'static str = "downloads";

const SETTINGS_PATH: &'static str = "noftp_settings.toml";

#[cfg(test)]
mod tests;

fn main() {
    //std::fs::File::create("downloads/a.txt").unwrap();
    App::run(Settings {
        antialiasing: true,
        ..Settings::default()
    }).unwrap();
}

#[derive(Debug, Clone, Copy)]
enum GUITab {
    Menu,
    Settings,
    Transfer,
    FriendIPs
}

struct GUIState {
    tab: GUITab
}

enum WarnErr {
    Warn(String),
    Err(String)
}

struct SettingsTab {
    port: String,
    add_ip: String,
    message: Option<WarnErr>,
    download_path: String
}

struct AppSettings {
    server_settings: ServerSettings,
    ips: Vec<String>
}

impl AppSettings {
    fn save(&self) {
        let mut settings_toml = toml::map::Map::new();
        settings_toml.insert("port".to_string(), toml::Value::Integer(self.server_settings.port as i64));
        settings_toml.insert("ips".to_string(),
            toml::Value::Array(
                self.ips.iter()
                    .map(|v| toml::Value::String(v.clone()))
                    .collect()
            )
        );
        settings_toml.insert("download_path".to_string(), toml::Value::String(self.server_settings.download_path.clone()));

        let settings_toml = toml::Value::Table(settings_toml);
        std::fs::write(SETTINGS_PATH, toml::to_string_pretty(&settings_toml).unwrap()).unwrap();
    }

    fn load() -> AppSettings {
        if let Ok(file) = std::fs::read_to_string(SETTINGS_PATH) {
            if let Ok(toml::Value::Table(settings)) = toml::from_str::<toml::Value>(&file) {
                Self::load_settings(settings)
            } else {
                AppSettings::default()
            }
        } else {
            AppSettings::default()
        }
    }

    fn load_settings(mut settings: toml::map::Map<String, toml::Value>) -> AppSettings {
        let port = if let Some(toml::Value::Integer(port)) = settings.remove("port"){
            port as u16
        } else {
            DEFAULT_PORT
        };

        let ips = if let Some(toml::Value::Array(ips)) = settings.remove("ips"){
            ips.into_iter()
                .filter_map(|value|
                    if let toml::Value::String(value) = value {
                        Some(value)
                    } else {
                        None
                    }
                ).collect()
        } else {
            vec![]
        };

        let download_path = if let Some(toml::Value::String(download_path)) = settings.remove("download_path"){
            download_path
        } else {
            DEFAULT_DOWNLOADS_PATH.to_string()
        };

        AppSettings {
            ips,
            server_settings: ServerSettings {
                port,
                download_path,
            }
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ips: vec![],
            server_settings: ServerSettings {
                port: DEFAULT_PORT,
                download_path: DEFAULT_DOWNLOADS_PATH.to_string(),
            }
        }
    }
}

struct TransferTab {
    selected_ip: Option<usize>,
    hovering_files: bool,
    transfering_files: Vec<PathBuf>
}

struct App {
    server: NoFTPServer,
    state: GUIState,
    settings_tab: SettingsTab,
    settings: AppSettings,
    transfer: TransferTab
}

#[derive(Debug, Clone)]
enum SettingChange {
    Port(String),
    Ip(String),
    DownloadPath(String),
}

#[derive(Debug, Clone)]
enum AppMessage {
    ChangeTab(GUITab),
    ChangeSetting(SettingChange),
    ResetUnsetSettings,
    ApplySettings,
    MessageList(Vec<Self>),
    DeleteIp(usize),
    AddIp,
    AddFileDialog,
    EventOcurred(iced::event::Event),
    SelectIp(usize),
    DeleteFile(usize),
    SendFiles,
    ClearFriendIpMessage,
    ExploreDownloadDirectory
}

enum FileDragEvent {
    FileHovered(PathBuf),
    FileDropped(PathBuf),
    FilesHoveredLeft
}

type Element<'a> = iced::Element<'a, AppMessage, iced::Renderer<Theme>>;

#[derive(Default)]
struct AppFlags;

impl Application for App {
    type Executor = executor::Default;

    type Message = AppMessage;

    type Theme = Theme;

    type Flags = AppFlags;

    fn new(_: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let settings = AppSettings::load();
        let server = NoFTPServer::new(settings.server_settings.clone());

        (
            App {
                server,
                state: GUIState {
                    tab: GUITab::Menu
                },
                settings_tab: SettingsTab {
                    port: settings.server_settings.port.to_string(),
                    add_ip: "".to_string(),
                    message: None,
                    download_path: "downloads".to_string(),
                },
                settings,
                transfer: TransferTab {
                    selected_ip: None,
                    hovering_files: false,
                    transfering_files: Vec::new()
                }
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "NoFTP".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::ChangeTab(tab) => self.state.tab = tab,
            AppMessage::ChangeSetting(setting) => self.change_setting(setting),
            AppMessage::ApplySettings => self.apply_settings(),
            AppMessage::MessageList(messages) => {
                #[allow(unused_must_use)]
                for message in messages {
                    self.update(message);
                }
            },
            AppMessage::DeleteIp(ip_index) => self.delete_ip(ip_index),
            AppMessage::AddIp => self.add_ip(),
            AppMessage::ResetUnsetSettings => self.reset_unset_setting(),
            AppMessage::AddFileDialog => {
                if let Ok(mut files) = native_dialog::FileDialog::new().show_open_multiple_file() {
                    self.transfer.transfering_files.append(&mut files);
                };
            },
            AppMessage::EventOcurred(event) => {
                if let Some(event) = self.handle_event(event) {
                    match event {
                        FileDragEvent::FileHovered(_) => self.transfer.hovering_files = true,
                        FileDragEvent::FileDropped(path) => {
                            self.transfer.transfering_files.push(path);
                            self.transfer.hovering_files = false
                        },
                        FileDragEvent::FilesHoveredLeft => self.transfer.hovering_files = false,
                    }
                }
            },
            AppMessage::SelectIp(ip) => self.transfer.selected_ip = Some(ip),
            AppMessage::DeleteFile(file_index) => {self.transfer.transfering_files.remove(file_index);},
            AppMessage::SendFiles => self.send_files(),
            AppMessage::ClearFriendIpMessage => self.settings_tab.message = None,
            AppMessage::ExploreDownloadDirectory => {
                let path = native_dialog::FileDialog::new().show_open_single_dir();
                if let Ok(Some(path)) = path {
                    self.settings_tab.download_path = path.to_str().unwrap_or("path with unkown characters").to_string();
                }
            },
        };

        Command::none()
    }

    fn view(&self) -> Element<'_> {
        match self.state.tab {
            GUITab::Menu => self.view_menu(),
            GUITab::Settings => self.view_settings(),
            GUITab::Transfer => self.view_transfer(),
            GUITab::FriendIPs => self.view_friend_ips(),
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::subscription::events().map(AppMessage::EventOcurred)
    }
}

impl App {
    fn view_menu(&self) -> Element<'_> {
        let column = col![
            text("Main Menu"),
            button(text("Settings")).on_press(AppMessage::ChangeTab(GUITab::Settings)),
            button(text("Transfer files")).on_press(AppMessage::ChangeTab(GUITab::Transfer)),
            button(text("File Dialog")).on_press(AppMessage::AddFileDialog)
        ].padding(20)
            .spacing(20)
            .max_width(500)
            .align_items(Alignment::Center);

        container(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn view_transfer(&self) -> Element<'_> {
        let ips_column = self.settings.ips.iter()
            .enumerate()
            .map(|(i, ip)| {
                let mut butt = button(text(ip)).on_press(AppMessage::SelectIp(i)).into();
                if let Some(selected_ip) = self.transfer.selected_ip {
                    if i == selected_ip {
                        butt = button(text(ip)).into()
                    }
                }

                butt
            }).collect();
        
        let ips_column = col(ips_column).padding(10).spacing(3).align_items(Alignment::End);

        let (files_column, files_close_column) = self.transfer.transfering_files.iter()
            .enumerate()
            .map(|(i, file_path)| {
                let file_path = file_path.to_str().unwrap_or("File with invalid characters");
                (
                    text(file_path).size(15).into(),
                    button(text("X").size(5)).on_press(AppMessage::DeleteFile(i)).into()
                )
            }).unzip();

        let files_column = row![
            col(files_column).spacing(5),
            col(files_close_column).spacing(5)
        ];

        let content = if self.transfer.hovering_files {
            col![
                text("DROP FILES HERE").size(80)
            ].padding(20)
                .spacing(20)
                .max_width(500)
                .align_items(Alignment::Center)
        } else {
            let send_files_button = if self.transfer.selected_ip.is_some() {
                button(text("Send files")).on_press(AppMessage::SendFiles)
            } else {
                button(text("Send files"))
            };

            col![
                text("Transfer"),
                button(text("Main Menu")).on_press(AppMessage::ChangeTab(GUITab::Menu)),
                row![
                    scrollable(ips_column),
                    col![
                        button(text("Add files")).on_press(AppMessage::AddFileDialog),
                        text("Drag files here").size(20),
                        scrollable(files_column)
                    ]
                ].height(Length::Fill),
                send_files_button
            ].padding(20)
                .spacing(20)
                .max_width(500)
                .align_items(Alignment::Center)
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn view_settings(&self) -> Element<'_> {
        let apply_button = if self.changed_settings() {
            button(text("Apply")).on_press(AppMessage::ApplySettings)
        } else {
            button(text("Apply"))
        };

        let column = col![
            text("Settings"),
            button(text("Main Menu")).on_press(AppMessage::MessageList(vec![
                AppMessage::ResetUnsetSettings,
                AppMessage::ChangeTab(GUITab::Menu)
            ])),
            col![
                col![
                    row![
                        text("Port: "),
                        text_input(&DEFAULT_PORT.to_string(), &self.settings_tab.port).on_input(|val| AppMessage::ChangeSetting(SettingChange::Port(val)))
                    ],
                    text(self.settings.server_settings.port),
                ].spacing(5),
                col![
                    row![
                        text("Download path: "),
                        text_input(DEFAULT_DOWNLOADS_PATH, &self.settings_tab.download_path).on_input(|val| AppMessage::ChangeSetting(SettingChange::DownloadPath(val))),
                        button("Explore").on_press(AppMessage::ExploreDownloadDirectory)
                    ],
                    text(&self.settings.server_settings.download_path),
                ].spacing(5)
            ].align_items(Alignment::Start)
                .spacing(20),
            button(text("Friend IPs")).on_press(AppMessage::ChangeTab(GUITab::FriendIPs)),
            apply_button
        ].padding(20)
            .spacing(20)
            .max_width(500)
            .align_items(Alignment::Center);

        container(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn view_friend_ips(&self) -> Element<'_> {
        let column_elements = self.settings.ips.iter()
            .enumerate()
            .map(|(i, ip)| {
                row![
                    text(ip),
                    button(text("X")).on_press(AppMessage::DeleteIp(i))
                ].spacing(3).into()
            }).collect();

        let ips_column = scrollable(
            col(column_elements)
                .align_items(Alignment::End)
                .padding(20)
                .spacing(5)
            ).height(Length::Fill)
                .width(200);

        let mut column = col!(
            ips_column,
            row![
                text("Add IP:"),
                text_input("IP", &self.settings_tab.add_ip)
                    .on_input(|val| AppMessage::ChangeSetting(SettingChange::Ip(val)))
                    .on_submit(AppMessage::AddIp),
                button(text("Add")).on_press(AppMessage::AddIp)
            ],
        );

        if let Some(message) = &self.settings_tab.message {
            let message = match message {
                WarnErr::Warn(message) => text(message).style(Color::from_rgba8(255, 0, 255, 1.0)),
                WarnErr::Err(message) => text(message).style(Color::from_rgba8(255, 0, 0, 1.0))
            };
            column = column.push(message);
        }

        let column = column.push(
            button(text("Return")).on_press(
                AppMessage::MessageList(vec![
                    AppMessage::ClearFriendIpMessage,
                    AppMessage::ChangeTab(GUITab::Settings)
                ])
            )
        ).padding(20)
            .spacing(20)
            .max_width(500)
            .align_items(Alignment::Center);

        container(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn apply_settings(&mut self) {
        let mut changed_server_setting = false;

        let mut port = DEFAULT_PORT;
        if self.settings.server_settings.port.to_string() != self.settings_tab.port {
            if let Ok(new_port) = self.settings_tab.port.parse() {
                port = new_port;
                changed_server_setting = true;
            } else if self.settings_tab.port.len() == 0 {
                port = DEFAULT_PORT;
                changed_server_setting = true;
            }
        }

        if self.settings.server_settings.download_path.to_string() != self.settings_tab.download_path {
            changed_server_setting = true;
        }

        if changed_server_setting {
            self.settings.server_settings.port = port;
            if self.settings_tab.download_path.len() > 0 {
                self.settings.server_settings.download_path = self.settings_tab.download_path.clone();
            } else {
                self.settings.server_settings.download_path = DEFAULT_DOWNLOADS_PATH.to_string()
            }
            self.server.restart(self.settings.server_settings.clone());
        }

        self.save_settings()
    }

    fn changed_settings(&self) -> bool {
        let old_port = self.settings.server_settings.port;
        let new_port = &self.settings_tab.port;
        if // The ports are different. But if the field is empty, check that the port is different to the default
            &old_port.to_string() != new_port
            && (new_port.len() > 0
                || (
                    new_port.len() == 0
                    && old_port != DEFAULT_PORT
                )
            )
        {
            return true
        }

        let old_download_path = &self.settings.server_settings.download_path;
        let new_download_path = &self.settings_tab.download_path;
        if // The paths are different. But if the field is empty, check that the path is different to the default
            old_download_path != new_download_path
            && (
                new_download_path.len() > 0
                || (
                    new_download_path.len() == 0
                    && old_download_path != DEFAULT_DOWNLOADS_PATH
                )
            )
        {
            return true
        }

        false
    }

    fn delete_ip(&mut self, ip_index: usize) {
        self.settings.ips.remove(ip_index);

        self.save_settings()
    }

    fn add_ip(&mut self) {
        match parse_socket(&self.settings_tab.add_ip) {
            Err(IPValidationMessage::Error(err)) => {
                let mut err = err.to_string();
                err.pop(); // remove final \n
                self.settings_tab.message = Some(WarnErr::Err(err));
            },
            Err(IPValidationMessage::Warning(warn, _)) => {
                let mut warn = warn.to_string();
                warn.pop(); // remove final \n
                self.settings_tab.message = Some(WarnErr::Warn(warn));

                self.add_ip_unchecked()
            },
            Ok(_) => {
                self.settings_tab.message = None;
                self.add_ip_unchecked()
            },
        };
    }

    fn add_ip_unchecked(&mut self) {
        self.settings.ips.push(self.settings_tab.add_ip.clone());
        self.settings_tab.add_ip = "".to_string();

        self.save_settings()
    }

    fn change_setting(&mut self, setting: SettingChange) {
        match setting {
            SettingChange::Port(port) => {
                if let Ok(_) = port.parse::<u16>() {
                    self.settings_tab.port = port
                } else if port.len() == 0 { // Allow to have an empty field
                    self.settings_tab.port = port
                }
            },
            SettingChange::Ip(ip) => {
                // TODO: Make this better, so only valid IPs are possible to be written, and also auto-insert the dots (.)
                // TODO: Make this accept IPv6
                if Regex::new(r"^[0-9\.:]*$").unwrap().is_match(&ip) {
                    self.settings_tab.add_ip = ip
                };
            },
            SettingChange::DownloadPath(path) => self.settings_tab.download_path = path,
        }
    }

    fn save_settings(&self) {
        self.settings.save()
    }

    fn reset_unset_setting(&mut self) {
        self.settings_tab.add_ip = "".to_string();
        self.settings_tab.port = self.settings.server_settings.port.to_string();
    }

    fn handle_event(&self, event: iced::Event) -> Option<FileDragEvent> {
        match event {
            iced::Event::Window(event) => match event {
                iced::window::Event::FileHovered(path) => Some(FileDragEvent::FileHovered(path)),
                iced::window::Event::FileDropped(path) => Some(FileDragEvent::FileDropped(path)),
                iced::window::Event::FilesHoveredLeft => Some(FileDragEvent::FilesHoveredLeft),
                _ => None
            },
            _ => None
        }
    }

    fn send_files(&self) {
        for file_path in self.transfer.transfering_files.iter() {
            if let Some(ip) = self.transfer.selected_ip {
                let ip = parse_socket(&self.settings.ips[ip]).unwrap();
                client::send_path(file_path, ip)
            }
        }
    }
}
