// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "./CathedralSPHINCSVerifier.sol";

contract Wrapper is CathedralSPHINCSVerifier {
    uint256 public gasBefore;
    uint256 public gasAfter;
    bool public lastResult;

    function measureGas(
        bytes32 message,
        bytes calldata signature,
        bytes32 publicKeyRoot
    ) external {
        gasBefore = gasleft();
        // CALL directly without this. so we don't bubble up reverts aggressively if it fails, wait we do if we use standard call
        (bool success, bytes memory ret) = address(this).call(abi.encodeWithSelector(this.verifySPHINCS.selector, message, signature, publicKeyRoot));
        require(success, "Internal call failed");
        lastResult = abi.decode(ret, (bool));
        gasAfter = gasleft();
    }
}
