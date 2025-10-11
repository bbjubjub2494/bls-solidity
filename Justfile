alias t := test
alias fmt := format

test *args:
	@just test/bls_ffi/build
	TEST_DATA_DIR=$(cargo run --bin print_data_dir) forge t {{args}}

bench:
	forge bind --overwrite --select EvmnetVerifier
	cargo run --bin bench

deploy-quicknet *args:
	forge script {{args}} script/DeployQuicknetRegistry.s.sol

deploy-evmnet *args:
	forge script {{args}} script/DeployEvmnetRegistry.s.sol

quicknet-prove-latest *args:
	#!/bin/sh
	set -ev
	curl https://api.drand.sh/v2/beacons/quicknet/rounds/latest > out/quicknet_round.json
	registry=`jq -r .transactions[0].contractAddress broadcast/DeployQuicknetRegistry.s.sol/31337/run-latest.json`
	cast send {{args}} $registry "proveRound(bytes, uint64)" `jq -r .signature out/quicknet_round.json` `jq -r .round out/quicknet_round.json`

evmnet-prove-latest *args:
	#!/bin/sh
	set -ev
	curl https://api.drand.sh/v2/beacons/evmnet/rounds/latest > out/evmnet_round.json
	registry=`jq -r .transactions[0].contractAddress broadcast/DeployEvmnetRegistry.s.sol/31337/run-latest.json`
	cast send {{args}} $registry "proveRound(bytes, uint64)" `jq -r .signature out/evmnet_round.json` `jq -r .round out/evmnet_round.json`

lint:
	forge fmt --check
	forge lint

format:
	forge fmt
	just test/bls_ffi/format
