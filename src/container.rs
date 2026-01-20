use anyhow::Result;
use nix::mount::{MsFlags, mount};
use nix::sched::{CloneFlags, unshare};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, chdir, chroot, execvp, fork, sethostname, Pid};
use std::ffi::CString;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct Container {
    pub rootfs: String,
    pub command: Vec<String>,
}

impl Container {
    pub fn new(rootfs: String, command: Vec<String>) -> Container {
        Container { rootfs, command }
    }

    pub fn run(&self) -> Result<()> {
        println!("(+) Unsharing Namespaces...");
        unshare(CloneFlags::CLONE_NEWPID)?;

        match unsafe { fork() }? {
            ForkResult::Parent { child } => {
                println!("(+) Parent waiting for child (PID: {})", child);

                thread::sleep(Duration::from_secs(1));
		
		self.setup_veth(child)?;

                println!("(+) Parent: Waiting for Child command...");
                waitpid(child, None)?;
                
                println!("(+) parent: Cleaning up network...");
                let _ = Command::new("ip")
                    .args(["link", "delete", "veth-host"])
                    .output();
            }
            ForkResult::Child => {
                self.child_process()?;
            }
        }
        Ok(())
    }

    fn setup_veth(&self, child_pid: Pid) -> Result<()> {
        println!("(+) Parent: Configuring Veth pair for PID {}", child_pid);
	
	let _ = Command::new("ip").args(["link", "delete", "veth-host"]).output();
	
      	Command::new("ip")
            .args([
                "link",
                "add",
                "veth-host",
                "type",
                "veth",
                "peer",
                "name",
                "veth-guest",
            ])
            .status()?;
	
	let netns_path = format!("/proc/{}/ns/net",child_pid);
        let status = Command::new("ip")
            .args(["link", "set", "veth-guest", "netns", &netns_path])
            .status()?;
        
        if !status.success() {
            eprintln!("(!) Failed to move veth. Is the child process dead?");
            return Ok(());
        }

	println!("(+) Parent: Assigning IP 10.0.0.1 to veth-host");
	Command::new("ip")
		.args(["addr", "add", "10.0.0.1/24","dev", "veth-host"])
		.status()?;
	
        Command::new("ip")
            .args(["link", "set", "veth-host", "up"])
            .status()?;

        Ok(())
    }

    fn child_process(&self) -> Result<()> {

        unshare(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNET)?;

        let root_path = Path::new(&self.rootfs);
        chdir(root_path)?;
        chroot(root_path)?;
        chdir("/")?;
        sethostname("crab-can")?;

        const NONE: Option<&'static [u8]> = None;
        mount(
            Some("proc"),
            "/proc",
            Some("proc"),
            MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
            NONE,
        )?;
        
        println!("(+) Child: Waiting for network interface...");
        let mut retries = 0;
        loop{
        	let output = Command::new("ip").arg("link").output();
        	
        	if let Ok(out) = output {
        		let s = String::from_utf8_lossy(&out.stdout);
        		if s.contains("veth-guest"){
        			println!("(+) Child: netwok cable detected!");
        			break;
        		}
        	}
        	
        	thread::sleep(Duration::from_millis(500));
        	retries +=1;
        	if retries > 10 {
        		eprintln!("(!) Child: Timed out waiting for network.");
        		break;
        	}
        }
        
        println!("(+) Child: Bringing up lo and veth-guest...");
        
        Command::new("ip").args(["link", "set", "lo", "up"]).status()?;
        
        Command::new("ip").args(["link", "set", "veth-guest", "up"]).status()?;
        Command::new("ip").args(["addr", "add", "10.0.0.2/24", "dev", "veth-guest"]).status()?;
        
        Command::new("ip").args(["route", "add", "default", "via", "10.0.0.1"]).status()?;

        let c_command = CString::new(self.command[0].clone())?;
        let c_args: Vec<CString> = self
            .command
            .iter()
            .map(|arg| CString::new(arg.clone()).unwrap())
            .collect();

        execvp(&c_command, &c_args)?;
        Ok(())
    }
}
