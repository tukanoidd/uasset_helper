use std::rc::Rc;

use enum_iterator::{all, Sequence};
use iced::{
    alignment::Horizontal, button, pick_list, scrollable, text_input, Alignment, Button, Checkbox,
    Column, Container, Element, Length, PickList, Row, Scrollable, Text, TextInput,
};
use iced_aw::{graphics::IconText, Icon, TabBar, TabLabel};
use iced_native::widget::Space;
use iced_native::Command;
use itertools::Itertools;
use num_traits::{FromPrimitive, ToPrimitive};
use smart_default::SmartDefault;

use crate::asset::AssetDirs;
use crate::{
    app::interactable_text::interactive_text_tooltip,
    asset::{Asset, AssetOrigin},
    dependency_tree::{DepTree, NodeID},
    util::{path_to_str, save_to_clipboard, SortOrder},
};

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

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Sequence)]
pub enum DepTreePageGraphSortType {
    Id,
    Filename,
    Path,
    NumDeps,
}

impl ToPrimitive for DepTreePageGraphSortType {
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

impl FromPrimitive for DepTreePageGraphSortType {
    fn from_i64(n: i64) -> Option<Self> {
        match n {
            n if n == DepTreePageGraphSortType::Id.to_i64().unwrap() => {
                Some(DepTreePageGraphSortType::Id)
            }
            n if n == DepTreePageGraphSortType::Filename.to_i64().unwrap() => {
                Some(DepTreePageGraphSortType::Filename)
            }
            n if n == DepTreePageGraphSortType::Path.to_i64().unwrap() => {
                Some(DepTreePageGraphSortType::Path)
            }
            n if n == DepTreePageGraphSortType::NumDeps.to_i64().unwrap() => {
                Some(DepTreePageGraphSortType::NumDeps)
            }
            _ => None,
        }
    }

    fn from_usize(n: usize) -> Option<Self> {
        match n {
            n if n == DepTreePageGraphSortType::Id as usize => Some(DepTreePageGraphSortType::Id),
            n if n == DepTreePageGraphSortType::Filename as usize => {
                Some(DepTreePageGraphSortType::Filename)
            }
            n if n == DepTreePageGraphSortType::Path as usize => {
                Some(DepTreePageGraphSortType::Path)
            }
            n if n == DepTreePageGraphSortType::NumDeps as usize => {
                Some(DepTreePageGraphSortType::NumDeps)
            }
            _ => None,
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        match n {
            n if n == DepTreePageGraphSortType::Id.to_u64().unwrap() => {
                Some(DepTreePageGraphSortType::Id)
            }
            n if n == DepTreePageGraphSortType::Filename.to_u64().unwrap() => {
                Some(DepTreePageGraphSortType::Filename)
            }
            n if n == DepTreePageGraphSortType::Path.to_u64().unwrap() => {
                Some(DepTreePageGraphSortType::Path)
            }
            n if n == DepTreePageGraphSortType::NumDeps.to_u64().unwrap() => {
                Some(DepTreePageGraphSortType::NumDeps)
            }
            _ => None,
        }
    }
}

impl ToString for DepTreePageGraphSortType {
    fn to_string(&self) -> String {
        match self {
            DepTreePageGraphSortType::Id => "ID",
            DepTreePageGraphSortType::Filename => "Filename",
            DepTreePageGraphSortType::Path => "Path",
            DepTreePageGraphSortType::NumDeps => "Num Dependencies",
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
    SortChanged(DepTreePageGraphSortType),
    SortOrderToggle,

    FooterInfoShow(Option<(NodeID, bool)>),

    SaveToClipboard(String),
}

#[derive(SmartDefault)]
pub struct DepTreePage {
    pub dep_tree: Option<DepTree>,

    #[default(DepTreePageTab::Graph)]
    pub tab: DepTreePageTab,

    #[default(10)]
    pub max_recurse_depth: u32,

    pub max_recurse_depth_text_input_state: text_input::State,
    pub gen_dep_tree_button_state: button::State,
    pub scrollable_dep_tree_viewer_state: scrollable::State,

    pub sort_pick_list_state: pick_list::State<DepTreePageGraphSortType>,
    pub sort_order_button_state: button::State,

    pub graph_footer_scrollable_state: scrollable::State,

    /// Filters for the graph
    #[default(vec![
    (AssetOrigin::Engine, true),
    (AssetOrigin::Project, true),
    (AssetOrigin::EnginePlugin, true),
    (AssetOrigin::ProjectPlugin, true)
    ])]
    pub filters: Vec<(AssetOrigin, bool)>,
    /// Sorting type of the graph
    #[default(DepTreePageGraphSortType::Id)]
    pub graph_sort_type: DepTreePageGraphSortType,
    /// Sorting order of the graph
    #[default(SortOrder::Ascending)]
    pub graph_sort_order: SortOrder,

