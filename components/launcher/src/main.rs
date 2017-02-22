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

extern crate habitat_launcher as launcher;

use std::thread;
use std::time::Duration;

use launcher::server;

fn main() {
    loop {
        println!("Starting Supervisor");
        match server::run() {
            Ok(status) => {
                println!("Supervisor went away with status {}, restarting", status);
            }
            Err(err) => {
                println!("Error starting Supervisor, {}", err);
                thread::sleep(Duration::from_millis(500));
            }
        }
    }
}
