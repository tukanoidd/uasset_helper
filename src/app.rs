mod dep_graph;
mod file_picker;
mod interactable_text;

use std::path::PathBuf;

use iced::{
    button, executor, pick_list, Alignment, Application, Column, Container, Element, Length,
    PickList, Row, Space, Text,
};
use iced_native::Command;

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
    current_tab: AppTab,

    // Header
    pick_list_tabs_state: pick_list::State<AppTab>,
    asset_file_picker_button_state: button::State,
    engine_folder_picker_button_state: button::State,

    // Body
    dep_tree_page: DepTreePage,
}

impl Application for GuiApp {
    type Executor = executor::Default;
    type Message = GuiAppMessage;
    type Flags = AssetDirs;

    fn new(asset_dirs: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                // System
                clipboard: arboard::Clipboard::new().unwrap(),

                // Cache
                asset_dirs,

                // State
                current_tab: AppTab::DependencyTree,

                // Header
                pick_list_tabs_state: Default::default(),
                asset_file_picker_button_state: Default::default(),
                engine_folder_picker_button_state: Default::default(),

                // Body
                dep_tree_page: Default::default(),
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

    fn view(&mut self) -> Element<'_, Self::Message> {
        let header = Self::header(
            &self.asset_dirs,
            self.current_tab,
            &mut self.pick_list_tabs_state,
            &mut self.asset_file_picker_button_state,
            &mut self.engine_folder_picker_button_state,
        );

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
}

impl GuiApp {
    fn header<'a>(
        asset_dirs: &AssetDirs,
        current_tab: AppTab,

        pick_list_tabs_state: &'a mut pick_list::State<AppTab>,
        asset_file_picker_button_state: &'a mut button::State,
        engine_folder_picker_button_state: &'a mut button::State,
    ) -> Element<'a, GuiAppMessage> {
        let pick_list_tabs = PickList::new(
            pick_list_tabs_state,
            &[AppTab::AssetInfo, AppTab::DependencyTree][..],
            Some(current_tab),
            GuiAppMessage::TabChanged,
        )
        .width(Length::FillPortion(2))
        .into();

        let asset_file_picker_text = asset_dirs.asset_file_name_str().unwrap_or_default();
        let asset_file_picker_tooltip = asset_dirs.asset_file_path_str();

        let asset_file_picker = file_picker::widget(
            asset_file_picker_button_state,
            "Pick Asset",
            &asset_file_picker_text,
            asset_file_picker_tooltip.clone(),
            (
                Some(GuiAppMessage::SaveTextToClipboard(
                    asset_file_picker_text.clone(),
                )),
                asset_file_picker_tooltip.map(GuiAppMessage::SaveTextToClipboard),
            ),
            GuiAppMessage::OpenFilePicker(true),
        )
        .width(Length::FillPortion(3))
        .into();

        let engine_folder_picker_text = asset_dirs.engine_dir_str().unwrap_or_default();

        let engine_folder_picker = file_picker::widget(
            engine_folder_picker_button_state,
            "Pick Engine Folder",
            &engine_folder_picker_text,
            None,
            (
                Some(GuiAppMessage::SaveTextToClipboard(
                    engine_folder_picker_text.clone(),
                )),
                None,
            ),
            GuiAppMessage::OpenFilePicker(true),
        )
        .width(Length::FillPortion(3))
        .into();

        Row::with_children(vec![
            pick_list_tabs,
            Space::with_width(Length::FillPortion(1)).into(),
            asset_file_picker,
            engine_folder_picker,
        ])
        .spacing(10)
        .max_height(30)
        .width(Length::Shrink)
        .into()
    }
}
