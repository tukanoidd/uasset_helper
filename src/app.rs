mod file_picker;
mod interactable_text;

use std::{path::PathBuf, rc::Rc};

use enum_iterator::{all, Sequence};
use iced::{
    alignment::Horizontal, button, executor, pick_list, scrollable, text_input, tooltip, Alignment,
    Application, Button, Checkbox, Column, Container, Element, Length, PickList, Row, Scrollable,
    Space, Text, TextInput,
};
use iced_aw::graphics::IconText;
use iced_aw::{Icon, TabBar, TabLabel};
use iced_native::Command;
use itertools::Itertools;
use num_traits::{FromPrimitive, ToPrimitive};
use smart_default::SmartDefault;

use crate::app::interactable_text::interactive_text_tooltip;
use crate::asset::AssetOrigin;
use crate::util::{path_to_str, SortOrder};
use crate::{
    asset::{Asset, AssetDirs},
    dependency_tree::{DepTree, NodeID},
};

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Sequence)]
pub enum DepTreePageGraphSort {
    Id,
    Filename,
    Path,
    NumDeps,
}

impl ToPrimitive for DepTreePageGraphSort {
    fn to_i64(&self) -> Option<i64> {
        Some(*self as usize as i64)
    }

    fn to_usize(&self) -> Option<usize> {
        Some(*self as usize)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(*self as usize as u64)
    }
}

impl FromPrimitive for DepTreePageGraphSort {
    fn from_i64(n: i64) -> Option<Self> {
        match n {
            n if n == DepTreePageGraphSort::Id.to_i64().unwrap() => Some(DepTreePageGraphSort::Id),
            n if n == DepTreePageGraphSort::Filename.to_i64().unwrap() => {
                Some(DepTreePageGraphSort::Filename)
            }
            n if n == DepTreePageGraphSort::Path.to_i64().unwrap() => {
                Some(DepTreePageGraphSort::Path)
            }
            n if n == DepTreePageGraphSort::NumDeps.to_i64().unwrap() => {
                Some(DepTreePageGraphSort::NumDeps)
            }
            _ => None,
        }
    }

    fn from_usize(n: usize) -> Option<Self> {
        match n {
            n if n == DepTreePageGraphSort::Id as usize => Some(DepTreePageGraphSort::Id),
            n if n == DepTreePageGraphSort::Filename as usize => {
                Some(DepTreePageGraphSort::Filename)
            }
            n if n == DepTreePageGraphSort::Path as usize => Some(DepTreePageGraphSort::Path),
            n if n == DepTreePageGraphSort::NumDeps as usize => Some(DepTreePageGraphSort::NumDeps),
            _ => None,
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        match n {
            n if n == DepTreePageGraphSort::Id.to_u64().unwrap() => Some(DepTreePageGraphSort::Id),
            n if n == DepTreePageGraphSort::Filename.to_u64().unwrap() => {
                Some(DepTreePageGraphSort::Filename)
            }
            n if n == DepTreePageGraphSort::Path.to_u64().unwrap() => {
                Some(DepTreePageGraphSort::Path)
            }
            n if n == DepTreePageGraphSort::NumDeps.to_u64().unwrap() => {
                Some(DepTreePageGraphSort::NumDeps)
            }
            _ => None,
        }
    }
}

impl ToString for DepTreePageGraphSort {
    fn to_string(&self) -> String {
        match self {
            DepTreePageGraphSort::Id => "ID",
            DepTreePageGraphSort::Filename => "Filename",
            DepTreePageGraphSort::Path => "Path",
            DepTreePageGraphSort::NumDeps => "Num Dependencies",
        }
        .to_string()
    }
}

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Sequence)]
pub enum DepTreePageTab {
    Graph,
    Failures,
}

impl ToPrimitive for DepTreePageTab {
    fn to_i64(&self) -> Option<i64> {
        Some(*self as usize as i64)
    }

    fn to_usize(&self) -> Option<usize> {
        Some(*self as usize)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(*self as usize as u64)
    }
}

impl FromPrimitive for DepTreePageTab {
    fn from_i64(n: i64) -> Option<Self> {
        match n {
            n if n == DepTreePageTab::Graph.to_i64().unwrap() => Some(DepTreePageTab::Graph),
            n if n == DepTreePageTab::Failures.to_i64().unwrap() => Some(DepTreePageTab::Failures),
            _ => None,
        }
    }

