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

use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::str::FromStr;

use core;
use core::package::{PackageIdent, PackageInstall};
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use launcher_protocol::LAUNCHER_PIPE_ENV;
use launcher_protocol::message::launcher as protocol;
use protobuf::{self, parse_from_bytes, Message};

use {SUP_CMD, SUP_PACKAGE_IDENT};
use error::{Error, Result};

const SUP_CMD_ENVVAR: &'static str = "HAB_SUP_BINARY";

type Receiver = IpcReceiver<Vec<u8>>;
type Server = IpcOneShotServer<Vec<u8>>;
type Sender = IpcSender<Vec<u8>>;

pub fn run() -> Result<ExitStatus> {
    let (server, pipe) = Server::new().map_err(Error::OpenPipe)?;
    let mut child = spawn_launcher(&pipe)?;
    let (rx, tx) = setup_connection(server)?;
    loop {
        match rx.recv() {
            Ok(bytes) => dispatch(&tx, &bytes),
            Err(err) => {
                println!("Error reading from Supervisor's pipe, {}", err);
                let status = child.wait().map_err(Error::SupShutdown)?;
                return Ok(status);
            }
        }
    }
}

pub fn send<T>(tx: &Sender, command: &T) -> Result<()>
    where T: protobuf::MessageStatic
{
    let mut msg = protocol::Envelope::new();
    msg.set_message_id(command.descriptor().name().to_string());
    msg.set_payload(command.write_to_bytes().map_err(Error::Serialize)?);
    let bytes = msg.write_to_bytes().map_err(Error::Serialize)?;
    tx.send(bytes).map_err(Error::Send)?;
    Ok(())
}

fn dispatch(tx: &Sender, bytes: &[u8]) {
    let msg = parse_from_bytes::<protocol::Envelope>(&bytes).unwrap();
    match msg.get_message_id() {
        "Spawn" => {
            let msg = parse_from_bytes::<protocol::Spawn>(msg.get_payload()).unwrap();
            println!("MSG!!! {:?}", msg);
            send(tx, &protocol::Ok::new()).unwrap();
        }
        unknown => println!("Received unknown message, {}", unknown),
    }
}

fn setup_connection(server: Server) -> Result<(Receiver, Sender)> {
    let (rx, raw) = server.accept().map_err(|_| Error::AcceptConn)?;
    let msg = parse_from_bytes::<protocol::Envelope>(&raw)
        .map_err(Error::Deserialize)?;
    let mut cmd = parse_from_bytes::<protocol::Register>(msg.get_payload())
        .map_err(Error::Deserialize)?;
    let tx = IpcSender::connect(cmd.take_pipe())
        .map_err(Error::Connect)?;
    send(&tx, &protocol::Ok::new()).unwrap();
    Ok((rx, tx))
}

fn spawn_launcher(pipe: &str) -> Result<Child> {
    let binary = supervisor_cmd()?;
    let mut command = Command::new(&binary);
    command.arg(pipe);
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .env(LAUNCHER_PIPE_ENV, pipe)
        .spawn()
        .map_err(Error::Spawn)?;
    Ok(child)
}

fn supervisor_cmd() -> Result<PathBuf> {
    if let Ok(command) = core::env::var(SUP_CMD_ENVVAR) {
        return Ok(PathBuf::from(command));
    }
    let ident = PackageIdent::from_str(SUP_PACKAGE_IDENT).unwrap();
    match PackageInstall::load_at_least(&ident, None) {
        Ok(install) => {
            match core::fs::find_command_in_pkg(SUP_CMD, &install, "/") {
                Ok(Some(cmd)) => Ok(cmd),
                _ => Err(Error::SupBinaryNotFound),
            }
        }
        Err(_) => Err(Error::SupPackageNotFound),
    }
}
