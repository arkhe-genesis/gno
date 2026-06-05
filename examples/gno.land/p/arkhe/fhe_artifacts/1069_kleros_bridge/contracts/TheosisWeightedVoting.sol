// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./PNKTheosisOracle.sol";

/**
 * @title TheosisWeightedVoting
 * @dev Computes the voting weight of a juror based on their base stake and their Theosis score.
 * Formula: EffectiveWeight = BaseWeight + (BaseWeight * TheosisScore / 10000 * TheosisMultiplier / 100)
 */
contract TheosisWeightedVoting {
    PNKTheosisOracle public oracle;

    // Multiplier for Theosis influence (e.g., 50 means max Theosis adds 50% weight)
    uint256 public theosisMultiplier;

    address public owner;

    event MultiplierUpdated(uint256 newMultiplier);

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner");
        _;
    }

    constructor(address _oracleAddress, uint256 _theosisMultiplier) {
        oracle = PNKTheosisOracle(_oracleAddress);
        theosisMultiplier = _theosisMultiplier;
        owner = msg.sender;
    }

    function setTheosisMultiplier(uint256 _multiplier) external onlyOwner {
        theosisMultiplier = _multiplier;
        emit MultiplierUpdated(_multiplier);
    }

    /**
     * @dev Calculates the effective voting weight of a juror.
     * @param juror The address of the juror.
     * @param baseWeight The base PNK staked/voting weight.
     */
    function getEffectiveWeight(address juror, uint256 baseWeight) public view returns (uint256) {
        uint256 theosis = oracle.getTheosis(juror); // 0 to 10000

        // Additional weight = Base * (Theosis / 10000) * (Multiplier / 100)
        uint256 additionalWeight = (baseWeight * theosis * theosisMultiplier) / (10000 * 100);

        return baseWeight + additionalWeight;
    }
}