    fn from_usize(n: usize) -> Option<Self> {
        match n {
            n if n == DepTreePageTab::Graph as usize => Some(DepTreePageTab::Graph),
            n if n == DepTreePageTab::Failures as usize => Some(DepTreePageTab::Failures),
            _ => None,
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        match n {
            n if n == DepTreePageTab::Graph.to_u64().unwrap() => Some(DepTreePageTab::Graph),
            n if n == DepTreePageTab::Failures.to_u64().unwrap() => Some(DepTreePageTab::Failures),
            _ => None,
        }
    }
}

impl ToString for DepTreePageTab {
    fn to_string(&self) -> String {
        match self {
            DepTreePageTab::Graph => "Graph",
            DepTreePageTab::Failures => "Failures",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum DepTreePageMsg {
    GenerateDependencyTree,
    SetMaxRecurseDepth(u32),
    TabChanged(DepTreePageTab),

    FilterChanged(usize, bool),
    SortChanged(DepTreePageGraphSort),
    SortOrderToggle,

    FooterInfoShow(Option<(NodeID, bool)>),
}

#[derive(SmartDefault)]
pub struct DepTreePageState {
    dep_tree: Option<DepTree>,

    #[default(DepTreePageTab::Graph)]
    tab: DepTreePageTab,

    #[default(10)]
    max_recurse_depth: u32,

    max_recurse_depth_text_input_state: text_input::State,
    gen_dep_tree_button_state: button::State,
    scrollable_dep_tree_viewer_state: scrollable::State,

    sort_pick_list_state: pick_list::State<DepTreePageGraphSort>,
    sort_order_button_state: button::State,

    graph_footer_scrollable_state: scrollable::State,

    #[default(vec![
        (AssetOrigin::Engine, true),
        (AssetOrigin::Project, true),
        (AssetOrigin::EnginePlugin, true),
        (AssetOrigin::ProjectPlugin, true)
    ])]
    filters: Vec<(AssetOrigin, bool)>,
    #[default(DepTreePageGraphSort::Id)]
    graph_sort: DepTreePageGraphSort,
    #[default(SortOrder::Ascending)]
    graph_sort_order: SortOrder,

