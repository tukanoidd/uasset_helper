use std::borrow::Cow;
use std::rc::Rc;

use enum_iterator::{all, Sequence};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        column, Button, Checkbox, Column, Container, PickList, Row, Scrollable, Space, Text,
        TextInput,
    },
    Alignment, Color, Command, Element, Length,
};
use iced_aw::{graphics::IconText, Icon, TabBar, TabLabel};
use iced_native::row;
use itertools::Itertools;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::{
    app::interactable_text::interactive_text_tooltip,
    asset::{Asset, AssetDirs, AssetOrigin},
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
    ClearDependencyTree,
    SetMaxRecurseDepth(Option<u32>),
    SetTab(DepTreePageTab),

    SetMinGraphDepth(Option<u32>),

    SetFilter(usize, bool),
    SetSortType(DepTreePageGraphSortType),
    ToggleSortOrder,
    SetShowChildren(bool),
    SetShowOnlyChanged(bool),

    ShowFooterInfo(Option<(NodeID, bool)>),

    SaveToClipboard(String),
}

pub struct DepTreePage {
    pub asset_dirs: AssetDirs,

    pub dep_tree: Option<DepTree>,

    pub tab: DepTreePageTab,

    pub max_recurse_depth: u32,
    pub max_recurse_depth_text: String,

    pub min_graph_depth: u32,
    pub min_graph_depth_text: String,

    /// Filters for the graph
    pub filters: Vec<(AssetOrigin, bool)>,
    /// Sorting type of the graph
    pub graph_sort_type: DepTreePageGraphSortType,
    /// Sorting order of the graph
    pub graph_sort_order: SortOrder,
    pub graph_show_children: bool,
    pub graph_show_only_changed: bool,

    /// ID and if info is extended
    pub footer_asset_show_min_info: Option<(NodeID, bool)>,
}

impl DepTreePage {
    pub fn new(asset_dirs: AssetDirs) -> Self {
        Self {
            asset_dirs,

            dep_tree: None,

            tab: DepTreePageTab::Graph,

            max_recurse_depth: 10,
            max_recurse_depth_text: String::from("10"),

            min_graph_depth: 0,
            min_graph_depth_text: String::from("0"),

            filters: vec![
                (AssetOrigin::Engine, true),
                (AssetOrigin::EnginePlugin, true),
                (AssetOrigin::Project, true),
                (AssetOrigin::ProjectPlugin, true),
            ],
            graph_sort_type: DepTreePageGraphSortType::Id,
            graph_sort_order: SortOrder::Ascending,
            graph_show_children: true,
            graph_show_only_changed: false,

            footer_asset_show_min_info: None,
        }
    }

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
                        if dependency_tree.nodes.len() > 1000 {
                            self.graph_show_children = false;
                        }