    /// ID and if info is extended
    pub footer_asset_show_min_info: Option<(NodeID, bool)>,
}

impl DepTreePage {
    pub fn update<'a, Message>(
        &mut self,
        message: DepTreePageMsg,
        asset_dirs: &AssetDirs,
        clipboard: &mut arboard::Clipboard,
    ) -> Command<Message>
    where
        Message: Clone + 'a,
    {
        match message {
            DepTreePageMsg::GenerateDependencyTree => {
                let dependency_tree = DepTree::build_with_pb(asset_dirs, self.max_recurse_depth);

                match dependency_tree {
                    Ok(dependency_tree) => {
                        self.dep_tree = Some(dependency_tree);
                    }
                    Err(err) => {
                        log::error!("Failed to generate dependency tree: {}", err);
                    }
                }
            }
            DepTreePageMsg::SetMaxRecurseDepth(new_max_recurse_depth) => {
                self.max_recurse_depth = new_max_recurse_depth;
            }
            DepTreePageMsg::TabChanged(new_tab) => {
                self.tab = new_tab;
            }

            DepTreePageMsg::FilterChanged(index, new_checked) => {
                self.filters[index].1 = new_checked;
            }
            DepTreePageMsg::SortChanged(new_sort) => {
                self.graph_sort_type = new_sort;
            }
            DepTreePageMsg::SortOrderToggle => {
                self.graph_sort_order.toggle();
            }
            DepTreePageMsg::FooterInfoShow(new_show) => {
                self.footer_asset_show_min_info = new_show;
            }
            DepTreePageMsg::SaveToClipboard(text) => save_to_clipboard(clipboard, text),
        }

        Command::none()
    }

