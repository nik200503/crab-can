use clap::{Parser,Subcommand};
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{sethostname, execvp, fork, chroot, chdir, ForkResult};
use nix::sys::wait::{waitpid, WaitStatus};
use std::ffi::CString;
use anyhow::Result;
use std::path::Path;
use nix::mount::{mount, MsFlags};


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
		#[arg(long)]
		rootfs:String,
		
		#[arg(last = true, required = true)]
		command: Vec<String>,
	},
}

fn main()-> Result<()> {
    let cli= Cli::parse();
    
    match &cli.command{
    	Commands::Run{ rootfs , command} => {
    	
    		println!("(+) Unsharing Namespaces...");
    		unshare(CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS)?;
    		
    		match unsafe {fork()}? {
    			ForkResult::Parent{ child } => {
    				println!("(+) Parent waiting fr child (PID: {})", child);
    				waitpid(child, None)?;
    			}
    			ForkResult::Child => {
    				println!("(+) Child setting up container...");
    				
    				let root_path = Path::new(&rootfs);
    				println!("(+) Chrooting into : {}", rootfs);
    				chdir(root_path)?;
    				chroot(root_path)?;
    				chdir("/")?;
    				sethostname("crab-can")?;
    				
    				println!("(+) Mounting/ proc...");
    				const NONE: Option<&'static [u8]> = None;
    				mount(
    					Some("proc"),
    					"/proc",
    					Some("proc"),
    					MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
    					NONE
    				)?;
    				
    				let c_command = CString::new(command[0].clone())?;
    				let c_args: Vec<CString> = command
    					.iter()
    					.map(|arg| CString::new(arg.clone()).unwrap())
    					.collect();
    				
    				execvp(&c_command, &c_args)?;
    			}
    		}
    	}
    }
    Ok(())
}
