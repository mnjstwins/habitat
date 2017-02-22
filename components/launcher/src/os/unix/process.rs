// Copyright (c) 2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::ffi::OsStr;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use core::os;
use core::os::process::Pid;
use libc;

use error::{Error, Result};

pub struct Child {
    pid: Pid,
    last_status: Option<i32>,
}

impl Child {
    pub fn new(child: &mut process::Child) -> Result<Child> {
        Ok(Child {
               pid: child.id() as Pid,
               last_status: None,
           })
    }

    pub fn id(&self) -> Pid {
        self.pid
    }

    pub fn status(&mut self) -> Result<HabExitStatus> {
        match self.last_status {
            Some(status) => Ok(HabExitStatus { status: Some(status as u32) }),
            None => {
                match process_status(self.pid) {
                    Ok(0) => Ok(HabExitStatus::default()),
                    Ok(code) => {
                        self.last_status = Some(code);
                        Ok(HabExitStatus::new(code))
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }

    pub fn kill(&mut self) -> Result<ShutdownMethod> {
        // check the group of the process being killed
        // if it is the root process of the process group
        // we send our signals to the entire process group
        // to prevent orphaned processes.
        let pgid = unsafe { libc::getpgid(self.pid) };
        if self.pid == pgid {
            debug!("pid to kill {} is the process group root. Sending signal to process group.",
                   self.pid);
            // sending a signal to the negative pid sends it to the
            // entire process group instead just the single pid
            self.pid = self.pid.neg();
        }

        signal(self.pid, Signal::TERM)?;
        let stop_time = SteadyTime::now() + Duration::seconds(8);
        loop {
            if let Ok(status) = self.status() {
                if !status.no_status() {
                    break;
                }
            }
            if SteadyTime::now() > stop_time {
                signal(self.pid, Signal::KILL)?;
                return Ok(ShutdownMethod::Killed);
            }
        }
        Ok(ShutdownMethod::GracefulTermination)
    }
}

impl ExitStatusExt for HabExitStatus {
    fn code(&self) -> Option<u32> {
        unsafe {
            match self.status {
                None => None,
                Some(status) if libc::WIFEXITED(status as libc::c_int) => {
                    Some(libc::WEXITSTATUS(status as libc::c_int) as u32)
                }
                _ => None,
            }
        }
    }

    fn signal(&self) -> Option<u32> {
        unsafe {
            match self.status {
                None => None,
                Some(status) if !libc::WIFEXITED(status as libc::c_int) => {
                    Some(libc::WTERMSIG(status as libc::c_int) as u32)
                }
                _ => None,
            }
        }
    }
}

pub fn exec<S>(path: S, svc_user: &str, svc_group: &str, env: &Env) -> Result<Command>
    where S: AsRef<OsStr>
{
    let mut cmd = Command::new(path);
    let uid =
        os::users::get_uid_by_name(svc_user)
            .ok_or(Error::Permissions(format!("No uid for user '{}' could be found", svc_user)))?;
    let gid =
        os::users::get_gid_by_name(svc_group)
            .ok_or(Error::Permissions(format!("No gid for group '{}' could be found", svc_group)))?;
    // We want the command to spawn processes in their own process group
    // and not the same group as the Launcher. Otherwise if a child process
    // sends SIGTERM to the group, the Launcher could be terminated.
    cmd.before_exec(setpgid(0, 0));
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .uid(uid)
        .gid(gid);
    for (key, val) in env.iter() {
        cmd.env(key, val);
    }
    Ok(cmd)
}

fn process_status(pid: Pid) -> Result<i32> {
    let mut status: i32 = 0;
    match unsafe { libc::waitpid(pid as i32, &mut status, libc::WNOHANG) } {
        0 => Ok(0),
        -1 => {
            // JW TODO: check errno and fill out a real error here
            Err(Error::WaitpidFailed(format!("Error calling waitpid on pid: {}", self.pid)))
        }
        _ => Ok(status),
    }
}

fn setpgid(pid: i32, pgid: i32) -> Result<()> {
    unsafe {
        libc::setpgid(pid, pgid);
    }
    Ok(())
}