    pub fn view(&mut self) -> Element<DepTreePageMsg> {
        let controls = Self::controls(
            self.max_recurse_depth,
            &mut self.max_recurse_depth_text_input_state,
            &mut self.gen_dep_tree_button_state,
        )
        .into();

        let (tab_bar, tab_body, footer) = Self::tabs(
            &self.dep_tree,
            self.tab,
            &self.filters,
            self.graph_sort_type,
            self.graph_sort_order,
            self.footer_asset_show_min_info,
            &mut self.scrollable_dep_tree_viewer_state,
            &mut self.sort_pick_list_state,
            &mut self.sort_order_button_state,
            &mut self.graph_footer_scrollable_state,
        );

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

    fn controls<'a>(
        max_recurse_depth: u32,

        max_recurse_depth_text_input_state: &'a mut text_input::State,
        gen_dep_tree_button_state: &'a mut button::State,
    ) -> Row<'a, DepTreePageMsg> {
        let max_recurse_limit = Container::new(
            Row::with_children(Vec::<Element<'a, DepTreePageMsg>>::from([
                Text::new("Max Recursion Depth: ").into(),
                TextInput::new(
                    max_recurse_depth_text_input_state,
                    "Type here...",
                    &max_recurse_depth.to_string(),
                    move |new_number| {
                        DepTreePageMsg::SetMaxRecurseDepth(
                            new_number.parse().unwrap_or(max_recurse_depth),
                        )
                    },
                )
                .width(Length::Units(30))
                .size(20)
                .padding([5, 10])
                .into(),
            ]))
            .spacing(10)
            .align_items(Alignment::End),
        )
        .into();

        let gen_tree_button = Button::new(
            gen_dep_tree_button_state,
            Text::new("Generate Dependency Tree").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Units(200))
        .on_press(DepTreePageMsg::GenerateDependencyTree)
        .into();

        Row::with_children(vec![max_recurse_limit, gen_tree_button])
            .spacing(20)
            .align_items(Alignment::Center)
    }

    #[allow(clippy::too_many_arguments)]
    fn tabs<'a>(
        dep_tree: &'a Option<DepTree>,
        tab: DepTreePageTab,

        filters: &'a [(AssetOrigin, bool)],
        graph_sort_type: DepTreePageGraphSortType,
        graph_sort_order: SortOrder,

        footer_asset_show_min_info: Option<(NodeID, bool)>,

        scrollable_dep_tree_viewer_state: &'a mut scrollable::State,
        sort_pick_list_state: &'a mut pick_list::State<DepTreePageGraphSortType>,
        sort_order_button_state: &'a mut button::State,
        graph_footer_scrollable_state: &'a mut scrollable::State,
    ) -> (
        Element<'a, DepTreePageMsg>,
        Scrollable<'a, DepTreePageMsg>,
        Option<Scrollable<'a, DepTreePageMsg>>,
    ) {
        let tab_bar = Container::new(
            TabBar::width_tab_labels(
                tab as usize,
                all::<DepTreePageTab>()
                    .map(|tab_val| TabLabel::Text(tab_val.to_string()))
                    .collect(),
                |tab| DepTreePageMsg::TabChanged(DepTreePageTab::from_usize(tab).unwrap()),
            )
            .spacing(20)
            .tab_width(Length::Units(300)),
        )
        .align_x(Horizontal::Center)
        .into();

        let (tab_body, footer) = {
            let viewer = Scrollable::new(scrollable_dep_tree_viewer_state)
                .height(Length::Fill)
                .spacing(10)
                .scrollbar_width(15)
                .scroller_width(15);

            match dep_tree {
                Some(dep_tree) => match tab {
                    DepTreePageTab::Graph => Self::graph_tab(
                        viewer,
                        dep_tree,
                        filters,
                        graph_sort_type,
                        graph_sort_order,
                        footer_asset_show_min_info,
                        sort_pick_list_state,
                        sort_order_button_state,
                        graph_footer_scrollable_state,
                    ),
                    DepTreePageTab::Failures => Self::failures_tab(viewer, dep_tree),
                },
                None => (
                    viewer.push(Text::new("No dependency tree generated yet.")),
                    None,
                ),
            }
        };

        (tab_bar, tab_body, footer)
    }

    #[allow(clippy::too_many_arguments)]
    fn graph_tab<'a>(
        viewer: Scrollable<'a, DepTreePageMsg>,
        dep_tree: &'a DepTree,

        filters: &'a [(AssetOrigin, bool)],
        graph_sort_type: DepTreePageGraphSortType,
        graph_sort_order: SortOrder,

        footer_asset_show_min_info: Option<(NodeID, bool)>,

        sort_pick_list_state: &'a mut pick_list::State<DepTreePageGraphSortType>,
        sort_order_button_state: &'a mut button::State,
        graph_footer_scrollable_state: &'a mut scrollable::State,
    ) -> (
        Scrollable<'a, DepTreePageMsg>,
        Option<Scrollable<'a, DepTreePageMsg>>,
    ) {
        let filters_row = Row::with_children(
            filters
                .iter()
                .enumerate()
                .map(|(index, (filter, on))| {
                    Checkbox::new(*on, filter.to_string(), move |new_checked| {
                        DepTreePageMsg::FilterChanged(index, new_checked)
                    })
                    .into()
                })
                .collect(),
        )
        .spacing(20);

        let filters_sort = filters_row
            .push(Space::with_width(Length::Units(15)))
            .push(
                PickList::new(
                    sort_pick_list_state,
                    all::<DepTreePageGraphSortType>().collect_vec(),
                    Some(graph_sort_type),
                    DepTreePageMsg::SortChanged,
                )
                .width(Length::Shrink)
                .padding([5, 10])
                .text_size(16),
            )
            .push(
                Button::new(
                    sort_order_button_state,
                    match graph_sort_order {
                        SortOrder::Ascending => IconText::new(Icon::SortDownAlt),
                        SortOrder::Descending => IconText::new(Icon::SortUpAlt),
                    },
                )
                .on_press(DepTreePageMsg::SortOrderToggle),
            );

        let viewer = viewer.push(filters_sort);

        let graph = dep_tree
            .nodes
            .iter()
            .filter(|(_, asset)| filters.contains(&(asset.origin, true)))
            .map(|(id, asset)| (id, asset, dep_tree.get_node_connections(*id)))
            .sorted_by(|(id1, asset1, cons1), (id2, asset2, cons2)| {
                let ordering = match graph_sort_type {
                    DepTreePageGraphSortType::Id => id1.cmp(id2),
                    DepTreePageGraphSortType::Filename => {
                        asset1.path.file_name().cmp(&asset2.path.file_name())
                    }
                    DepTreePageGraphSortType::Path => asset1.path.cmp(&asset2.path),
                    DepTreePageGraphSortType::NumDeps => cons1.len().cmp(&cons2.len()),
                };

                match graph_sort_order {
                    SortOrder::Ascending => ordering,
                    SortOrder::Descending => ordering.reverse(),
                }
            })
            .fold(viewer, |viewer, (node_id, asset, node_connections)| {
                let viewer = viewer.push(Self::asset_name_text(false, *node_id, asset.clone()));

                node_connections
                    .iter()
                    .filter_map(|&con_node_id| match dep_tree.get_node(con_node_id) {
                        Some(con_asset) => match filters.contains(&(con_asset.origin, true)) {
                            true => Some((con_node_id, con_asset)),
                            false => None,
                        },
                        None => None,
                    })
                    .fold(viewer, |viewer, (con_node_id, con_asset)| {
                        viewer.push(Self::asset_name_text(true, con_node_id, con_asset))
                    })
                    .push(Space::with_height(Length::Units(10)))
            });

        let graph_info = footer_asset_show_min_info.map(|(node_id, extended)| {
            let node = dep_tree.get_node(node_id).unwrap();

            let dependencies = dep_tree.get_node_connections(node_id);

            Scrollable::new(graph_footer_scrollable_state)
                .push(Text::new(format!("Full path: {:?}", node.path)).size(12))
                .push(Text::new(format!("Origin: {:?}", node.origin)).size(12))
                .push(
                    Text::new(format!(
                        "Dependencies: {}{}",
                        dependencies.len(),
                        match extended {
                            true => format!(
                                " ({})",
                                dependencies
                                    .iter()
                                    .map(|id| path_to_str(&dep_tree.get_node(*id).unwrap().path))
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

    fn failures_tab<'a>(
        viewer: Scrollable<'a, DepTreePageMsg>,
        dep_tree: &'a DepTree,
    ) -> (
        Scrollable<'a, DepTreePageMsg>,
        Option<Scrollable<'a, DepTreePageMsg>>,
    ) {
        (
            dep_tree.failures.iter().fold(viewer, |viewer, failure| {
                viewer.push(Text::new(failure.to_string()).color([0.9, 0.1, 0.1]))
            }),
            None,
        )
    }

    fn asset_name_text<'state>(
        connected: bool,
        node_id: NodeID,
        asset: Rc<Asset>,
    ) -> Element<'state, DepTreePageMsg> {
        let name = asset.file_name_str();
        let name_known = name.is_some();

        let text = format!(
            "{}{} - {}",
            if connected { "└─── " } else { "" },
            node_id,
            name.unwrap_or_else(|| "...Unknown...".to_string())
        );

        interactive_text_tooltip(
            &text,
            None,
            Some(if !name_known {
                [0.8, 0.2, 0.2]
            } else if connected {
                [0.2, 0.2, 0.8]
            } else {
                [0.2, 0.8, 0.2]
            }),
            (Some(DepTreePageMsg::SaveToClipboard(text.clone())), None),
            (
                Some(DepTreePageMsg::FooterInfoShow(Some((node_id, false)))),
                Some(DepTreePageMsg::FooterInfoShow(Some((node_id, true)))),
                Some(DepTreePageMsg::FooterInfoShow(None)),
            ),
        )
    }
}