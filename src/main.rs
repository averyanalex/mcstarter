use anyhow::Result;
use clap::{Parser, Subcommand};
use lock::get_lock_entry;

use std::collections::HashMap;
use std::env::set_current_dir;
use std::fs::{self, create_dir_all};
use std::include_str;
use std::os::unix::process::CommandExt;
use std::process::Command;

mod build;
mod cache;
mod config;
mod download;
mod env;
mod hash;
mod lock;
mod merger;

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
    /// Download files to cache
    Download {
        #[clap(default_value_t = String::from("./cache"))]
        cache: String,
    },
    /// Build server
    Build {
        /// Target directory
        #[clap(default_value_t = String::from("./build"))]
        target: String,
        #[clap(default_value_t = String::from("./cache"))]
        cache: String,
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
            fs::write(format!("./mcstarter.yml"), default_config)?;
            println!("Initialized mcstarter.yml")
        }

        Commands::Lock {} => {
            println!("Locking...");
            let config = config::load_config(false)?;

            let core_bytes = download::download_core(&config.core, &config).await?;
            let core_hash = hash::hash_bytes(&core_bytes);

            let mut lock: HashMap<String, String> = HashMap::new();

            lock.insert(String::from("core"), core_hash);

            for (name, plugin) in &config.plugins {
                let plugin_bytes = download::download_plugin(name, &plugin, &config).await?;
                let plugin_hash = hash::hash_bytes(&plugin_bytes);
                lock.insert(name.clone(), plugin_hash);
            }

            lock::save_lock(&lock)?;
            println!("Done!");
        }

        Commands::Download { cache } => {
            let config = config::load_config(false)?;
            let lock = lock::load_lock()?;

            create_dir_all(cache)?;

            cache::cache_core(&config.core, &config, &lock, cache).await?;
            cache::cache_plugins(&config.plugins, &config, &lock, cache).await?;
        }

        Commands::Build { target, cache } => {
            let config = config::load_config(true)?;
            let lock = lock::load_lock()?;

            create_dir_all(target)?;

            build::build_core(&lock, &target, &cache).await?;
            build::build_plugins(&config.plugins, &lock, &target, &cache).await?;
            build::build_files(&config.include, &target).await?;
        }

        Commands::Launch { target } => {
            let config = config::load_config(true)?;
            let lock = lock::load_lock()?;

            let core_hash = get_lock_entry(&String::from("core"), &lock)?;

            set_current_dir(target)?;

            let mut args = config.launch.java_args;
            args.push_back(String::from("-jar"));
            args.push_back(format!("core-{core_hash}.jar"));

            let mut mc_args = config.launch.mc_args;
            args.append(&mut mc_args);

            print!("Running java");
            for arg in &args {
                print!(" {}", arg);
            }

            println!();

            Command::new("java").args(args).exec();
            panic!("can't launch")
        }
    }
    Ok(())
}
