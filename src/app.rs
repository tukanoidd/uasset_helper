mod dep_graph;
mod file_picker;
mod interactable_text;

use std::path::PathBuf;

use iced::{
    executor,
    widget::{Column, Container, PickList, Space, Text},
    Alignment, Application, Element, Length, Theme,
};
use iced_native::{row, Command};

use crate::{
    app::dep_graph::{DepTreePage, DepTreePageMsg},
    asset::AssetDirs,
    util::save_to_clipboard,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AppTab {
    AssetInfo,
    DependencyTree,
}

impl ToString for AppTab {
    #[inline]
    fn to_string(&self) -> String {
        self.title().to_string()
    }
}

impl AppTab {
    fn title(&self) -> &str {
        match self {
            AppTab::AssetInfo => "Asset Info",
            AppTab::DependencyTree => "Dependency Tree",
        }
    }
}

#[derive(Debug, Clone)]
pub enum GuiAppMessage {
    TabChanged(AppTab),

    SaveTextToClipboard(String),

    OpenFilePicker(bool),

    SetAssetPath(Option<PathBuf>),
    SetEnginePath(Option<PathBuf>),

    DepTreePage(DepTreePageMsg),
}

pub struct GuiApp {
    // System
    clipboard: arboard::Clipboard,

    // Cache
    asset_dirs: AssetDirs,

    // State
    theme: Theme,
    current_tab: AppTab,

    // Body
    dep_tree_page: DepTreePage,
}

impl Application for GuiApp {
    type Executor = executor::Default;
    type Message = GuiAppMessage;
    type Theme = Theme;
    type Flags = AssetDirs;

    fn new(asset_dirs: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                // System
                clipboard: arboard::Clipboard::new().unwrap(),

                // Cache
                asset_dirs: asset_dirs.clone(),

                // State
                theme: Theme::Dark,
                current_tab: AppTab::DependencyTree,

                // Body
                dep_tree_page: DepTreePage::new(asset_dirs),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "UAsset Helper".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            GuiAppMessage::TabChanged(new_tab) => {
                self.current_tab = new_tab;
            }
            GuiAppMessage::OpenFilePicker(asset) => {
                return Command::perform(file_picker::open(asset), move |path| match asset {
                    true => GuiAppMessage::SetAssetPath(path),
                    false => GuiAppMessage::SetEnginePath(path),
                })
            }
            GuiAppMessage::SetAssetPath(path) => {
                self.asset_dirs.update_asset_file(path);
            }
            GuiAppMessage::SetEnginePath(path) => {
                self.asset_dirs.update_engine_dir(path);
            }
            GuiAppMessage::DepTreePage(dep_tree_page_msg) => {
                return self.dep_tree_page.update(
                    dep_tree_page_msg,
                    &self.asset_dirs,
                    &mut self.clipboard,
                );
            }
            GuiAppMessage::SaveTextToClipboard(text) => {
                save_to_clipboard(&mut self.clipboard, text)
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let header = Self::header(&self.asset_dirs, self.current_tab);

        let body = match self.current_tab {
            AppTab::AssetInfo => Container::new(Text::new("Asset Info")).into(),
            AppTab::DependencyTree => self.dep_tree_page.view().map(GuiAppMessage::DepTreePage),
        };

        Container::new(
            Column::with_children(vec![header, body])
                .spacing(20)
                .align_items(Alignment::Center),
        )
        .padding(20)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }
}

impl GuiApp {
    fn header<'a>(asset_dirs: &AssetDirs, current_tab: AppTab) -> Element<'a, GuiAppMessage> {
        let pick_list_tabs = PickList::new(
            &[AppTab::AssetInfo, AppTab::DependencyTree][..],
            Some(current_tab),
            GuiAppMessage::TabChanged,
        )
        .width(Length::FillPortion(2));

        let asset_file_picker_text = asset_dirs.asset_file_name_str().unwrap_or_default();
        let asset_file_picker_tooltip = asset_dirs.asset_file_path_str();

        let asset_file_picker = file_picker::widget(
            "Pick Asset",
            asset_file_picker_text.clone(),
            asset_file_picker_tooltip.clone(),
            (
                Some(GuiAppMessage::SaveTextToClipboard(asset_file_picker_text)),
                asset_file_picker_tooltip.map(GuiAppMessage::SaveTextToClipboard),
                None,
            ),
            GuiAppMessage::OpenFilePicker(true),
        )
        .width(Length::FillPortion(3));

        let engine_folder_picker_text = asset_dirs.engine_dir_str().unwrap_or_default();

        let engine_folder_picker = file_picker::widget(
            "Pick Engine Folder",
            engine_folder_picker_text.clone(),
            None,
            (
                Some(GuiAppMessage::SaveTextToClipboard(
                    engine_folder_picker_text,
                )),
                None,
                None,
            ),
            GuiAppMessage::OpenFilePicker(true),
        )
        .width(Length::FillPortion(3));

        Container::new(
            row![
                pick_list_tabs,
                Space::with_width(Length::FillPortion(1)),
                asset_file_picker,
                engine_folder_picker,
            ]
            .spacing(10)
            .width(Length::Shrink),
        )
        .max_height(30)
        .into()
    }
}
