// Copyright (C) 2019  Braiins Systems s.r.o.
//
// This file is part of Braiins Open-Source Initiative (BOSI).
//
// BOSI is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// Please, keep in mind that we may also license BOSI or any part thereof
// under a proprietary license. For more information on the terms and conditions
// of such proprietary license or if you have any other questions, please
// contact us at opensource@braiins.com.

//! Defines the API command handler (`Handler`)

pub use json::Value;
use serde_json as json;

use super::{MultiResponse, Response, ResponseSet};

/// A handler to be implemented by the API implementation,
/// takes care of producing a response for each command.
#[async_trait::async_trait]
pub trait Handler: Sync + Send {
    async fn handle_devs(&self) -> Option<Response>;
    async fn handle_edevs(&self) -> Option<Response>;
    async fn handle_version(&self) -> Option<Response>;
    async fn handle_config(&self) -> Option<Response>;
}

/// Holds an incomming API command
pub struct Command(Value);

impl Command {
    pub fn new(json: Value) -> Self {
        Self(json)
    }

    pub async fn handle_single(
        &self,
        cmd: &str,
        _param: Option<&Value>,
        handler: &dyn Handler,
    ) -> Option<Response> {
        match cmd {
            "devs" => handler.handle_devs().await,
            "edevs" => handler.handle_edevs().await,
            "version" => handler.handle_version().await,
            "config" => handler.handle_config().await,
            _ => None,
        }
    }

    pub async fn handle(&self, handler: &dyn Handler) -> Option<ResponseSet> {
        let cmd = self.0.get("command").and_then(Value::as_str)?;
        let param = self.0.get("parameter");

        if !cmd.contains('+') {
            self.handle_single(cmd, param, handler)
                .await
                .map(ResponseSet::Single)
        } else {
            let mut responses = MultiResponse::new();

            for cmd in cmd.split('+') {
                let resp = self.handle_single(cmd, param, handler).await?;
                let resp = json::to_value(&resp).ok()?;
                responses.add_response(cmd, resp);
            }

            Some(ResponseSet::Multi(responses))
        }
    }
}
