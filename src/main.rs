use clap::{Parser,Subcommand};
use std::process::Command;
use nix::sched::{unshare, CloneFlags};
use nix::unistd::sethostname;

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
    	
    		println!("Output: setting up container isolation...");
    		unshare(CloneFlags::CLONE_NEWUTS).expect("failed to unshare UTS namespace (did you run with sudo?)");
    		sethostname("crab-containers").expect("failed to set hostname");
    		
    		
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
