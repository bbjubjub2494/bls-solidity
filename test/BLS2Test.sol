pragma solidity ^0.8;

import {Test} from "forge-std-1.10.0/src/Test.sol";

import {BLS2} from "src/libraries/BLS2.sol";

import {Common} from "test/Common.sol";

contract BLS2Test is Test, Common {
    function fixture_tc() public view returns (TestCaseJson[] memory filtered) {
        return loadTestCases("bls12_testcases.json");
    }

    function table_marshal_unmarshal(TestCaseJson memory tc) public pure {
        bytes memory g1data = parseHex(tc.sig);
        assertEq(BLS2.g1Marshal(BLS2.g1Unmarshal(g1data)), g1data);

        bytes memory g2data = parseHex(tc.pk);
        assertEq(BLS2.g2Marshal(BLS2.g2Unmarshal(g2data)), g2data);
    }

    function table_verify(TestCaseJson memory tc) public {
        BLS2.PointG2 memory pk = BLS2.g2Unmarshal(parseHex(tc.pk));
        BLS2.PointG1 memory sig = BLS2.g1Unmarshal(parseHex(tc.sig));
        BLS2.PointG1 memory m_expected = BLS2.g1Unmarshal(parseHex(tc.m_expected));

        BLS2.PointG1 memory m = BLS2.hashToPoint(bytes(tc.dst), parseHex(tc.message));
        assert(m.x_hi == m_expected.x_hi);
        assert(m.x_lo == m_expected.x_lo);
        assert(m.y_hi == m_expected.y_hi);
        assert(m.y_lo == m_expected.y_lo);

        (bool pairingSuccess, bool callSuccess) = BLS2.verifySingle(sig, pk, m);
        assert(pairingSuccess);
        assert(callSuccess);
    }

    function table_unmarshal_compressed(TestCaseJson memory tc) public {
        BLS2.PointG1 memory expected = BLS2.g1Unmarshal(parseHex(tc.sig));
        BLS2.PointG1 memory actual = BLS2.g1UnmarshalCompressed(parseHex(tc.sig_compressed));

        assert(actual.x_hi == expected.x_hi);
        assert(actual.x_lo == expected.x_lo);
        assert(actual.y_hi == expected.y_hi);
        assert(actual.y_lo == expected.y_lo);
    }

    function test_snapshot_verify_compressed() public {
        // snapshots do not work well in table tests as of Foundry 1.3.1, workaround here.
        TestCaseJson memory tc = fixture_tc()[3];
        BLS2.PointG2 memory pk = BLS2.g2Unmarshal(parseHex(tc.pk));
        bytes memory sigCompressedBytes = parseHex(tc.sig_compressed);
        bytes memory msg = parseHex(tc.message);

        vm.startSnapshotGas("BLS2", "verify_compressed");
        BLS2.PointG1 memory sig = BLS2.g1UnmarshalCompressed(sigCompressedBytes);
        BLS2.PointG1 memory m = BLS2.hashToPoint(bytes(tc.dst), msg);
        (bool pairingSuccess, bool callSuccess) = BLS2.verifySingle(sig, pk, m);
        vm.stopSnapshotGas();
        assert(pairingSuccess && callSuccess);
    }

    function test_snapshot_verify_uncompressed() public {
        // snapshots do not work well in table tests as of Foundry 1.3.1, workaround here.
        TestCaseJson memory tc = fixture_tc()[3];
        BLS2.PointG2 memory pk = BLS2.g2Unmarshal(parseHex(tc.pk));
        bytes memory sigBytes = parseHex(tc.sig);
        bytes memory msg = parseHex(tc.message);

        vm.startSnapshotGas("BLS2", "verify_uncompressed");
        BLS2.PointG1 memory sig = BLS2.g1Unmarshal(sigBytes);
        BLS2.PointG1 memory m = BLS2.hashToPoint(bytes(tc.dst), msg);
        (bool pairingSuccess, bool callSuccess) = BLS2.verifySingle(sig, pk, m);
        vm.stopSnapshotGas();
        assert(pairingSuccess && callSuccess);
    }
}
