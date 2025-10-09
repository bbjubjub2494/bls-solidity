pragma solidity ^0.8;

import {TestBase} from "forge-std-1.10.0/src/Base.sol";
import {Vm} from "forge-std-1.10.0/src/Vm.sol";


library Utils {
    function eq(string memory a, string memory b) internal pure returns (bool) {
        return keccak256(abi.encodePacked(a)) == keccak256(abi.encodePacked(b));
    }

    function parseHexChar(bytes1 b) internal pure returns (uint8) {
	    uint8 char = uint8(b);
	if (char >= 0x30 && char <= 0x39) {
	    return char - 0x30; // '0'-'9' -> 0-9
	} else if (char >= 0x41 && char <= 0x46) {
	    return char - 0x41 + 10; // 'A'-'F' -> 10-15
	} else if (char >= 0x61 && char <= 0x66) {
	    return char - 0x61 + 10; // 'a'-'f' -> 10-15
	} else {
	    revert("Invalid hex character");
	}
    }

    function parseHex(string memory hexString) internal pure returns (bytes memory) {
        bytes memory buf = bytes(hexString);
        bytes memory result = new bytes(buf.length / 2);
        for (uint256 i = 0; i < buf.length; i += 2) {
            result[i / 2] = bytes1(
                uint8(
                    parseHexChar(buf[i]) * 16
                        + parseHexChar(buf[i + 1])
                )
            );
        }
        return result;
    }

    function loadTestCases() internal view returns (Common.TestCase[] memory) {
Vm vm = Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D);
        bytes memory data = vm.parseJson(vm.readFile(string.concat(vm.envString("TEST_DATA_DIR"), "/testcases.json")));
        return abi.decode(data, (Common.TestCase[]));
    }
}

abstract contract Common is TestBase {
    // This is a common base contract for BLS2 tests and QuicknetRegistry tests.
    // It provides utility functions to read test cases and parse hex strings.
    struct TestCase {
        // alphabetical order due to vm.parseJson quirks
        string application;
        uint64 drand_round_number; // Optional: 0 if n/a
        string dst;
        string[] hints;
        string m_expected;
        string message;
        string pk;
        string scheme; // either "BN254" or "BLS12381"
        string sig;
        string sig_compressed;
    }

    function eq(string memory a, string memory b) public pure returns (bool) {
	    return Utils.eq(a, b);
    }

    function parseHex(string memory hexString) public pure returns (bytes memory) {
        return Utils.parseHex(hexString);
    }

    function fixture_tc() public view returns (TestCase[] memory testcases) {
	    return Utils.loadTestCases();
    }
}
