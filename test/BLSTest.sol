pragma solidity ^0.8;

import {Test, console2} from "forge-std-1.10.0/src/Test.sol";

import {BLS} from "src/libraries/BLS.sol";

import {Utils, Common} from "test/Common.sol";

contract BLSTest is Test, Common {
    struct TestCase {
        BLS.PointG2 pk;
        bytes sig;
        bytes sig_compressed;
        bytes message;
        string dst;
        BLS.PointG1 m_expected;
        uint256[] hints;
    }

    function fixture_tc() public view returns (TestCase[] memory testcases) {
        TestCaseJson[] memory json = loadTestCases("bn254_testcases.json");
        testcases = new TestCase[](json.length);
        for (uint256 i = 0; i < json.length; i++) {
            uint256[] memory hints = new uint256[](json[i].hints.length);
            for (uint256 j = 0; j < json[i].hints.length; j++) {
                hints[j] = BLS.fqUnmarshal(Utils.parseHex(json[i].hints[j]));
            }
            testcases[i] = TestCase({
                pk: BLS.g2Unmarshal(Utils.parseHex(json[i].pk)),
                sig: Utils.parseHex(json[i].sig),
                sig_compressed: Utils.parseHex(json[i].sig_compressed),
                message: Utils.parseHex(json[i].message),
                dst: json[i].dst,
                m_expected: BLS.g1Unmarshal(Utils.parseHex(json[i].m_expected)),
                hints: hints
            });
        }
    }

    function test_sample_signature() public {
        BLS.PointG2 memory pk = BLS.PointG2(
            [
                5838992826193349966357268616665404381433472226083567344457223955089099207810,
                7443551230336632695654952939029281494467559716523100842530624021561054602204
            ],
            [
                8587802453880553245650725475740899117965522901638095133768179426207296887795,
                21457554579773484299265442011477624571798879699696467812994904700443263942520
            ]
        );
        BLS.PointG1 memory sig = BLS.PointG1(
            7901903815049524482096231647574410489430116000772174647307272692982723667747,
            9602902115719281793852526489338977181632383432254830939648007541154086057984
        );
        string memory message = "hello";
        string memory dst =
            "dcipher-randomness-v01-BN254G1_XMD:KECCAK-256_SVDW_RO_0x0000000000000000000000000000000000000000000000000000000000000001";
        BLS.PointG1 memory messageP = BLS.hashToPoint(bytes(dst), bytes(message));
        BLS.verifySingle(sig, pk, messageP);
    }

    function table_marshal_unmarshal(TestCase memory tc) public pure {
        bytes memory g1data = tc.sig;
        assertEq(BLS.g1Marshal(BLS.g1Unmarshal(g1data)), g1data);

        bytes memory g2data = BLS.g2Marshal(tc.pk);
        assertEq(BLS.g2Marshal(BLS.g2Unmarshal(g2data)), g2data);
    }

    function table_verify(TestCase memory tc) public {
        BLS.PointG1 memory m = BLS.hashToPoint(bytes(tc.dst), tc.message);
        BLS.PointG1 memory sig = BLS.g1Unmarshal(tc.sig);
        assert(m.x == tc.m_expected.x);
        assert(m.y == tc.m_expected.y);

        (bool pairingSuccess, bool callSuccess) = BLS.verifySingle(sig, tc.pk, m);
        assert(pairingSuccess);
        assert(callSuccess);
    }

    function table_hints(TestCase memory tc) public {
        BLS.PointG2 memory pk = tc.pk;
        BLS.PointG1 memory sig = BLS.g1Unmarshal(tc.sig);
        BLS.PointG1 memory m_expected = tc.m_expected;

        BLS.TranscriptIterator memory t = BLS.TranscriptIterator({hints: tc.hints, position: 0});

        BLS.PointG1 memory m = BLS.hashToPointFromHints(bytes(tc.dst), tc.message, t);
        assert(m.x == m_expected.x);
        assert(m.y == m_expected.y);

        (bool pairingSuccess, bool callSuccess) = BLS.verifySingle(sig, pk, m);
        assert(pairingSuccess);
        assert(callSuccess);
    }

    function table_compressed(TestCase memory tc) public {
        BLS.PointG1 memory sig = BLS.g1UnmarshalCompressed(tc.sig_compressed);
        BLS.PointG1 memory sig_expected = BLS.g1Unmarshal(tc.sig);

        assertEq(sig.x, sig_expected.x);
        assertEq(sig.y, sig_expected.y);
    }

    function table_snapshot_verify_uncompressed_hints(TestCase memory tc) public {
        bytes memory sigBytes = tc.sig;

        BLS.TranscriptIterator memory t = BLS.TranscriptIterator({hints: tc.hints, position: 0});

        vm.startSnapshotGas("BLS", "verify_uncompressed_hints");
        BLS.PointG1 memory sig = BLS.g1Unmarshal(sigBytes);
        BLS.PointG1 memory m = BLS.hashToPointFromHints(bytes(tc.dst), tc.message, t);
        (bool pairingSuccess, bool callSuccess) = BLS.verifySingle(sig, tc.pk, m);
        vm.stopSnapshotGas();
        assert(pairingSuccess && callSuccess);
    }

    function table_snapshot_verify_uncompressed(TestCase memory tc) public {
        vm.startSnapshotGas("BLS", "verify_uncompressed");
        BLS.PointG1 memory sig = BLS.g1Unmarshal(tc.sig);
        BLS.PointG1 memory m = BLS.hashToPoint(bytes(tc.dst), tc.message);
        (bool pairingSuccess, bool callSuccess) = BLS.verifySingle(sig, tc.pk, m);
        vm.stopSnapshotGas();
        assert(pairingSuccess && callSuccess);
    }

    function table_snapshot_verify_compressed(TestCase memory tc) public {
        bytes memory sigBytes = tc.sig_compressed;

        vm.startSnapshotGas("BLS", "verify_compressed");
        BLS.PointG1 memory sig = BLS.g1UnmarshalCompressed(sigBytes);
        BLS.PointG1 memory m = BLS.hashToPoint(bytes(tc.dst), tc.message);
        (bool pairingSuccess, bool callSuccess) = BLS.verifySingle(sig, tc.pk, m);
        vm.stopSnapshotGas();
        assert(pairingSuccess && callSuccess);
    }
}
