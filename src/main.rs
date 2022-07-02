use anyhow::Result;
use clap::{Parser, Subcommand};
use download::download;
use hash::hash;

use std::collections::HashMap;
use std::env::set_current_dir;
use std::fs::{self, create_dir_all};
use std::include_str;
use std::os::unix::process::CommandExt;
use std::process::Command;

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
    /// Generate mcstarter.yml in current directory
    Init {},
    /// Update mcstarter.lock
    Lock {},
    /// Build server
    Build {
        /// Target directory
        #[clap(default_value_t = String::from("./build"))]
        target: String,
    },
    /// Launch server
    Launch {
        /// Target directory
        #[clap(default_value_t = String::from("./build"))]
        target: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init {} => {
            let default_config = include_str!("mcstarter.yml");
            fs::write(format!("./mcstarter.yml"), &default_config)?;
            println!("Initialized mcstarter.yml")
        }
        Commands::Lock {} => {
            println!("Locking...");
            let config = config::load_config()?;

            let core = download(&config.core.url).await?;
            let core_hash = hash(&core);

            let mut lock: HashMap<String, String> = HashMap::new();

            lock.insert(String::from("core"), core_hash);

            for plugin in config.plugins.iter() {
                let plugin_bytes = download(&plugin.url).await?;
                let plugin_hash = hash(&plugin_bytes);
                lock.insert(plugin.name.clone(), plugin_hash);
            }

            lock::save_lock(lock)?;
            println!("Done!");
        }
        Commands::Build { target } => {
            let config = config::load_config()?;
            let lock = lock::load_lock()?;
            let core = download(&config.core.url).await?;
            let core_hash = hash(&core);
            if &&core_hash
                != lock
                    .get(&String::from("core"))
                    .get_or_insert(&String::from(""))
            {
                panic!("hash of core doen't match");
            } else {
                fs::write(format!("{target}/core.jar"), &core)?;
            }
            create_dir_all(target)?;
        }
        Commands::Launch { target } => {
            set_current_dir(target)?;
            Command::new("java").args(["-jar", "core.jar"]).exec();
            panic!("can't launch")
        }
    }
    Ok(())
}
