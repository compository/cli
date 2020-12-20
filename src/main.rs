use conductor_api::{app_websocket::AppWebsocket, types::ClientAppResponse};
use holochain_types::{app::InstalledCell, cell::CellId};
use publish::{publish_dna_template, publish_insantiated_dna};
use structopt::StructOpt;

mod conductor_api;
mod dna_file;
mod publish;
mod types;

use anyhow::{anyhow, Result};
use dna_file::{get_zomes, read_dna};
use tracing::instrument;

#[derive(Debug, StructOpt)]
#[structopt(name = "compository-publish")]
struct Opt {
    #[structopt(short = "w", long = "workdir")]
    workdir: std::path::PathBuf,
    #[structopt(short = "-c", long = "compository-dna-hash")]
    compository_dna_hash: String,
    #[structopt(short = "u", long = "url")]
    url: String,
    #[structopt(short = "i", long = "installed-app-id")]
    installed_app_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    run().await
}

#[instrument(err)]
async fn run() -> Result<()> {
    let opt = Opt::from_args();

    let dna_def_json = read_dna(&opt.workdir).await?;

    let dna_name = dna_def_json.name.clone();

    let zomes = get_zomes(&dna_def_json, &opt.workdir).await?;

    let mut ws = AppWebsocket::connect(opt.url.clone()).await?;

    println!("Connected to the holochain conductor at {}", opt.url);

    let compository_cell_id =
        get_compository_cell_id(&mut ws, opt.installed_app_id, opt.compository_dna_hash).await?;

    println!("Connected to compository with {:?}", compository_cell_id);

    let template_hash = publish_dna_template(&mut ws, &compository_cell_id, dna_name, zomes).await?;

    let dna_file = dna_def_json.compile_dna_file(&opt.workdir).await?;
    publish_insantiated_dna(&mut ws, &compository_cell_id, dna_file, template_hash).await?;

    Ok(())
}

async fn get_compository_cell_id(
    ws: &mut AppWebsocket,
    installed_app_id: String,
    compository_dna_hash: String,
) -> Result<CellId> {
    let app_info = ws.app_info(installed_app_id.clone()).await?;

    match app_info {
        ClientAppResponse::AppInfo(Some(info)) => {
            find_cell_for_dna(compository_dna_hash, info.cell_data).map(|c| c.into_id())
        }
        ClientAppResponse::AppInfo(None) => Err(anyhow!(format!(
            "Could not find app with it {}",
            installed_app_id
        ))),
        _ => Err(anyhow!("Bad response")),
    }
}

fn find_cell_for_dna(dna_hash: String, cells: Vec<InstalledCell>) -> Result<InstalledCell> {
    let maybe_compository_cell = cells.into_iter().find(|cell| {
        let dna_hash = format!("{:?}", cell.clone().into_id().dna_hash());
        dna_hash == dna_hash
    });

    maybe_compository_cell.ok_or(anyhow!(format!(
        "Could not find dna {} in this installed app",
        dna_hash
    )))
}
