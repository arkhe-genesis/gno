// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./CathedralKlerosBridgeWithVoting.sol";

contract MockVeaInbox is IVeaInbox {
    uint64 public nonce;

    event MessageSent(address to, bytes4 fnSelector, bytes data);

    function sendMessage(address _to, bytes4 _fnSelector, bytes memory _data) external returns (uint64) {
        nonce++;
        emit MessageSent(_to, _fnSelector, _data);
        return nonce;
    }
}
