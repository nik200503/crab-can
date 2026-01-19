mod container;

use anyhow::Result;
use clap::{Parser, Subcommand};
use container::Container;

#[derive(Parser)]
#[command(name = "crab-can")]
#[command(about = "A simple container runtime in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(long)]
        rootfs: String,

        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { rootfs, command } => {
            let container = Container::new(rootfs, command);
            container.run()?;
        }
    }
    Ok(())
}
