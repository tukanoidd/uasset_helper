use std::{collections::HashMap, path::Path, rc::Rc, sync::atomic::AtomicU64};

use itertools::Itertools;

use crate::{
    asset::{Asset, AssetError},
    util::SplitVecContainer,
};

pub type NodeID = u64;

static NODE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_id() -> NodeID {
    NODE_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Acquire)
}

#[allow(dead_code)]
pub struct DepTree {
    pub root_node_id: NodeID,
    pub nodes: HashMap<NodeID, Rc<Asset>>,
    pub node_connections: HashMap<NodeID, Vec<NodeID>>,
    pub failures: Vec<AssetError>,
}

impl DepTree {
    pub fn build(
        asset_dirs: &AssetDirs,
        max_recurse_depth: u32,
        pb: Option<&mut ProgressBar>,
    ) -> Result<Self, AssetError> {
        log::debug!(
            "Building the dependency tree of \"{:?}\"...",
            asset_dirs.asset_file_path.as_ref()
        );

        let mut nodes = HashMap::new();
        let mut node_connections: HashMap<NodeID, Vec<NodeID>> = HashMap::new();
        let mut known_paths = Vec::new();
        let mut known_failed_paths = Vec::new();
        let mut failures = Vec::new();

        let root_node = Asset::new(asset_dirs.asset_file_path.as_ref().unwrap()).map(Rc::new)?;

        log::debug!("Got the root asset node!");

        let root_node_id = new_id();
        nodes.insert(root_node_id, root_node);

        if max_recurse_depth > 0 {
            if let Some(pb) = &pb {
                pb.set_message(format!("Building the network of dependencies recursively with maximum recurse depth of {max_recurse_depth} ..."));
            }

            let mut unresolved_nodes_ids = vec![root_node_id];

            let mut recurse_depths = HashMap::new();
            recurse_depths.insert(root_node_id, 0);

            while let Some(cur_node_id) = unresolved_nodes_ids.pop() {
                if let Some(pb) = &pb {
                    pb.set_message(format!("Resolving node with ID {cur_node_id}"));
                }

                if recurse_depths.get(&cur_node_id).unwrap() >= &max_recurse_depth {
                    log::warn!("Reached the maximum recurse depth of {max_recurse_depth} for node with ID {cur_node_id}! Skipping...");

                    continue;
                }

                let cur_node = nodes.get(&cur_node_id).cloned().unwrap();
                let asset_path = &cur_node.path;

                if let Some(pb) = &pb {
                    pb.set_message(format!("Getting the dependency paths of node with ID {cur_node_id} ({asset_path:?}) ..."));
                }

                let (dep_paths, mut fails) = cur_node.get_dependency_asset_paths(
                    asset_dirs.content_dir.as_ref().unwrap(),
                    &asset_dirs.engine_content_dir,
                    &asset_dirs.plugins_dirs,
                );

                fails.retain(|fail| !known_failed_paths.contains(&fail.path));
                known_failed_paths.extend(fails.iter().map(|fail| fail.path.clone()));
                failures.extend(fails);

                let unresolved_deps = dep_paths
                    .iter()
                    .filter(|&dep_path| !known_paths.contains(dep_path))
                    .collect::<Vec<_>>();

                if let Some(pb) = &pb {
                    pb.set_message(format!(
                        "Creating nodes for {} unresolved paths...",
                        unresolved_deps.len()
                    ));
                }

                let (unresolved_nodes, mut fails): (Vec<Rc<Asset>>, Vec<AssetError>) =
                    unresolved_deps
                        .into_iter()
                        .map(|dep_path| Asset::new(dep_path).map(Rc::new))
                        .fold(
                            SplitVecContainer::default(),
                            |mut success_failure, asset| {
                                match asset {
                                    Ok(asset) => {
                                        success_failure.push_left(asset);
                                    }
                                    Err(err) => {
                                        success_failure.push_right(err);
                                    }
                                };

                                success_failure
                            },
                        )
                        .into();

                if let Some(pb) = &pb {
                    pb.set_message("Saving all the failures...");
                }

                fails.retain(|fail| !known_failed_paths.contains(&fail.path));
                known_failed_paths.extend(fails.iter().map(|fail| fail.path.clone()));
                failures.extend(fails);

                if let Some(pb) = &pb {
                    pb.set_message("Caching new known paths...");
                }

                known_paths.extend(unresolved_nodes.iter().map(|asset| asset.path.clone()));

                if let Some(pb) = &pb {
                    pb.set_message("Saving the unresolved nodes...");
                }

                nodes.extend(unresolved_nodes.into_iter().map(|asset| {
                    let id = new_id();

                    unresolved_nodes_ids.push(id);
                    node_connections.entry(cur_node_id).or_default().push(id);
                    recurse_depths.insert(id, recurse_depths.get(&cur_node_id).unwrap() + 1);

                    (id, asset)
                }));

                let nodes_amount = nodes.len();

                if let Some(pb) = &pb {
                    pb.set_length(nodes_amount as u64);
                    pb.set_position((nodes_amount - unresolved_nodes_ids.len()) as u64)
                }
            }
        }

        Ok(Self {
            root_node_id,
            nodes,
            node_connections,
            failures,
        })
    }