                        self.dep_tree = Some(dependency_tree);
                    }
                    Err(err) => {
                        log::error!("Failed to generate dependency tree: {}", err);
                    }
                }
            }
            DepTreePageMsg::ClearDependencyTree => {
                self.dep_tree = None;
            }
            DepTreePageMsg::SetMaxRecurseDepth(new_max_recurse_depth) => {
                match new_max_recurse_depth {
                    Some(new_depth) => {
                        self.max_recurse_depth = new_depth;
                        self.max_recurse_depth_text = new_depth.to_string();
                    }
                    None => self.max_recurse_depth_text = String::new(),
                }
            }
            DepTreePageMsg::SetTab(new_tab) => {
                self.tab = new_tab;
            }
            DepTreePageMsg::SetFilter(index, new_checked) => {
                self.filters[index].1 = new_checked;
            }
            DepTreePageMsg::SetSortType(new_sort) => {
                self.graph_sort_type = new_sort;
            }
            DepTreePageMsg::ToggleSortOrder => {
                self.graph_sort_order.toggle();
            }
            DepTreePageMsg::ShowFooterInfo(new_footer_info) => {
                self.footer_asset_show_min_info = new_footer_info;
            }
            DepTreePageMsg::SaveToClipboard(text) => save_to_clipboard(clipboard, text),
            DepTreePageMsg::SetMinGraphDepth(new_min_graph_depth) => match new_min_graph_depth {
                Some(new_depth) => {
                    self.min_graph_depth = new_depth;
                    self.min_graph_depth_text = new_depth.to_string();
                }
                None => self.min_graph_depth_text = String::new(),
            },
            DepTreePageMsg::SetShowChildren(new_graph_show_children) => {
                self.graph_show_children = new_graph_show_children
            }
            DepTreePageMsg::SetShowOnlyChanged(new_graph_show_only_changed) => {
                self.graph_show_only_changed = new_graph_show_only_changed;
            }
        }

        Command::none()
    }

    pub fn view(&self) -> Element<DepTreePageMsg> {
        let controls = Self::controls(self.dep_tree.is_some(), &self.max_recurse_depth_text).into();

        let (tab_bar, tab_body, footer) = Self::tabs(
            &self.asset_dirs,
            &self.dep_tree,
            self.tab,
            self.min_graph_depth,
            &self.min_graph_depth_text,
            &self.filters,
            self.graph_sort_type,
            self.graph_sort_order,
            self.graph_show_children,
            self.graph_show_only_changed,
            self.footer_asset_show_min_info,
        );

        let mut res = Column::with_children(vec![controls, tab_bar, tab_body])
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
        dep_tree_exists: bool,
        max_recurse_depth_text: &str,
    ) -> Row<'a, DepTreePageMsg> {
        let max_recurse_limit = Self::text_with_input(
            "Max Recursion Depth: ",
            max_recurse_depth_text,
            move |new_number| {
                DepTreePageMsg::SetMaxRecurseDepth({
                    Self::only_numeric_chars(&new_number).parse().ok()
                })
            },
        )
        .into();

        let gen_tree_button =
            Button::new(Text::new("Generate").horizontal_alignment(Horizontal::Center))
                .width(Length::Units(150))
                .on_press(DepTreePageMsg::GenerateDependencyTree)
                .into();

        let mut widgets = vec![max_recurse_limit, gen_tree_button];

        if dep_tree_exists {
            let clear_tree_button =
                Button::new(Text::new("Clear").horizontal_alignment(Horizontal::Center))
                    .width(Length::Units(150))
                    .on_press(DepTreePageMsg::ClearDependencyTree)
                    .into();

            widgets.push(clear_tree_button);
        }

        Row::with_children(widgets)
            .spacing(20)
            .align_items(Alignment::Center)
    }

    fn text_with_input<'a>(
        text: impl Into<Cow<'a, str>>,
        value: &str,
        on_input: impl Fn(String) -> DepTreePageMsg + 'a,
    ) -> Container<'a, DepTreePageMsg> {
        let container = Container::new({
            row![
                Text::new(text)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Right),
                TextInput::new("Type here...", value, on_input)
                    .width(Length::Units(30))
                    .size(15)
                    .padding([5, 10]),
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        })
        .align_y(Vertical::Center);

        container
    }

    fn only_numeric_chars(str: &str) -> String {
        str.chars()
            .filter(|char| char.is_numeric())
            .collect::<String>()
    }

    #[allow(clippy::too_many_arguments)]
    fn tabs<'a>(
        asset_dirs: &AssetDirs,

        dep_tree: &'a Option<DepTree>,
        tab: DepTreePageTab,

        min_graph_depth: u32,
        min_graph_depth_text: &str,

        filters: &'a [(AssetOrigin, bool)],
        graph_sort_type: DepTreePageGraphSortType,
        graph_sort_order: SortOrder,
        graph_show_children: bool,
        graph_show_only_changed: bool,

        footer_asset_show_min_info: Option<(NodeID, bool)>,
    ) -> (
        Element<'a, DepTreePageMsg>,
        Element<'a, DepTreePageMsg>,
        Option<Element<'a, DepTreePageMsg>>,
    ) {
        let tab_bar = Container::new(
            TabBar::width_tab_labels(
                tab as usize,
                all::<DepTreePageTab>()
                    .map(|tab_val| TabLabel::Text(tab_val.to_string()))
                    .collect(),
                |tab| DepTreePageMsg::SetTab(DepTreePageTab::from_usize(tab).unwrap()),
            )
            .spacing(20)
            .tab_width(Length::Units(300)),
        )
        .align_x(Horizontal::Center)
        .into();

        let (tab_body, footer) = {
            let viewer = Column::new();

            let (tab_body, footer) = match dep_tree {
                Some(dep_tree) => match tab {
                    DepTreePageTab::Graph => {
                        let (min_depth_filters_sort, graph, nodes_count, graph_info) =
                            Self::graph_tab(
                                asset_dirs,
                                dep_tree,
                                min_graph_depth,
                                min_graph_depth_text,
                                filters,
                                graph_sort_type,
                                graph_sort_order,
                                graph_show_children,
                                graph_show_only_changed,
                                footer_asset_show_min_info,
                            );

                        let body = Column::with_children(Vec::from([
                            min_depth_filters_sort,
                            Space::with_height(Length::Units(10)).into(),
                            Text::new(format!("Nodes: {}", nodes_count))
                                .horizontal_alignment(Horizontal::Center)
                                .into(),
                            Space::with_height(Length::Units(10)).into(),
                            graph,
                        ]))
                        .align_items(Alignment::Center)
                        .into();

                        (body, graph_info)
                    }
                    DepTreePageTab::Failures => Self::failures_tab(viewer, dep_tree),
                },
                None => (
                    viewer
                        .push(Text::new("No dependency tree generated yet."))
                        .into(),
                    None,
                ),
            };

            (tab_body, footer)
        };

        (tab_bar, tab_body, footer)
    }

    #[allow(clippy::too_many_arguments)]
    fn graph_tab<'a>(
        asset_dirs: &AssetDirs,

        dep_tree: &'a DepTree,
        min_graph_depth: u32,
        min_graph_depth_text: &str,

        filters: &'a [(AssetOrigin, bool)],
        graph_sort_type: DepTreePageGraphSortType,
        graph_sort_order: SortOrder,
        graph_show_children: bool,
        graph_show_only_changed: bool,

        footer_asset_show_min_info: Option<(NodeID, bool)>,
    ) -> (
        Element<'a, DepTreePageMsg>,
        Element<'a, DepTreePageMsg>,
        usize,
        Option<Element<'a, DepTreePageMsg>>,
    ) {
        let mut show_only_changed_show_children_min_depth_filters_sort =
            Row::new().spacing(15).align_items(Alignment::Center);

        let mut show_only_changed_show_children_min_depth =
            Vec::<Element<'a, DepTreePageMsg>>::from([
                Checkbox::new(
                    graph_show_children,
                    "Show Node Children",
                    DepTreePageMsg::SetShowChildren,
                )
                .spacing(5)
                .into(),
                Self::text_with_input(
                    "Min Graph Depth:",
                    min_graph_depth_text,
                    move |new_number| {
                        DepTreePageMsg::SetMinGraphDepth(
                            Self::only_numeric_chars(&new_number)
                                .parse()
                                .ok()
                                .map(|num: u32| num.clamp(0, dep_tree.max_recurse_depth)),
                        )
                    },
                )
                .into(),
            ]);

        if asset_dirs.project_git_repo.is_some() || asset_dirs.project_git_repo.is_some() {
            show_only_changed_show_children_min_depth.insert(
                0,
                Checkbox::new(
                    graph_show_only_changed,
                    "Show Only Modified File Nodes (from Git Repo)",
                    DepTreePageMsg::SetShowOnlyChanged,
                )
                .spacing(5)
                .into(),
            );
        }

        show_only_changed_show_children_min_depth_filters_sort =
            show_only_changed_show_children_min_depth_filters_sort
                .push(Column::with_children(
                    show_only_changed_show_children_min_depth,
                ))
                .push(Space::with_width(Length::Units(5)));

        let columns = filters
            .iter()
            .cloned()
            .enumerate()
            .chunks(2)
            .into_iter()
            .map(|chunk| chunk.collect::<Vec<(usize, (AssetOrigin, bool))>>())
            .collect_vec();

        let min_depth_filters = columns.into_iter().fold(
            Row::new().spacing(10).align_items(Alignment::Center),
            move |row, column| {
                row.push(
                    Column::with_children(
                        column
                            .into_iter()
                            .map(move |(index, (filter, on))| {
                                Checkbox::new(on, filter.to_string(), move |new_checked| {
                                    DepTreePageMsg::SetFilter(index, new_checked)
                                })
                                .spacing(5)
                                .into()
                            })
                            .collect(),
                    )
                    .spacing(5),
                )
            },
        );

        let show_only_changed_show_children_min_depth_filters_sort =
            show_only_changed_show_children_min_depth_filters_sort
                .push(min_depth_filters)
                .push(Space::with_width(Length::Units(5)))
                .push(
                    PickList::new(
                        all::<DepTreePageGraphSortType>().collect_vec(),
                        Some(graph_sort_type),
                        DepTreePageMsg::SetSortType,
                    )
                    .width(Length::Shrink)
                    .padding([5, 10])
                    .text_size(16),
                )
                .push(
                    Button::new(match graph_sort_order {
                        SortOrder::Ascending => IconText::new(Icon::SortDownAlt),
                        SortOrder::Descending => IconText::new(Icon::SortUpAlt),
                    })
                    .on_press(DepTreePageMsg::ToggleSortOrder),
                )
                .into();

        let nodes = dep_tree
            .nodes
            .iter()
            .filter_map(|(&node_id, asset)| {
                let main_check = dep_tree.get_recurse_depth(node_id).unwrap_or_default()
                    >= min_graph_depth
                    && filters.contains(&(asset.origin, true));

                if !main_check {
                    None
                } else {
                    let has_changed_in_git_repo = asset_dirs
                        .get_git_repo(asset.origin)
                        .as_ref()
                        .and_then(|repo| {
                            asset_dirs
                                .get_relative_path(asset)
                                .and_then(|relative_path| repo.status_file(&relative_path).ok())
                        })
                        .map(|status| status.is_index_modified() || status.is_wt_modified())
                        .unwrap_or_default();

                    match graph_show_only_changed {
                        true => match has_changed_in_git_repo {
                            true => Some((node_id, asset, true)),
                            false => None,
                        },
                        false => Some((node_id, asset, has_changed_in_git_repo)),
                    }
                }
            })
            .map(|(id, asset, has_changed)| {
                (id, asset, has_changed, dep_tree.get_node_connections(id))
            })
            .sorted_by(|(id1, asset1, _, cons1), (id2, asset2, _, cons2)| {
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
            .collect_vec();

        let nodes_count = nodes.len();

        let graph = nodes.into_iter().fold(
            vec![],
            |mut graph, (node_id, asset, has_changed, node_connections)| {
                graph.push(Self::asset_name_text(
                    false,
                    node_id,
                    asset.clone(),
                    has_changed,
                ));

                match graph_show_children {
                    true => {
                        let mut graph = node_connections
                            .iter()
                            .filter_map(|&con_node_id| {
                                dep_tree.get_node(con_node_id).and_then(|con_node| {
                                    let main_check = filters.contains(&(con_node.origin, true));

                                    if !main_check {
                                        None
                                    } else {
                                        let has_changed_in_git_repo = asset_dirs
                                            .get_git_repo(con_node.origin)
                                            .as_ref()
                                            .and_then(|repo| {
                                                asset_dirs.get_relative_path(&con_node).and_then(
                                                    |relative_path| {
                                                        repo.status_file(&relative_path).ok()
                                                    },
                                                )
                                            })
                                            .map(|status| {
                                                status.is_index_modified()
                                                    || status.is_wt_modified()
                                            })
                                            .unwrap_or_default();

                                        match graph_show_only_changed {
                                            true => match has_changed_in_git_repo {
                                                true => Some((node_id, con_node, true)),
                                                false => None,
                                            },
                                            false => {
                                                Some((node_id, con_node, has_changed_in_git_repo))
                                            }
                                        }
                                    }
                                })
                            })
                            .fold(graph, |mut graph, (con_node_id, con_asset, has_changed)| {
                                graph.push(Self::asset_name_text(
                                    true,
                                    con_node_id,
                                    con_asset,
                                    has_changed,
                                ));

                                graph
                            });

                        graph.push(Space::with_height(Length::Units(10)).into());

                        graph
                    }
                    false => graph,
                }
            },
        );

        let graph = Scrollable::new(Column::with_children(graph)).into();

        let graph_info = footer_asset_show_min_info.and_then(|(node_id, extended)| {
            let Some(node) = dep_tree.get_node(node_id) else {
                return None;
            };

            let dependencies = dep_tree.get_node_connections(node_id);

            Some(
                Scrollable::new(
                    column![
                        Text::new(format!("Full path: {:?}", node.path)).size(14),
                        Text::new(format!("Origin: {:?}", node.origin)).size(14),
                        Text::new(format!(
                            "Parent Node: {}",
                            dep_tree
                                .get_parent_node_id(node_id)
                                .and_then(|parent_node_id| dep_tree.get_node(parent_node_id).map(
                                    |parent_node| format!(
                                        "{} - {}",
                                        parent_node_id,
                                        parent_node
                                            .file_name_str()
                                            .unwrap_or_else(|| "...Unknown...".to_string())
                                    )
                                ))
                                .unwrap_or_else(|| "None".to_string())
                        ))
                        .size(14),
                        Text::new(format!(
                            "Dependencies: {}{}",
                            dependencies.len(),
                            match extended {
                                true => format!(
                                    " ({})",
                                    dependencies
                                        .iter()
                                        .map(|id| path_to_str(
                                            &dep_tree.get_node(*id).unwrap().path
                                        ))
                                        .collect_vec()
                                        .join(", ")
                                ),
                                false => String::new(),
                            }
                        ))
                        .size(14),
                    ]
                    .spacing(15),
                )
                .height(Length::Units(100))
                .into(),
            )
        });

        (
            show_only_changed_show_children_min_depth_filters_sort,
            graph,
            nodes_count,
            graph_info,
        )
    }

    fn failures_tab<'a>(
        viewer: Column<'a, DepTreePageMsg>,
        dep_tree: &'a DepTree,
    ) -> (
        Element<'a, DepTreePageMsg>,
        Option<Element<'a, DepTreePageMsg>>,
    ) {
        (
            dep_tree
                .failures
                .iter()
                .fold(viewer, |viewer, failure| {
                    viewer.push(Text::new(failure.to_string()).style(Color::from([0.9, 0.1, 0.1])))
                })
                .into(),
            None,
        )
    }

    fn asset_name_text<'state>(
        connected: bool,
        node_id: NodeID,
        asset: Rc<Asset>,

        has_changed_in_git_repo: bool,
    ) -> Element<'state, DepTreePageMsg> {
        let name = asset.file_name_str();
        let name_known = name.is_some();

        let text = format!(
            "{}{} - {}",
            if connected { "└─── " } else { "" },
            node_id,
            name.clone().unwrap_or_else(|| "...Unknown...".to_string())
        );

        interactive_text_tooltip::<DepTreePageMsg>(
            text.clone(),
            None,
            Some(if has_changed_in_git_repo {
                [0.75, 0.75, 0.15]
            } else if !name_known {
                [0.8, 0.2, 0.2]
            } else if connected {
                [0.2, 0.2, 0.8]
            } else {
                [0.2, 0.8, 0.2]
            }),
            (
                Some(DepTreePageMsg::SaveToClipboard(
                    name.clone().unwrap_or_else(|| text.clone()),
                )),
                Some(DepTreePageMsg::SaveToClipboard(asset.path_str())),
                name.map(|file_name_str| {
                    DepTreePageMsg::SaveToClipboard(
                        file_name_str
                            .strip_suffix(".uasset")
                            .map(String::from)
                            .unwrap_or(file_name_str),
                    )
                }),
            ),
            (
                Some(DepTreePageMsg::ShowFooterInfo(Some((node_id, false)))),
                Some(DepTreePageMsg::ShowFooterInfo(Some((node_id, true)))),
                Some(DepTreePageMsg::ShowFooterInfo(None)),
            ),
        )
    }
}
