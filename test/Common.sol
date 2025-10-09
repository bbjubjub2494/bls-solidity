pragma solidity ^0.8;

import {TestBase} from "forge-std-1.10.0/src/Base.sol";

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
            result[i / 2] = bytes1(uint8(parseHexChar(buf[i]) * 16 + parseHexChar(buf[i + 1])));
        }
        return result;
    }
}

abstract contract Common is TestBase {
    struct TestCaseJson {
        // alphabetical order due to vm.parseJson quirks
        string application;
        uint64 drand_round_number; // Optional: 0 if n/a
        string dst;
        string[] hints;
        string m_expected;
        string message;
        string pk;
        string sig;
        string sig_compressed;
    }

    function eq(string memory a, string memory b) public pure returns (bool) {
        return Utils.eq(a, b);
    }

    function parseHex(string memory hexString) public pure returns (bytes memory) {
        return Utils.parseHex(hexString);
    }

    function loadTestCases(string memory filename) internal view returns (TestCaseJson[] memory) {
        bytes memory data = vm.parseJson(vm.readFile(string.concat(vm.envString("TEST_DATA_DIR"), "/", filename)));
        return abi.decode(data, (TestCaseJson[]));
    }
}
