use utils::serialize::point::{PointDeserializeCompressed, PointSerializeUncompressed};

use alloy_evm::{
    Evm, EvmFactory,
    env::EvmEnv,
    eth::EthEvmFactory,
    revm::{
        context::TxEnv,
        context_interface::result::ExecutionResult,
        database::{Database, EmptyDBTyped, InMemoryDB},
        inspector::Inspector,
    },
};

use alloy::{
    primitives::{Address, U256},
    sol_types::SolCall,
};

use ark_ff::{BigInteger, PrimeField};

use sha3::{Digest, Keccak256};

use std::fs::File;
use std::io::{BufReader, prelude::*};
use std::path::PathBuf;
use std::str::FromStr;

use foundry_contracts::evmnet_verifier::EvmnetVerifier;

static DST: &[u8] = b"BLS_SIG_BN254G1_XMD:KECCAK-256_SVDW_RO_NUL_";

struct DataPoint {
    exec_gas: u64,
    data_gas: u64,
}

fn main() -> anyhow::Result<()> {
    let db = InMemoryDB::new(EmptyDBTyped::new());
    let mut evm = EthEvmFactory::default().create_evm(db, EvmEnv::default());

    // deploy the contract
    let tx = TxEnv::builder()
        .create()
        .data(EvmnetVerifier::BYTECODE.clone())
        .build()
        .unwrap();
    let r = evm.transact_commit(tx)?;
    let contract_address = r.created_address().unwrap();

    let mut data = BufReader::new(File::open(PathBuf::from_str(
        "bench/data/evmnet_1000_rounds.bin",
    )?)?);
    let mut compressed_data = Vec::<DataPoint>::new();
    let mut uncompressed_data = Vec::<DataPoint>::new();
    let mut hints_data = Vec::<DataPoint>::new();
    for rn in 1u64..=1000 {
        let mut sig = [0u8; 32];
        data.read_exact(&mut sig)?;
        let s = ark_bn254::G1Affine::deser_compressed(&sig)?;

        compressed_data.push(measure(
            &mut evm,
            contract_address,
            EvmnetVerifier::verifyCompressedCall::from((sig.into(), rn)).abi_encode(),
        )?);

        uncompressed_data.push(measure(
            &mut evm,
            contract_address,
            EvmnetVerifier::verifyUncompressedCall::from((s.ser_uncompressed()?.into(), rn))
                .abi_encode(),
        )?);

        let msg = &Keccak256::digest(rn.to_be_bytes());
        let (_, hints) = hash_to_curve::hash_to_g1_custom_with_hints::<Keccak256>(msg, DST);
        let hints: Vec<U256> = hints
            .into_iter()
            .map(|p| U256::from_be_slice(&p.into_bigint().to_bytes_be()))
            .collect();

        hints_data.push(measure(
            &mut evm,
            contract_address,
            EvmnetVerifier::verifyWithHintsCall::from((s.ser_uncompressed()?.into(), rn, hints))
                .abi_encode(),
        )?);
    }

    println!("Compressed:");
    summarize(&compressed_data);
    println!("Uncompressed:");
    summarize(&uncompressed_data);
    println!("With Hints:");
    summarize(&hints_data);

    Ok(())
}

fn measure<DB, I>(
    evm: &mut <EthEvmFactory as EvmFactory>::Evm<DB, I>,
    contract_address: Address,
    calldata: Vec<u8>,
) -> anyhow::Result<DataPoint>
where
    DB: Database + std::fmt::Debug,
    I: Inspector<alloy_evm::eth::EthEvmContext<DB>> + Default,
    <DB as Database>::Error: Sync + Send + 'static,
{
    let data_gas = calldata
        .iter()
        .map(|b| if *b == 0 { 4 } else { 16 })
        .sum::<u64>();
    let tx = TxEnv::builder()
        .to(contract_address)
        .data(calldata.into())
        .nonce(1)
        .build()
        .unwrap();
    let ExecutionResult::Success {
        gas_used: exec_gas, ..
    } = evm.transact(tx)?.result
    else {
        panic!("unexpected result");
    };
    Ok(DataPoint { exec_gas, data_gas })
}

fn compute_stats<I>(data: I) -> (f64, f64)
where
    I: Iterator,
    I::Item: Into<f64>,
{
    let mut count = 0f64;
    let mut sum = 0f64;
    let mut sum_sq = 0f64;
    for v in data {
        let v = v.into();
        count += 1f64;
        sum += v;
        sum_sq += v * v;
    }
    let mean = sum / count;
    let variance = (sum_sq / count) - (mean * mean);

    (mean, variance.sqrt())
}

fn summarize(data: &[DataPoint]) {
    let (exec_mean, exec_std) = compute_stats(data.iter().map(|d| d.exec_gas as f64));
    let (data_mean, data_std) = compute_stats(data.iter().map(|d| d.data_gas as f64));
    let (total_mean, total_std) =
        compute_stats(data.iter().map(|d| (d.exec_gas + d.data_gas) as f64));

    println!("Exec Gas: {:.2} ± {:.2}", exec_mean, exec_std);
    println!("Data Gas: {:.2} ± {:.2}", data_mean, data_std);
    println!("Total Gas: {:.2} ± {:.2}", total_mean, total_std);
}
