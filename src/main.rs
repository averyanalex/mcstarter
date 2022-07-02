use clap::{Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Lock {} => {
            println!("Locking...")
        }
    }
}
