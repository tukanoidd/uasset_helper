use std::{
    collections::HashSet,
    ffi::OsString,
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
};

use uasset::{AssetHeader, ImportIterator};

use crate::util::{path_to_str, SplitVecContainer};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetError {
    pub path: PathBuf,
    pub reason: String,
}

impl Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to read asset ({:?}). Reason: {}",
            self.path, self.reason
        )
    }
}

impl AssetError {
    pub fn new(path: impl AsRef<Path>, reason: impl Into<String>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            reason: reason.into(),
        }
    }
}

impl std::error::Error for AssetError {}

#[derive(Debug)]
pub struct Asset {
    pub package: AssetHeader<File>,
    pub path: PathBuf,
}

impl Asset {
    pub fn new(asset_path: impl AsRef<Path>) -> Result<Self, AssetError> {
        let package = Self::read_asset(&asset_path)?;

        Ok(Self {
            package,
            path: asset_path.as_ref().to_path_buf(),
        })
    }

    pub fn path(&self) -> OsString {
        self.path.clone().into_os_string()
    }

    pub fn path_str(&self) -> String {
        self.path().to_str().unwrap().to_string()
    }

    pub fn file_name(&self) -> Option<OsString> {
        self.path.file_name().map(|s| s.to_os_string())
    }

    pub fn file_name_str(&self) -> Option<String> {
        self.file_name().map(|s| s.to_str().unwrap().to_string())
    }

    #[inline]
    pub fn get_dependency_names(&self) -> ImportIterator<File> {
        self.package.package_import_iter()
    }

