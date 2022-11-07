#![feature(box_syntax)]

mod app;
mod asset;
mod dependency_tree;
mod util;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::eyre;
use graphviz_rust::cmd::{CommandArg, Format};
use graphviz_rust::dot_structures::Graph;
use graphviz_rust::exec;
use graphviz_rust::printer::PrinterContext;
use iced::window::Position;
use iced::{window, Application, Settings};

use crate::app::GuiApp;
use crate::asset::AssetDirs;
use crate::dependency_tree::DepTree;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A general helper too for the Unreal Engine's .uasset files",
    long_about = None
)]
struct Args {
    #[arg(long, default_value = "false")]
    gui: bool,

    #[arg(short, long)]
    file: Option<PathBuf>,

    #[arg(short, long)]
    engine: Option<PathBuf>,

    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    DependencyTree {
        #[arg(short = 'r', long, default_value = "64")]
        max_recurse_depth: u32,
    },
}

fn main() -> eyre::Result<()> {
    let Args {
        gui,
        file: uasset_file_path,
        engine: engine_dir,
        verbose,
        command,
    } = Args::parse();

    std::env::set_var(
        "RUST_LOG",
        match verbose {
            true => "debug",
            false => "info",
        },
    );

    pretty_env_logger::init();

    if !gui && uasset_file_path.is_none() {
        return Err(eyre::eyre!(
            "Please specify the asset path if not using the gui"
        ));
    }

    let asset_dirs = AssetDirs::new(uasset_file_path, engine_dir);

    if gui {
        run_app(asset_dirs)?;
    } else {
        match command {
            Command::DependencyTree { max_recurse_depth } => {
                let dependency_tree = DepTree::build_with_pb(&asset_dirs, max_recurse_depth)?;

                // graph gen
                {
                    use std::io::Write;

                    use graphviz_rust::printer::DotPrinter;

                    let graph: Graph = dependency_tree.into();

                    let graph_dot = graph.print(&mut PrinterContext::default());
                    let mut file = std::fs::File::create("deptree.dot")?;
                    file.write_all(graph_dot.as_bytes())?;

                    let graph_svg = exec(
                        graph,
                        &mut PrinterContext::default(),
                        vec![CommandArg::Format(Format::Svg)],
                    )
                    .unwrap();

                    let mut file = std::fs::File::create("deptree.svg")?;
                    file.write_all(graph_svg.as_bytes())?;
                }
            }
        }
    }

    Ok(())
}

fn run_app(asset_dirs: AssetDirs) -> eyre::Result<()> {
    let settings = Settings {
        id: None,
        window: window::Settings {
            size: (1000, 600),
            position: Position::Centered,
            min_size: Some((800, 600)),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: None,
        },
        flags: asset_dirs,
        default_font: Some(include_bytes!(
            "../resources/fonts/jetbrains_mono/fonts/ttf/JetBrainsMono-Medium.ttf"
        )),
        default_text_size: 18,
        text_multithreading: true,
        antialiasing: true,
        exit_on_close_request: true,
        try_opengles_first: false,
    };

    GuiApp::run(settings)?;

    Ok(())
}
