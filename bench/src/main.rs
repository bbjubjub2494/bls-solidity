use utils::hash_to_curve::CustomPairingHashToCurve;
use utils::serialize::point::{
    PointDeserializeCompressed, PointDeserializeUncompressed, PointSerializeCompressed,
    PointSerializeUncompressed,
};

use ark_bn254::Bn254;
use ark_ec::{AffineRepr, pairing::Pairing};
use ark_ff::Zero;

use alloy::{
        providers::{ext::AnvilApi, Provider, ProviderBuilder},
            node_bindings::Anvil,
};

use digest::Digest;

use std::fs::File;
use std::io::{BufReader, prelude::*};
use std::path::PathBuf;
use std::str::FromStr;

use foundry_contracts::evmnet_verifier::EvmnetVerifier;

static DST: &str = "BN254G1_XMD:KECCAK-256_SVDW_RO";

fn hex_ser_compressed(p: &impl PointSerializeCompressed) -> String {
    hex::encode(p.ser_compressed().unwrap())
}

fn hex_ser_uncompressed(p: &impl PointSerializeUncompressed) -> String {
    hex::encode(p.ser_uncompressed().unwrap())
}

fn hex_deser_compressed<T: PointDeserializeCompressed>(s: &str) -> T {
    let bytes = hex::decode(s).unwrap();
    T::deser_compressed(&bytes[..]).unwrap()
}

fn hex_deser_uncompressed<T: PointDeserializeUncompressed>(s: &str) -> T {
    let bytes = hex::decode(s).unwrap();
    T::deser_uncompressed(&bytes[..]).unwrap()
}

/*
use revm::{
    MainnetEvm,
    handler::instructions::EthInstructions,
};

pub fn create_evm_instance() -> MainnetEvm {
    MainnetEvm::new(
}
*/

use alloy_evm::{
    eth::{EthEvm, EthEvmFactory},
    env::EvmEnv,
    Evm,
    EvmFactory,
revm::database::{EmptyDBTyped, InMemoryDB},
revm::context::TxEnv,
};

use alloy::sol_types::SolCall;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = InMemoryDB::new(EmptyDBTyped::new());
    let mut evm = EthEvmFactory::default().create_evm(db, EvmEnv::default());

    // deploy the contract
    let tx = TxEnv::builder().create().data(EvmnetVerifier::BYTECODE.clone()).build().unwrap();
    let r = evm.transact_commit(tx)?;
    let contract_address = r.created_address().unwrap();


    let dst = format!("BLS_SIG_{DST}_NUL_");

    let pk = "07e1d1d335df83fa98462005690372c643340060d205306a9aa8106b6bd0b3820557ec32c2ad488e4d4f6008f89a346f18492092ccc0d594610de2732c8b808f0095685ae3a85ba243747b1b2f426049010f6b73a0cf1d389351d5aaaa1047f6297d3a4f9749b33eb2d904c9d9ebf17224150ddd7abd7567a9bec6c74480ee0b";

    let mut data = BufReader::new(File::open(PathBuf::from_str(
        "bench/data/evmnet_1000_rounds.bin",
    )?)?);
    for rn in 1u64..=1000 {
        let mut sig = [0u8; 32];
        data.read_exact(&mut sig)?;
        let p = hex_deser_uncompressed(pk);
        let s = ark_bn254::G1Affine::deser_compressed(&sig)?;
        let msg = &sha3::Keccak256::digest(rn.to_be_bytes());
        let m = Bn254::hash_to_g1_custom::<sha3::Keccak256>(msg, dst.as_bytes());

        assert!(
            Bn254::multi_pairing(&[m, s.into()], &[p, -ark_bn254::G2Affine::generator()]).is_zero()
        );

        let calldata =   EvmnetVerifier::verifyCompressedCall::from((sig.into(), rn)).abi_encode();
        let tx = TxEnv::builder().to(contract_address).data(calldata.into()).nonce(1).build().unwrap();
        let r = evm.transact(tx)?;
        println!("result: {r:?}");

        let calldata =   EvmnetVerifier::verifyUncompressedCall::from((s.ser_uncompressed()?.into(), rn)).abi_encode();
        let tx = TxEnv::builder().to(contract_address).data(calldata.into()).nonce(1).build().unwrap();
        let r = evm.transact(tx)?;
        println!("result: {r:?}");

        break; // TODO: remove
    }

    Ok(())
}
