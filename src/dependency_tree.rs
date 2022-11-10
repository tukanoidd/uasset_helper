use std::{
    collections::{HashMap, HashSet},
    path::Path,
    rc::Rc,
    sync::atomic::AtomicU64,
};

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

fn reset_id() {
    NODE_ID_COUNTER.store(0, std::sync::atomic::Ordering::SeqCst)
}

#[allow(dead_code)]
pub struct DepTree {
    pub root_node_id: NodeID,
    pub nodes: HashMap<NodeID, Rc<Asset>>,
    pub node_connections: HashMap<NodeID, Vec<NodeID>>,
    pub failures: Vec<AssetError>,

    pub max_recurse_depth: u32,
    pub recurse_depths: HashMap<NodeID, u32>,
}

impl DepTree {
    pub fn build(
        asset_dirs: &AssetDirs,
        max_recurse_depth: u32,
        pb: Option<&mut ProgressBar>,
    ) -> Result<Self, AssetError> {
        reset_id();

        log::debug!(
            "Building the dependency tree of \"{:?}\"...",
            asset_dirs.asset_file_path.as_ref()
        );

        let mut nodes = HashMap::new();
        let mut node_connections: HashMap<NodeID, Vec<NodeID>> = HashMap::new();
        let mut known_paths = HashSet::new();
        let mut failures = HashSet::new();

        let root_node = Asset::new(asset_dirs.asset_file_path.as_ref().unwrap()).map(Rc::new)?;

        log::debug!("Got the root asset node!");

        let root_node_id = new_id();
        known_paths.insert(root_node.path.clone());
        nodes.insert(root_node_id, root_node);

        // Tracking the depth of the "recursion" of the dependecy chain
        let mut recurse_depths = HashMap::new();
        // We put the original (root) node into the map
        recurse_depths.insert(root_node_id, 0);

        if max_recurse_depth > 0 {
            if let Some(pb) = &pb {
                pb.set_message(format!("Building the network of dependencies recursively with maximum recurse depth of {max_recurse_depth} ..."));
            }

            // List we use to be able to dynamically resolve incoming nodes
            let mut unresolved_nodes_ids = vec![root_node_id];

            // We do iterations as long as there are unresolved ids
            while let Some(cur_node_id) = unresolved_nodes_ids.pop() {
                if let Some(pb) = &pb {
                    pb.set_message(format!("Resolving node with ID {cur_node_id}"));
                }

                // We don't need to resolve current node's dependencies if it is at the maximum depth level
                if *recurse_depths.get(&cur_node_id).unwrap() >= max_recurse_depth {
                    continue;
                }

                // Get the current node
                let cur_node = nodes.get(&cur_node_id).cloned().unwrap();
                // Get current node assets path
                let asset_path = &cur_node.path;

                if let Some(pb) = &pb {
                    pb.set_message(format!("Getting the dependency paths of node with ID {cur_node_id} ({asset_path:?}) ..."));
                }

                // Get current nodes dependencies
                let (dep_paths, fails) = cur_node.get_dependency_asset_paths(
                    asset_dirs.content_dir.as_ref().unwrap(),
                    &asset_dirs.engine_content_dir,
                    &asset_dirs.plugins_dirs,
                );

                // Find all the assets dependency paths that we haven't checked out yet
                let unresolved_deps = dep_paths
                    .into_iter()
                    .filter(|dep_path| {
                        !known_paths.contains(dep_path)
                            && !failures
                                .iter()
                                .any(|fail: &AssetError| &fail.path == dep_path)
                    })
                    .collect::<Vec<_>>();

                // Add new fails to the final list
                failures.extend(fails);

                if let Some(pb) = &pb {
                    pb.set_message(format!(
                        "Creating nodes for {} unresolved paths...",
                        unresolved_deps.len()
                    ));
                }

                // Try to create nodes from the dependencies and split the list in 2, for successes and failures
                let (unresolved_nodes, fails): (Vec<Rc<Asset>>, Vec<AssetError>) =
                    Into::<SplitVecContainer<Rc<Asset>, AssetError>>::into(
                        unresolved_deps
                            .into_iter()
                            // Create an Asset from the dependency path and wrap it in a ref-counted pointer, so we don't waste memory cloning it
                            .map(|dep_path| Asset::new(dep_path).map(Rc::new))
                            // Collect it back to vector
                            .collect::<Vec<_>>(),
                    )
                    .into();

                if let Some(pb) = &pb {
                    pb.set_message("Saving all the failures...");
                }

                // Add new fails to the final list
                failures.extend(fails);

                if let Some(pb) = &pb {
                    pb.set_message("Caching new known paths...");
                }

                // Add new known paths
                known_paths.extend(unresolved_nodes.iter().map(|node| node.path.clone()));

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
            failures: failures.into_iter().collect(),

            max_recurse_depth,
            recurse_depths,
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

    pub fn get_recurse_depth(&self, id: NodeID) -> Option<u32> {
        self.recurse_depths.get(&id).copied()
    }

    pub fn get_parent_node_id(&self, id: NodeID) -> Option<NodeID> {
        self.node_connections
            .iter()
            .find(|(_, children)| children.contains(&id))
            .map(|(parent, _)| *parent)
    }

    pub fn get_parent_node(&self, id: NodeID) -> Option<Rc<Asset>> {
        self.get_parent_node_id(id)
            .and_then(|parent_id| self.get_node(parent_id))
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