    // ID and if info is extended
    footer_asset_show_min_info: Option<(NodeID, bool)>,
}

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
    dep_tree_page_state: DepTreePageState,
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
                dep_tree_page_state: Default::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "UAsset Helper".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<GuiAppMessage> {
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
            GuiAppMessage::DepTreePage(dep_tree_page_msg) => match dep_tree_page_msg {
                DepTreePageMsg::GenerateDependencyTree => {
                    let dependency_tree = DepTree::build_with_pb(
                        &self.asset_dirs,
                        self.dep_tree_page_state.max_recurse_depth,
                    );

                    match dependency_tree {
                        Ok(dependency_tree) => {
                            self.dep_tree_page_state.dep_tree = Some(dependency_tree);
                        }
                        Err(err) => {
                            log::error!("Failed to generate dependency tree: {}", err);
                        }
                    }
                }
                DepTreePageMsg::SetMaxRecurseDepth(new_max_recurse_depth) => {
                    self.dep_tree_page_state.max_recurse_depth = new_max_recurse_depth;
                }
                DepTreePageMsg::TabChanged(new_tab) => {
                    self.dep_tree_page_state.tab = new_tab;
                }

                DepTreePageMsg::FilterChanged(index, new_checked) => {
                    self.dep_tree_page_state.filters[index].1 = new_checked;
                }
                DepTreePageMsg::SortChanged(new_sort) => {
                    self.dep_tree_page_state.graph_sort = new_sort;
                }
                DepTreePageMsg::SortOrderToggle => {
                    self.dep_tree_page_state.graph_sort_order.toggle();
                }
                DepTreePageMsg::FooterInfoShow(new_show) => {
                    self.dep_tree_page_state.footer_asset_show_min_info = new_show;
                }
            },
            GuiAppMessage::SaveTextToClipboard(text) => match self.clipboard.set_text(text) {
                Ok(_) => {
                    log::info!("Copied text to clipboard");
                }
                Err(err) => {
                    log::error!("Failed to copy text to clipboard: {}", err);
                }
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let header = {
            let pick_list_tabs = PickList::new(
                &mut self.pick_list_tabs_state,
                &[AppTab::AssetInfo, AppTab::DependencyTree][..],
                Some(self.current_tab),
                GuiAppMessage::TabChanged,
            )
            .width(Length::FillPortion(2))
            .into();

            let asset_file_picker = file_picker::widget(
                &mut self.asset_file_picker_button_state,
                true,
                "Pick Asset",
                &self.asset_dirs.asset_file_name_str().unwrap_or_default(),
                self.asset_dirs.asset_file_path_str(),
            )
            .width(Length::FillPortion(3))
            .into();

            let engine_folder_picker = file_picker::widget(
                &mut self.engine_folder_picker_button_state,
                false,
                "Pick Engine Folder",
                &self.asset_dirs.engine_dir_str().unwrap_or_default(),
                None,
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
        };

        let body = match self.current_tab {
            AppTab::AssetInfo => Container::new(Text::new("Asset Info")).into(),
            AppTab::DependencyTree => {
                let max_recurse_limit = Container::new(
                    Row::with_children(vec![
                        Text::new("Max Recursion Depth: ").into(),
                        TextInput::new(
                            &mut self.dep_tree_page_state.max_recurse_depth_text_input_state,
                            "Type here...",
                            &self.dep_tree_page_state.max_recurse_depth.to_string(),
                            |new_number| {
                                GuiAppMessage::DepTreePage(DepTreePageMsg::SetMaxRecurseDepth(
                                    new_number
                                        .parse()
                                        .unwrap_or(self.dep_tree_page_state.max_recurse_depth),
                                ))
                            },
                        )
                        .width(Length::Units(30))
                        .size(20)
                        .padding([5, 10])
                        .into(),
                    ])
                    .spacing(10)
                    .align_items(Alignment::End),
                )
                .into();

                let gen_tree_button = Button::new(
                    &mut self.dep_tree_page_state.gen_dep_tree_button_state,
                    Text::new("Generate Dependency Tree").horizontal_alignment(Horizontal::Center),
                )
                .width(Length::Units(200))
                .on_press(GuiAppMessage::DepTreePage(
                    DepTreePageMsg::GenerateDependencyTree,
                ))
                .into();

                let controls = Row::with_children(vec![max_recurse_limit, gen_tree_button])
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .into();

                let tab_bar = Container::new(
                    TabBar::width_tab_labels(
                        self.dep_tree_page_state.tab as usize,
                        all::<DepTreePageTab>()
                            .map(|tab_val| TabLabel::Text(tab_val.to_string()))
                            .collect(),
                        |tab| {
                            GuiAppMessage::DepTreePage(DepTreePageMsg::TabChanged(
                                DepTreePageTab::from_usize(tab).unwrap(),
                            ))
                        },
                    )
                    .spacing(20)
                    .tab_width(Length::Units(300)),
                )
                .align_x(Horizontal::Center)
                .into();

                let (tab_body, footer) = {
                    let viewer = Scrollable::new(
                        &mut self.dep_tree_page_state.scrollable_dep_tree_viewer_state,
                    )
                    .height(Length::Fill)
                    .spacing(10)
                    .scrollbar_width(15)
                    .scroller_width(15);

                    match &self.dep_tree_page_state.dep_tree {
                        Some(dep_tree) => match self.dep_tree_page_state.tab {
                            DepTreePageTab::Graph => {
                                let filters = Row::with_children(
                                    self.dep_tree_page_state
                                        .filters
                                        .iter()
                                        .enumerate()
                                        .map(|(index, (filter, on))| {
                                            Checkbox::new(
                                                *on,
                                                filter.to_string(),
                                                move |new_checked| {
                                                    GuiAppMessage::DepTreePage(
                                                        DepTreePageMsg::FilterChanged(
                                                            index,
                                                            new_checked,
                                                        ),
                                                    )
                                                },
                                            )
                                            .into()
                                        })
                                        .collect_vec(),
                                )
                                .spacing(20);

                                let filters_sort = filters
                                    .push(Space::with_width(Length::Units(15)))
                                    .push(
                                        PickList::new(
                                            &mut self.dep_tree_page_state.sort_pick_list_state,
                                            all::<DepTreePageGraphSort>().collect_vec(),
                                            Some(self.dep_tree_page_state.graph_sort),
                                            |new_sort| {
                                                GuiAppMessage::DepTreePage(
                                                    DepTreePageMsg::SortChanged(new_sort),
                                                )
                                            },
                                        )
                                        .width(Length::Shrink)
                                        .padding([5, 10])
                                        .text_size(16),
                                    )
                                    .push(
                                        Button::new(
                                            &mut self.dep_tree_page_state.sort_order_button_state,
                                            match self.dep_tree_page_state.graph_sort_order {
                                                SortOrder::Ascending => {
                                                    IconText::new(Icon::SortDownAlt)
                                                }
                                                SortOrder::Descending => {
                                                    IconText::new(Icon::SortUpAlt)
                                                }
                                            },
                                        )
                                        .on_press(
                                            GuiAppMessage::DepTreePage(
                                                DepTreePageMsg::SortOrderToggle,
                                            ),
                                        ),
                                    );

                                let viewer = viewer.push(filters_sort);

                                let graph = dep_tree
                                    .nodes
                                    .iter()
                                    .filter(|(_, asset)| {
                                        self.dep_tree_page_state
                                            .filters
                                            .contains(&(asset.origin, true))
                                    })
                                    .map(|(id, asset)| {
                                        (id, asset, dep_tree.get_node_connections(*id))
                                    })
                                    .sorted_by(|(id1, asset1, cons1), (id2, asset2, cons2)| {
                                        let ordering = match self.dep_tree_page_state.graph_sort {
                                            DepTreePageGraphSort::Id => id1.cmp(id2),
                                            DepTreePageGraphSort::Filename => asset1
                                                .path
                                                .file_name()
                                                .cmp(&asset2.path.file_name()),
                                            DepTreePageGraphSort::Path => {
                                                asset1.path.cmp(&asset2.path)
                                            }
                                            DepTreePageGraphSort::NumDeps => {
                                                cons1.len().cmp(&cons2.len())
                                            }
                                        };

                                        match self.dep_tree_page_state.graph_sort_order {
                                            SortOrder::Ascending => ordering,
                                            SortOrder::Descending => ordering.reverse(),
                                        }
                                    })
                                    .fold(viewer, |viewer, (node_id, asset, node_connections)| {
                                        let viewer = viewer.push(Self::asset_name_text(
                                            false,
                                            *node_id,
                                            asset.clone(),
                                        ));

                                        node_connections
                                            .iter()
                                            .filter_map(|&con_node_id| {
                                                match dep_tree.get_node(con_node_id) {
                                                    Some(con_asset) => match self
                                                        .dep_tree_page_state
                                                        .filters
                                                        .contains(&(con_asset.origin, true))
                                                    {
                                                        true => Some((con_node_id, con_asset)),
                                                        false => None,
                                                    },
                                                    None => None,
                                                }
                                            })
                                            .fold(viewer, |viewer, (con_node_id, con_asset)| {
                                                viewer.push(Self::asset_name_text(
                                                    true,
                                                    con_node_id,
                                                    con_asset,
                                                ))
                                            })
                                            .push(Space::with_height(Length::Units(10)))
                                    });

                                let graph_info = self
                                    .dep_tree_page_state
                                    .footer_asset_show_min_info
                                    .map(|(node_id, extended)| {
                                        let node = dep_tree.get_node(node_id).unwrap();

                                        let dependencies = dep_tree.get_node_connections(node_id);

                                        Scrollable::new(
                                            &mut self
                                                .dep_tree_page_state
                                                .graph_footer_scrollable_state,
                                        )
                                        .push(
                                            Text::new(format!("Full path: {:?}", node.path))
                                                .size(12),
                                        )
                                        .push(
                                            Text::new(format!("Origin: {:?}", node.origin))
                                                .size(12),
                                        )
                                        .push(
                                            Text::new(format!(
                                                "Dependencies: {}{}",
                                                dependencies.len(),
                                                match extended {
                                                    true => format!(
                                                        " ({})",
                                                        dependencies
                                                            .iter()
                                                            .map(|id| path_to_str(
                                                                &dep_tree
                                                                    .get_node(*id)
                                                                    .unwrap()
                                                                    .path
                                                            ))
                                                            .collect_vec()
                                                            .join(", ")
                                                    ),
                                                    false => String::new(),
                                                }
                                            ))
                                            .size(12),
                                        )
                                        .spacing(15)
                                    });

                                (graph, graph_info)
                            }
                            DepTreePageTab::Failures => (
                                dep_tree.failures.iter().fold(viewer, |viewer, failure| {
                                    viewer
                                        .push(Text::new(failure.to_string()).color([0.9, 0.1, 0.1]))
                                }),
                                None,
                            ),
                        },
                        None => (
                            viewer.push(Text::new("No dependency tree generated yet.")),
                            None,
                        ),
                    }
                };

                let mut res = Column::with_children(vec![controls, tab_bar, tab_body.into()])
                    .spacing(10)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .align_items(Alignment::Center);

                if let Some(footer) = footer {
                    res = res.push(footer);
                }

                res.into()
            }
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
    fn asset_name_text<'state>(
        connected: bool,
        node_id: NodeID,
        asset: Rc<Asset>,
    ) -> Element<'state, GuiAppMessage> {
        let name = asset.file_name_str();
        let name_known = name.is_some();

        interactive_text_tooltip(
            format!(
                "{}{} - {}",
                if connected { "└─── " } else { "" },
                node_id,
                name.unwrap_or_else(|| "...Unknown...".to_string())
            ),
            None,
            Some(if !name_known {
                [0.8, 0.2, 0.2]
            } else if connected {
                [0.2, 0.2, 0.8]
            } else {
                [0.2, 0.8, 0.2]
            }),
            (
                Some(GuiAppMessage::DepTreePage(DepTreePageMsg::FooterInfoShow(
                    Some((node_id, false)),
                ))),
                Some(GuiAppMessage::DepTreePage(DepTreePageMsg::FooterInfoShow(
                    Some((node_id, true)),
                ))),
                Some(GuiAppMessage::DepTreePage(DepTreePageMsg::FooterInfoShow(
                    None,
                ))),
            ),
        )
    }
}
