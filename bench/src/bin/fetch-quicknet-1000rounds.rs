use std::io::Write;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Round {
    pub round: u64,
    pub signature: String,
}


fn main() -> anyhow::Result<()> {
    let Some(path) = std::env::args_os().nth(1) else {
        Err(anyhow::anyhow!("pass output file as first argument"))?
    };
    let mut out_file = std::fs::File::create(path)?;
    let client = reqwest::blocking::Client::new();
    let network = "quicknet";
    for i in 1..=1000 {
        let sig = fetch_unverified(&client, network, i)?;
        out_file.write(&sig)?;
    }
    Ok(())
}

fn fetch_unverified(client: &reqwest::blocking::Client, network: &str, round_number: u64) -> anyhow::Result<[u8; 48]> {
    let rep = client.get(format!("https://api.drand.sh/v2/beacons/{network}/rounds/{round_number}")).send()?;
    let round: Round = serde_json::from_reader(rep)?;
    let mut sig = [0u8; 48];
    println!("{}", round.signature);
    hex::decode_to_slice(round.signature, &mut sig)?;
    Ok(sig)
}
