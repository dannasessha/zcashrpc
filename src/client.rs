//! Includes both `Client` and all of the RPC response types.
#[macro_use]
mod defapi;

use crate::{ResponseResult, ZecAmount};
use reqwest;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::ops::RangeFrom;

/// A `Client` is used to make multiple requests to a specific zcashd RPC server. Requests are invoked by async methods that correspond to `zcashd` RPC API method names with request-specific parameters. Each such method has an associated response type.
pub struct Client {
    url: String,
    auth: String,
    reqcli: reqwest::Client,
    idit: RangeFrom<u64>,
}

impl Client {
    /// Construct a new `Client` with connection & authentication info.
    /// - `hostport` is a host/ip with an optional `:PORT` appended.
    /// - `authcookie` is the contents of `~/.zcash/.cookie`.
    pub fn new(hostport: String, authcookie: String) -> Client {
        Client {
            url: format!("http://{}/", hostport),
            auth: format!("Basic {}", authcookie),
            reqcli: reqwest::Client::new(),
            idit: (0..),
        }
    }

    /// Construct a `Client` using the values of the environment variables `"ZCASHRPC_HOST"` and `"ZCASHRPC_AUTH"` as the arguments to `Client::new`.
    pub fn from_env() -> Result<Client, std::env::VarError> {
        use std::env::var;

        let host = var("ZCASHRPC_HOST")?;
        let auth = var("ZCASHRPC_AUTH")?;
        Ok(Client::new(host, auth))
    }

    async fn make_request<R>(
        &mut self,
        method: &'static str,
        args: Vec<serde_json::Value>,
    ) -> ResponseResult<R>
    where
        R: DeserializeOwned,
    {
        use crate::{
            envelope::{RequestEnvelope, ResponseEnvelope},
            json,
        };

        let id = self.idit.next().unwrap();
        let reqresp = self
            .reqcli
            .post(&self.url)
            .header("Authorization", &self.auth)
            .body(&RequestEnvelope::wrap(id, method, args))
            .send()
            .await?;
        let text = reqresp.text().await?;
        let respenv: ResponseEnvelope = json::parse_value(json::parse_string(text)?)?;
        let resp = respenv.unwrap(id)?;
        Ok(resp)
    }
}

def_api_method! {
    getinfo() -> GetInfoResponse {
        balance: ZecAmount,
        blocks: u64,
        connections: u64,
        difficulty: f64,
        errors: String,
        keypoololdest: u64,
        keypoolsize: u64,
        paytxfee: ZecAmount,
        protocolversion: u64,
        proxy: String,
        relayfee: ZecAmount,
        testnet: bool,
        timeoffset: u64,
        version: u64,
        walletversion: u64
    }
}

def_api_method! {
    getblockchaininfo() -> GetBlockChainInfoResponse {
        chain: String,
        blocks: u64,
        headers: u64,
        bestblockhash: String,
        difficulty: f64,
        verificationprogress: f64,
        chainwork: String,
        pruned: bool,
        size_on_disk: u64,
        commitments: u64,
        valuePools: Vec<ValuePool>,
        softforks: Vec<Softfork>,
        upgrades: std::collections::HashMap<String, NetworkUpgradeDesc>,
        consensus: Consensus,
        pruneheight: Option<u64>,
        fullyNotified: Option<bool>
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ValuePool {
    pub id: String,
    pub monitored: bool,
    pub chainValue: Option<ZecAmount>,
    pub chainValueZat: Option<u64>,
    pub valueDelta: Option<ZecAmount>,
    pub valueDeltaZat: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Softfork {
    pub id: String,
    pub version: i64,
    pub enforce: SoftforkMajorityDesc,
    pub reject: SoftforkMajorityDesc,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SoftforkMajorityDesc {
    pub status: bool,
    pub found: i64,
    pub required: i64,
    pub window: serde_json::Value, // FIXME
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkUpgradeDesc {
    pub name: String,
    pub activationheight: u64,
    pub status: String, // FIXME: enum-ify
    pub info: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Consensus {
    pub chaintip: String,
    pub nextblock: String,
}
