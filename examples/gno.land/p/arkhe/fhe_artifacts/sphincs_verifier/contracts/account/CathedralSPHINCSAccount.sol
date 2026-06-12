// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "../CathedralSPHINCSVerifier.sol";

// Mock interface for EntryPoint AA (EIP-4337)
interface IEntryPoint {
    function getNonce(address sender, uint192 key) external view returns (uint256 nonce);
}

contract CathedralSPHINCSAccount {
    CathedralSPHINCSVerifier public immutable verifier;
    bytes32 public immutable publicKeyRoot;
    IEntryPoint public immutable entryPoint;

    constructor(address _verifier, bytes32 _publicKeyRoot, address _entryPoint) {
        verifier = CathedralSPHINCSVerifier(_verifier);
        publicKeyRoot = _publicKeyRoot;
        entryPoint = IEntryPoint(_entryPoint);
    }

    function validateUserOp(
        bytes32 userOpHash,
        bytes calldata signature,
        uint256 missingAccountFunds
    ) external returns (uint256 validationData) {
        // Integrate with Cathedral AA EntryPoint
        bool isValid = verifier.verifySPHINCS(userOpHash, signature, publicKeyRoot);
        require(isValid, "Invalid SPHINCS signature");

        if (missingAccountFunds > 0) {
            (bool success, ) = msg.sender.call{value: missingAccountFunds}("");
            require(success, "Fund return failed");
        }
        return 0; // 0 for valid
    }
}
