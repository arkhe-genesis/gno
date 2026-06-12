// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

// Extensão do contrato CathedralGovernance
contract BTFSProviderRegistry {
    struct Provider {
        address wallet;
        bytes blsPublicKey;
        uint256 stake;
        bool active;
        uint256 reputation;
    }
    mapping(string => Provider) public providers; // key = peerId

    uint256 public minStake = 1000 * 10**18; // Example min stake

    event ProviderRegistered(string peerId, address wallet);
    event ProviderSlashed(string peerId, uint256 penalty);

    modifier onlyThreshold() {
        // Mock modifier for threshold signature requirement
        _;
    }

    function registerProvider(string memory peerId, bytes memory blsPubKey) external payable {
        require(msg.value >= minStake, "stake too low");
        providers[peerId] = Provider(msg.sender, blsPubKey, msg.value, true, 100);
        emit ProviderRegistered(peerId, msg.sender);
    }

    function slashProvider(string memory peerId, uint256 penalty, bytes memory proof) external onlyThreshold {
        Provider storage p = providers[peerId];
        require(p.active, "inactive");
        p.stake -= penalty;
        if (p.stake < minStake) p.active = false;
        emit ProviderSlashed(peerId, penalty);
        // Transfere penalidade para um fundo de compensação
    }
}
