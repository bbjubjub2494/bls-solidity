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

use foundry_contracts::{
    evmnet_verifier::EvmnetVerifier,
    quicknet_verifier::QuicknetVerifier,
};

macro_rules! write_row {
    ($f:expr, $($col:expr),+) => {
        writeln!($f, "{:20}\t{:20}\t{:20}", $($col),+)
    };
}

fn main() -> anyhow::Result<()> {
    let db = InMemoryDB::new(EmptyDBTyped::new());
    let mut evm = EthEvmFactory::default().create_evm(db, EvmEnv::default());

    // deploy the contract
    let tx = TxEnv::builder()
        .create()
        .data(QuicknetVerifier::BYTECODE.clone())
        .build()
        .unwrap();
    let r = evm.transact_commit(tx)?;
    let contract_address = r.created_address().unwrap();

    let mut data = BufReader::new(File::open(PathBuf::from_str(
        "bench/data/quicknet_1000_rounds.bin",
    )?)?);

    let mut f = File::create("results/quicknet_verify_1000_evm.dat")?;
    write_row!(f, "exec_gas", "data_gas", "algorithm")?;

    for rn in 1u64..=1000 {
        let mut sig = [0u8; 48];
        data.read_exact(&mut sig)?;
        let s = ark_bls12_381::G1Affine::deser_compressed(&sig)?;

        let (exec_gas, data_gas) = measure(
            &mut evm,
            contract_address,
            QuicknetVerifier::verifyCompressedCall::from((sig.into(), rn)).abi_encode(),
        )?;

        write_row!(f, exec_gas, data_gas, "compressed")?;

        let (exec_gas, data_gas) = measure(
            &mut evm,
            contract_address,
            QuicknetVerifier::verifyUncompressedCall::from((s.ser_uncompressed()?.into(), rn))
                .abi_encode(),
        )?;
        write_row!(f, exec_gas, data_gas, "uncompressed")?;
    }

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
    let ExecutionResult::Success {
        gas_used, ..
    } = evm.transact(tx)?.result
    else {
        panic!("unexpected result");
    };
    let exec_gas = gas_used.checked_sub(INTRINSIC_GAS + data_gas).expect("unexpected gas");
    Ok((exec_gas, data_gas))
}
