use anyhow::Result;
use nix::mount::{MsFlags, mount};
use nix::sched::{CloneFlags, unshare};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, chdir, chroot, execvp, fork, sethostname};
use std::ffi::CString;
use std::path::Path;

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
        unshare(CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS)?;

        match unsafe { fork() }? {
            ForkResult::Parent { child } => {
                println!("(+) Parent waiting for child (PID: {})", child);
                waitpid(child, None)?;
                println!("(+) Container Exited.");
            }
            ForkResult::Child => {
                self.child_process()?;
            }
        }
        Ok(())
    }

    fn child_process(&self) -> Result<()> {
        println!("(+) child setting up container....");

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