    pub fn build_with_pb(
        asset_dirs: &AssetDirs,
        max_recurse_depth: u32,
    ) -> color_eyre::Result<Self> {
        let mut pb = ProgressBar::new(1);
        pb.set_style(indicatif::ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {wide_msg}",
        )?);

        let dependency_tree = DepTree::build(asset_dirs, max_recurse_depth, Some(&mut pb))?;

        pb.finish_with_message("Done");

        Ok(dependency_tree)
    }

    #[allow(dead_code)]
    pub fn get_node(&self, id: NodeID) -> Option<Rc<Asset>> {
        self.nodes.get(&id).cloned()
    }

    pub fn get_root_node(&self) -> Rc<Asset> {
        self.nodes.get(&self.root_node_id).cloned().unwrap()
    }

    #[allow(dead_code)]
    pub fn find_node_by_path(&self, path: impl AsRef<Path>) -> Option<Rc<Asset>> {
        self.nodes
            .values()
            .find(|node| node.path == path.as_ref())
            .cloned()
    }

    #[allow(dead_code)]
    pub fn get_node_connections(&self, id: NodeID) -> Vec<NodeID> {
        self.node_connections.get(&id).cloned().unwrap_or_default()
    }

    pub fn print_node_paths(&self) {
        let res = self
            .nodes
            .iter()
            .sorted_by(|(_, asset), (_, asset2)| asset.path.cmp(&asset2.path))
            .fold(
                "\n===== Loaded Asset Paths =====\n".to_string(),
                |res, (node_id, asset)| format!("{}Node {} - {:?}\n", res, &node_id, asset.path),
            )
            + "==============================\n";

        log::info!("{}", res);
    }

    pub fn print_fails(&self) {
        let res = self
            .failures
            .iter()
            .sorted_by(|fail1, fail2| fail1.path.cmp(&fail2.path))
            .fold(
                "\n===== Errors during the building of the dependency tree =====\n".to_string(),
                |res, err| format!("{}{}\n", res, &err.to_string()),
            )
            + "============================================================\n";

        log::error!("{}", res);
    }
}

use std::ffi::OsStr;

use graphviz_rust::dot_structures::Graph;
use indicatif::ProgressBar;

use crate::asset::AssetDirs;

fn fix_file_name(file_name: Option<&OsStr>) -> String {
    file_name
        .unwrap()
        .to_str()
        .unwrap()
        .replace([' ', '-'], "_")
        .split('.')
        .next()
        .unwrap()
        .to_owned()
}

impl From<DepTree> for Graph {
    fn from(value: DepTree) -> Self {
        use graphviz_rust::dot_structures::Id;

        let root_node = value.get_root_node();

        let graph_node_ids = value
            .nodes
            .iter()
            .map(|(&node_id, node)| {
                use graphviz_rust::dot_structures::NodeId;

                (
                    node_id,
                    NodeId(Id::Plain(fix_file_name(node.path.file_name())), None),
                )
            })
            .collect::<HashMap<_, _>>();

        let mut statements = vec![];
        statements.extend(value.nodes.iter().flat_map(|(node_id, _)| {
            use graphviz_rust::dot_structures::{Edge, EdgeTy, Node, Stmt, Vertex};

            let mut sub_statements = vec![];

            let graph_node_id = graph_node_ids.get(node_id).unwrap();

            sub_statements.push(Stmt::Node(Node::new(graph_node_id.clone(), vec![])));

            sub_statements.extend(value.get_node_connections(*node_id).iter().map(
                |connection_node_id| {
                    Stmt::Edge(Edge {
                        ty: EdgeTy::Pair(
                            Vertex::N(graph_node_id.clone()),
                            Vertex::N(graph_node_ids.get(connection_node_id).unwrap().clone()),
                        ),
                        attributes: vec![],
                    })
                },
            ));

            sub_statements
        }));

        Graph::DiGraph {
            id: Id::Plain(format!(
                "Dep_Tree_{}",
                fix_file_name(root_node.path.file_name())
            )),
            strict: true,
            stmts: statements,
        }
    }
}
