use anyhow::Result;
use nix::mount::{MsFlags, mount};
use nix::sched::{CloneFlags, unshare};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, chdir, chroot, execvp, fork, sethostname};
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

                println!("(+) Parent: Waiting for child to initialize network...");
                thread::sleep(Duration::from_secs(3));

                if let Err(e) = self.setup_veth(child) {
                    eprintln!("(!) Parent Error setting up network: {}", e);
                }

                println!("(+) Parent: Waiting for Container command to finish...");
                waitpid(child, None)?;
                println!("(+) parent: Cleaning up network...");
                let _ = Command::new("ip")
                    .args(["link", "delete", "veth-host"])
                    .output()?;
            }
            ForkResult::Child => {
                self.child_process()?;
            }
        }
        Ok(())
    }

    fn setup_veth(&self, child_pid: nix::unistd::Pid) -> Result<()> {
        println!("(+) Parent: Configuring Veth pair for PID {}", child_pid);

        let status = Command::new("ip")
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
            .output()?;
        if !status.status.success() {
            eprintln!(
                "(!) Failed to create veth: {:?}",
                String::from_utf8_lossy(&status.stderr)
            );
        }

        let status = Command::new("ip")
            .args(["link", "set", "veth-guest", "netns", &child_pid.to_string()])
            .output()?;
        if !status.status.success() {
            eprintln!(
                "(!) Failed to move veth: {:?}",
                String::from_utf8_lossy(&status.stderr)
            );
        }

        Command::new("ip")
            .args(["link", "set", "veth-host", "up"])
            .status()?;

        Ok(())
    }

    fn child_process(&self) -> Result<()> {
        println!("(+) child: Unsharing remaining namespaces...");

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
