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

use foundry_contracts::{evmnet_verifier::EvmnetVerifier, quicknet_verifier::QuicknetVerifier};

macro_rules! write_row {
    ($f:expr, $($col:expr),+) => {
        writeln!($f, "{},{},{}", $($col),+)
    };
}

macro_rules! benchmark_network {
    (
        $network:ident,
        curve = $curve:ty,
        compressed_size = $comp_size:expr,
        verifier = $verifier:ident,
        data_path = $data_path:expr,
        output_path = $output_path:expr
    ) => {{
        println!("Benchmarking {} network...", stringify!($network));

        let db = InMemoryDB::new(EmptyDBTyped::new());
        let mut evm = EthEvmFactory::default().create_evm(db, EvmEnv::default());

        // deploy the contract
        let tx = TxEnv::builder()
            .create()
            .data($verifier::BYTECODE.clone())
            .build()
            .unwrap();
        let r = evm.transact_commit(tx)?;
        let contract_address = r.created_address().unwrap();

        let mut data = BufReader::new(File::open(PathBuf::from_str($data_path)?)?);

        let mut f = File::create($output_path)?;
        write_row!(f, "exec_gas", "data_gas", "algorithm")?;

        for rn in 1u64..=1000 {
            let mut sig = [0u8; $comp_size];
            data.read_exact(&mut sig)?;
            let s = <$curve>::deser_compressed(&sig)?;

            let (exec_gas, data_gas) = measure(
                &mut evm,
                contract_address,
                $verifier::verifyCompressedCall::from((sig.into(), rn)).abi_encode(),
            )?;

            write_row!(f, exec_gas, data_gas, "compressed")?;

            let (exec_gas, data_gas) = measure(
                &mut evm,
                contract_address,
                $verifier::verifyUncompressedCall::from((s.ser_uncompressed()?.into(), rn))
                    .abi_encode(),
            )?;
            write_row!(f, exec_gas, data_gas, "uncompressed")?;
        }

        Ok::<(), anyhow::Error>(())
    }};
}

fn main() -> anyhow::Result<()> {
    // Benchmark Quicknet (BLS12-381)
    benchmark_network!(
        quicknet,
        curve = ark_bls12_381::G1Affine,
        compressed_size = 48,
        verifier = QuicknetVerifier,
        data_path = "data/quicknet_1000_rounds.bin",
        output_path = "results/quicknet_verify_1000_evm.dat"
    )?;

    // Benchmark Evmnet (BN254)
    benchmark_network!(
        evmnet,
        curve = ark_bn254::G1Affine,
        compressed_size = 32,
        verifier = EvmnetVerifier,
        data_path = "data/evmnet_1000_rounds.bin",
        output_path = "results/evmnet_verify_1000_evm.dat"
    )?;

    Ok(())
}

const INTRINSIC_GAS: u64 = 21000;

fn measure<DB, I>(
    evm: &mut <EthEvmFactory as EvmFactory>::Evm<DB, I>,
    contract_address: Address,
    calldata: Vec<u8>,
) -> anyhow::Result<(u64, u64)>
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
    let ExecutionResult::Success { gas_used, .. } = evm.transact(tx)?.result else {
        panic!("unexpected result");
    };
    let exec_gas = gas_used
        .checked_sub(INTRINSIC_GAS + data_gas)
        .expect("unexpected gas");
    Ok((exec_gas, data_gas))
}
