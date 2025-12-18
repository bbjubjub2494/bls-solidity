use anyhow::{anyhow, Result};

use std::io::Write;

use utils::serialize::point::{
    PointSerializeCompressed,
    PointDeserializeUncompressed,
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Round {
    pub round: u64,
    pub signature: String,
}


fn main() -> Result<()> {
    let Some(network) = std::env::args().nth(1) else {
        Err(anyhow!("pass network name as first argument"))?
    };
    let Some(path) = std::env::args_os().nth(2) else {
        Err(anyhow!("pass output file as second argument"))?
    };
    let mut out_file = std::fs::File::create(path)?;
    let client = reqwest::blocking::Client::new();
    match network.as_str() {
        "quicknet" => {
    for i in 1..=1000 {
        let mut buf = [0u8; 48];
        fetch_unverified(&client, &network, i, &mut buf)?;
        out_file.write(&buf)?;
    }
        },
        "evmnet" => {
    for i in 1..=1000 {
        let mut buf = [0u8; 64];
        fetch_unverified(&client, &network, i, &mut buf)?;
        let sig = ark_bn254::G1Affine::deser_uncompressed(&buf)?;
        out_file.write(&sig.ser_compressed()?)?;
    }
        },
        _ => Err(anyhow!("no such network: {:?}", network))?
    }
    Ok(())
}

fn fetch_unverified(client: &reqwest::blocking::Client, network: &str, round_number: u64, out: &mut [u8]) -> Result<()> {
    let rep = client.get(format!("https://api.drand.sh/v2/beacons/{network}/rounds/{round_number}")).send()?;
    let round: Round = serde_json::from_reader(rep)?;
    hex::decode_to_slice(round.signature, out)?;
    Ok(())
}
