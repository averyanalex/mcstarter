use anyhow::Result;
use clap::{Parser, Subcommand};
use download::download;
use hash::hash;

use std::collections::HashMap;

mod config;
mod download;
mod hash;
mod lock;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Updates mcstarter.lock
    Lock {},
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Lock {} => {
            println!("Locking...");
            let config = config::load_config()?;

            let core = download(&config.core.url).await?;
            let core_hash = hash(core);

            let mut lock: HashMap<String, String> = HashMap::new();

            lock.insert(String::from("core"), core_hash);

            for plugin in config.plugins.iter() {
                let plugin_bytes = download(&plugin.url).await?;
                let plugin_hash = hash(plugin_bytes);
                lock.insert(plugin.name.clone(), plugin_hash);
            }

            lock::save_lock(lock)?;
            println!("Done!");
        }
    }
    Ok(())
}