    pub fn get_dependency_asset_paths(
        &self,
        content_dir: impl AsRef<Path>,
        engine_content_dir: &Option<impl AsRef<Path>>,
        plugins_dirs: &[impl AsRef<Path>],
    ) -> (Vec<PathBuf>, Vec<AssetError>) {
        let result: SplitVecContainer<PathBuf, AssetError> = self.get_dependency_names().fold(
            SplitVecContainer::default(),
            |mut result_container, dependency_name| {
                let dep = if !dependency_name.ends_with(".uasset") {
                    dependency_name + ".uasset"
                } else {
                    dependency_name
                };
                let segments: Vec<_> = dep[1..].split('/').collect();

                let Some(&root_folder) = segments.first() else {
                    log::debug!("Error: Couldn't get the root folder of the path");

                    return result_container;
                };

                let asset_path = match root_folder {
                    "Game" => {
                        let path = content_dir.as_ref().join(segments[1..].join("/"));

                        match path.exists() {
                            true => Ok(path),
                            false => {
                                Err(AssetError::new(
                                    path,
                                    "The asset doesn't exist in the game content directory",
                                ))
                            }
                        }
                    },
                    "Engine" => {
                        match engine_content_dir {
                            Some(engine_content_dir) => {
                                let path= engine_content_dir.as_ref().join(segments[1..].join("/"));

                                match path.exists() {
                                    true => Ok(path),
                                    false => Err(AssetError::new(path, "The asset doesn't exist in the engine content directory")),
                                }
                            },
                            None => {
                                Err(AssetError::new(&dep, "Engine content directory is not set!"))
                            }
                        }
                    },
                    "Script" => {
                        Err(AssetError::new(&dep, "Need to figure out what this folder is for yet, cuz I can't seem to find much info about it online and can't find files on my drive"))
                    },
                    root_dir => {
                        let candidate_dirs = plugins_dirs.iter().map(|plugins_dir| {
                            walkdir::WalkDir::new(plugins_dir.as_ref()).max_depth(10).into_iter().flat_map(|entry| {
                                entry.ok().and_then(|entry| {
                                    if entry.file_name() == root_dir {
                                        let content_dir = entry.path().join("Content");

                                        if content_dir.exists() {
                                            Some(content_dir)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                            })
                        }).fold(HashSet::new(), |mut acc, candidate_dirs| {
                            acc.extend(candidate_dirs);
                            acc
                        });

                        match candidate_dirs.iter().find_map(|candidate_dir| {
                            let path = candidate_dir.join(segments[1..].join("/"));

                            if path.exists() {
                                Some(path)
                            } else {
                                None
                            }
                        }) {
                            Some(file_path) => {
                                Ok(file_path)
                            },
                            None => {
                                Err(AssetError::new(&dep, "Couldn't find the asset in any of the plugins directories".to_string()))
                            }
                        }
                    }
                };

                match asset_path {
                    Ok(asset_path) => {
                        result_container.push_left(asset_path);
                    }
                    Err(err) => {
                        result_container.push_right(err);
                    }
                };

                result_container
            },
        );

        log::debug!(
            "Successfully got asset paths: {}, Failed: {}",
            result.left.len(),
            result.right.len()
        );

        result.into()
    }

    pub fn read_asset(asset_path: impl AsRef<Path>) -> Result<AssetHeader<File>, AssetError> {
        if asset_path.as_ref().exists() {
            let file = File::open(asset_path.as_ref())
                .map_err(|err| AssetError::new(asset_path.as_ref(), err.to_string()))?;
            let asset_header = AssetHeader::new(file).map_err(|err| {
                AssetError::new(
                    asset_path.as_ref(),
                    format!("Failed to read asset: {}", err),
                )
            })?;

            Ok(asset_header)
        } else {
            Err(AssetError::new(asset_path.as_ref(), "Could not find asset"))
        }
    }
}

#[derive(Debug)]
pub struct AssetDirs {
    pub asset_file_path: Option<PathBuf>,
    pub game_root: Option<PathBuf>,
    pub content_dir: Option<PathBuf>,
    pub engine_dir: Option<PathBuf>,
    pub engine_content_dir: Option<PathBuf>,
    pub plugins_dirs: Vec<PathBuf>,
}

impl AssetDirs {
    pub fn new(asset_file_path: Option<PathBuf>, engine_dir: Option<PathBuf>) -> Self {
        let (game_root, content_dir) = Self::get_game_dirs(&asset_file_path);
        let (engine_dir, engine_content_dir) = Self::get_engine_dirs(&engine_dir);
        let plugins_dirs = Self::get_plugins_dirs(&game_root, &engine_dir);

        Self {
            asset_file_path,
            game_root,
            content_dir,
            engine_dir,
            engine_content_dir,
            plugins_dirs,
        }
    }

    pub fn asset_file_name(&self) -> Option<OsString> {
        self.asset_file_path
            .as_ref()
            .map(|asset_file_path| asset_file_path.file_name().unwrap().to_os_string())
    }

    pub fn asset_file_name_str(&self) -> Option<String> {
        self.asset_file_name()
            .map(|asset_file_name| asset_file_name.into_string().unwrap())
    }

    pub fn asset_file_path_str(&self) -> Option<String> {
        self.asset_file_path.as_ref().map(path_to_str)
    }

    pub fn game_root_str(&self) -> Option<String> {
        self.game_root.as_ref().map(path_to_str)
    }

    pub fn content_dir_str(&self) -> Option<String> {
        self.content_dir.as_ref().map(path_to_str)
    }

    pub fn engine_dir_str(&self) -> Option<String> {
        self.engine_dir.as_ref().map(path_to_str)
    }

    pub fn engine_content_dir_str(&self) -> Option<String> {
        self.engine_content_dir.as_ref().map(path_to_str)
    }

    pub fn plugins_dirs_str(&self) -> Vec<String> {
        self.plugins_dirs.iter().map(path_to_str).collect()
    }

    pub fn get_game_dirs(asset_file_path: &Option<PathBuf>) -> (Option<PathBuf>, Option<PathBuf>) {
        let game_root = asset_file_path
            .as_ref()
            .map(|asset_file_path| {
                asset_file_path
                    .iter()
                    .map(|x| x.to_str().unwrap())
                    .take_while(|x| x != &"Content")
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .map(|game_root| {
                game_root
                    .strip_prefix('/')
                    .unwrap_or(&game_root)
                    .to_string()
            })
            .map(PathBuf::from);
        let content_dir = game_root
            .as_ref()
            .map(|game_root| game_root.join("Content"));

        (game_root, content_dir)
    }

    pub fn get_engine_dirs(engine_dir: &Option<PathBuf>) -> (Option<PathBuf>, Option<PathBuf>) {
        let engine_dir = engine_dir.as_ref().map(|dir| {
            if dir.ends_with("Engine") {
                dir.clone()
            } else {
                dir.join("Engine")
            }
        });
        let engine_content_dir = engine_dir
            .clone()
            .map(|engine_dir| engine_dir.join("Content"));

        (engine_dir, engine_content_dir)
    }

    pub fn get_plugins_dirs(
        game_root: &Option<PathBuf>,
        engine_dir: &Option<PathBuf>,
    ) -> Vec<PathBuf> {
        let mut res = vec![];

        if let Some(game_root) = &game_root {
            res.push(game_root.join("Plugins"));
        }

        if let Some(engine_dir) = &engine_dir {
            res.push(engine_dir.join("Plugins"));
        }

        res
    }

    pub fn update_game_root(&mut self, asset_file_path: Option<PathBuf>) {
        self.asset_file_path = asset_file_path;

        let (game_root, content_dir) = Self::get_game_dirs(&self.asset_file_path);

        self.game_root = game_root;
        self.content_dir = content_dir;

        self.update_plugin_dirs();
    }

    pub fn update_engine_dir(&mut self, engine_dir: Option<PathBuf>) {
        let (engine_dir, engine_content_dir) = Self::get_engine_dirs(&engine_dir);

        self.engine_dir = engine_dir;
        self.engine_content_dir = engine_content_dir;

        self.update_plugin_dirs();
    }

    pub fn update_plugin_dirs(&mut self) {
        self.plugins_dirs = Self::get_plugins_dirs(&self.game_root, &self.engine_dir);
    }
}
