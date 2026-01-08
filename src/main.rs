use clap::{Parser,Subcommand};
use std::process::Command;

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
    		
    		let mut child = Command::new(cmd);
    		child.args(args);
    		
    		let status = child.status().expect("failed to execute command");
    		
    		if status.success(){
    			println!("Command finished successfully!");
    		}else{
    			println!("Command failed with exit code: {:?}",status.code());
    		}
    	}
    }
}
