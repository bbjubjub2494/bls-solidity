use std::fs::File;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DrandRound {
    pub signature: String,
    pub round: u64,
}

fn fetch_evmnet_round(client: &Client, r: u64) -> anyhow::Result<String> {
    let sig_hex = client
        .get(format!("https://api.drand.sh/v2/beacons/evmnet/rounds/{r}"))
        .send()?
        .json::<DrandRound>()?
        .signature;
    Ok(sig_hex)
}

fn main() -> anyhow::Result<()> {
    // This disables the default behavior of cargo to rerun the build script every time something
    // changes in the crate. This would cause unnecessary rebuilds since this script only depends
    // on the network.
    build_rs::output::rerun_if_changed("build.rs");

    let client = Client::new();

    let mut rounds = Vec::new();
    for i in 1..=1000 {
        let round = fetch_evmnet_round(&client, i)?;
        rounds.push(round);
    }
    serde_json::to_writer_pretty(
        File::create(build_rs::input::out_dir().join("evmnet_1000_rounds.json"))?,
        &rounds,
    )?;
    Ok(())
}
