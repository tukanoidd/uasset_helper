use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    fs::File,
    path::{Path, PathBuf},
    rc::Rc,
};

use uasset::{AssetHeader, ImportIterator};

use crate::util::{path_to_str, SplitVecContainer};

#[derive(Debug, Clone, Eq, Hash)]
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

impl PartialEq for AssetError {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl std::error::Error for AssetError {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AssetOrigin {
    Project,
    Engine,
    ProjectPlugin,
    EnginePlugin,
}

impl ToString for AssetOrigin {
    fn to_string(&self) -> String {
        match self {
            AssetOrigin::Project => "Project",
            AssetOrigin::Engine => "Engine",
            AssetOrigin::ProjectPlugin => "Project Plugin",
            AssetOrigin::EnginePlugin => "Engine Plugin",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct Asset {
    pub package: AssetHeader<File>,
    pub path: PathBuf,
    pub origin: AssetOrigin,
}

impl Asset {
    pub fn new(asset_path: impl AsRef<Path>) -> Result<Self, AssetError> {
        if !asset_path.as_ref().is_file()
            || asset_path.as_ref().extension() != Some(OsStr::new("uasset"))
        {
            return Err(AssetError::new(
                asset_path.as_ref(),
                "File does not exist or is not a .uasset file",
            ));
        }

        let package = Self::read_asset(&asset_path)?;

        let plugin_path = asset_path
            .as_ref()
            .iter()
            .take_while(|seg| *seg != "Plugins")
            .collect::<PathBuf>()
            .join("Plugins");
        let is_plugin = plugin_path != asset_path.as_ref() && plugin_path.exists();

        let (is_engine, _) = AssetDirs::is_engine_path(&asset_path);

        let origin = match (is_engine, is_plugin) {
            (true, true) => AssetOrigin::EnginePlugin,
            (true, false) => AssetOrigin::Engine,
            (false, true) => AssetOrigin::ProjectPlugin,
            (false, false) => AssetOrigin::Project,
        };

        Ok(Self {
            package,
            path: asset_path.as_ref().to_path_buf(),
            origin,
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

#[derive(Clone)]
pub struct AssetDirs {
    pub asset_file_path: Option<PathBuf>,
    pub project_dir: Option<PathBuf>,
    pub content_dir: Option<PathBuf>,
    pub engine_dir: Option<PathBuf>,
    pub engine_content_dir: Option<PathBuf>,
    pub plugins_dirs: Vec<PathBuf>,

    pub project_git_repo: Option<Rc<git2::Repository>>,
    pub engine_git_repo: Option<Rc<git2::Repository>>,
}

impl Debug for AssetDirs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmtools::write!(
                f,
            "AssetDirs: {{\n"
            |f| {
                    f.write_str(&format!("\tasset_file_path: {:?},\n", self.asset_file_path))?;
                    f.write_str(&format!("\tproject_dir: {:?},\n", self.project_dir))?;
                    f.write_str(&format!("\tcontent_dir: {:?},\n", self.content_dir))?;
                    f.write_str(&format!("\tengine_dir: {:?},\n", self.engine_dir))?;
                    f.write_str(&format!("\tengine_content_dir: {:?},\n", self.engine_content_dir))?;
                    f.write_str(&format!("\tplugin_dirs: {:?},\n", self.plugins_dirs))?;

                    f.write_str(&format!("\tproject_git_repo: {},\n", match self.project_git_repo {
                        Some(_) => "Exists",
                        None => "Doesn't Exist",
                    }))?;
                    f.write_str(&format!("\tengine_git_repo: {},\n", match self.engine_git_repo {
                        Some(_) => "Exists",
                        None => "Doesn't Exist",
                    }))?;
                }
            "}}"
        )
    }
}

impl AssetDirs {
    pub fn new(asset_file_path: Option<PathBuf>, engine_dir: Option<PathBuf>) -> Self {
        let (project_dir, content_dir, engine_dir, engine_content_dir) =
            Self::get_dirs(&asset_file_path, &engine_dir);
        let plugins_dirs = Self::get_plugins_dirs(&project_dir, &engine_dir);

        let project_git_repo = Self::get_project_git_repo(&project_dir);
        let engine_git_repo = Self::get_engine_git_repo(&engine_dir);

        Self {
            asset_file_path,
            project_dir,
            content_dir,
            engine_dir,
            engine_content_dir,
            plugins_dirs,

            project_git_repo,
            engine_git_repo,
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

    pub fn project_dir_str(&self) -> Option<String> {
        self.project_dir.as_ref().map(path_to_str)
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

    pub fn get_project_dirs(
        asset_file_path: &Option<PathBuf>,
    ) -> (Option<PathBuf>, Option<PathBuf>) {
        let project_dir = asset_file_path
            .as_ref()
            .map(|asset_file_path| {
                asset_file_path
                    .iter()
                    .map(|x| x.to_str().unwrap())
                    .take_while(|x| x != &"Content")
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .map(|project_dir| {
                project_dir
                    .strip_prefix('/')
                    .unwrap_or(&project_dir)
                    .to_string()
            })
            .map(PathBuf::from);
        let content_dir = project_dir
            .as_ref()
            .map(|project_dir| project_dir.join("Content"));

        (project_dir, content_dir)
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

    pub fn get_dirs(
        asset_file_path: &Option<PathBuf>,
        engine_path: &Option<PathBuf>,
    ) -> (
        Option<PathBuf>,
        Option<PathBuf>,
        Option<PathBuf>,
        Option<PathBuf>,
    ) {
        let (project_dir, content_dir, engine_dir, engine_content_dir) = match asset_file_path {
            Some(asset_file_path_ref) => match Self::is_engine_path(asset_file_path_ref) {
                (true, Some((engine_dir, engine_content_dir))) => {
                    (None, None, Some(engine_dir), Some(engine_content_dir))
                }
                (false, None) => {
                    let (project_dir, content_dir) = Self::get_project_dirs(&asset_file_path);

                    (project_dir, content_dir, None, None)
                }
                _ => (None, None, None, None),
            },
            None => (None, None, None, None),
        };

        match (engine_dir, engine_content_dir) {
            (Some(engine_dir), Some(engine_content_dir)) => (
                project_dir,
                content_dir,
                Some(engine_dir),
                Some(engine_content_dir),
            ),
            _ => {
                let (engine_dir, engine_content_dir) = Self::get_engine_dirs(&engine_path);

                (project_dir, content_dir, engine_dir, engine_content_dir)
            }
        }
    }

    pub fn is_engine_path(asset_file_path: impl AsRef<Path>) -> (bool, Option<(PathBuf, PathBuf)>) {
        let engine_path: PathBuf = asset_file_path
            .as_ref()
            .iter()
            .take_while(|seg| *seg != "Engine")
            .collect::<PathBuf>()
            .join("Engine");

        let res = if engine_path == asset_file_path.as_ref() || !engine_path.exists() {
            (false, None)
        } else {
            let engine_content_dir = engine_path.join("Content");

            match engine_content_dir.exists() {
                true => (true, Some((engine_path, engine_content_dir))),
                false => (false, None),
            }
        };

        res
    }

    pub fn get_plugins_dirs(
        project_dir: &Option<PathBuf>,
        engine_dir: &Option<PathBuf>,
    ) -> Vec<PathBuf> {
        let mut res = vec![];

        if let Some(project_dir) = &project_dir {
            res.push(project_dir.join("Plugins"));
        }

        if let Some(engine_dir) = &engine_dir {
            res.push(engine_dir.join("Plugins"));
        }

        res
    }

    pub fn get_git_repos(
        project_dir: &Option<PathBuf>,
        engine_dir: &Option<PathBuf>,
    ) -> (Option<Rc<git2::Repository>>, Option<Rc<git2::Repository>>) {
        let project_git_repo = Self::get_project_git_repo(project_dir);
        let engine_git_repo = Self::get_engine_git_repo(engine_dir);

        (project_git_repo, engine_git_repo)
    }

    pub fn get_project_git_repo(project_dir: &Option<PathBuf>) -> Option<Rc<git2::Repository>> {
        project_dir
            .as_ref()
            .and_then(|project_dir| git2::Repository::open(project_dir).ok())
            .map(Rc::new)
    }

    pub fn get_engine_git_repo(engine_dir: &Option<PathBuf>) -> Option<Rc<git2::Repository>> {
        engine_dir
            .as_ref()
            .and_then(|engine_dir| git2::Repository::open(engine_dir).ok())
            .map(Rc::new)
    }

    pub fn update_asset_file(&mut self, asset_file_path: Option<PathBuf>) {
        self.asset_file_path = asset_file_path;

        let (project_dir, content_dir, engine_dir, engine_content_dir) =
            Self::get_dirs(&self.asset_file_path, &self.engine_dir);

        self.project_dir = project_dir;
        self.content_dir = content_dir;
        self.engine_dir = engine_dir;
        self.engine_content_dir = engine_content_dir;

        self.update_plugin_dirs();
        self.update_project_git_repo();
    }

    pub fn update_engine_dir(&mut self, engine_dir: Option<PathBuf>) {
        let (engine_dir, engine_content_dir) = Self::get_engine_dirs(&engine_dir);

        self.engine_dir = engine_dir;
        self.engine_content_dir = engine_content_dir;

        self.update_plugin_dirs();
        self.update_engine_git_repo();
    }

    pub fn update_plugin_dirs(&mut self) {
        self.plugins_dirs = Self::get_plugins_dirs(&self.project_dir, &self.engine_dir);
    }

    pub fn update_project_git_repo(&mut self) {
        self.project_git_repo = Self::get_project_git_repo(&self.project_dir);
    }

    pub fn update_engine_git_repo(&mut self) {
        self.engine_git_repo = Self::get_engine_git_repo(&self.engine_dir);
    }

    pub fn get_git_repo(&self, asset_origin: AssetOrigin) -> Option<Rc<git2::Repository>> {
        match asset_origin {
            AssetOrigin::Project | AssetOrigin::ProjectPlugin => self.project_git_repo.clone(),
            AssetOrigin::Engine | AssetOrigin::EnginePlugin => self.engine_git_repo.clone(),
        }
    }

    pub fn get_relative_path(&self, asset: &Asset) -> Option<PathBuf> {
        match asset.origin {
            AssetOrigin::Project | AssetOrigin::ProjectPlugin => self
                .project_dir
                .as_ref()
                .and_then(|project_dir| asset.path.strip_prefix(project_dir).ok().map(Into::into)),
            AssetOrigin::Engine | AssetOrigin::EnginePlugin => self
                .engine_dir
                .as_ref()
                .and_then(|engine_dir| asset.path.strip_prefix(engine_dir).ok().map(Into::into)),
        }
    }
}
