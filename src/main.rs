use clap::{Parser,Subcommand};

#[derive(Parser)]
#[command(name = "crab-can")]
#[command(about = "A simple container runtime in Rust", long_about = None)]

struct Cli{
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands{
	Run{
		#[arg(required = true)]
		cmd:String,
		
		#[arg(trailing_var_arg = true)]
		args:Vec<String>,
	},
}

fn main() {
    let cli= Cli::parse();
    
    match &cli.command{
    	Commands::Run{ cmd , args} => {
    		println!("Output : Preparing to run '{}' with args {:?}", cmd, args);
    	}
    }
}
