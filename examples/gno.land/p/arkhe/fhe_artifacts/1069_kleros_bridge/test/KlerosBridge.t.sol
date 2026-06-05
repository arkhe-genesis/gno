// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../contracts/PNKTheosisOracle.sol";
import "../contracts/TheosisWeightedVoting.sol";
import "../contracts/CathedralKlerosBridgeWithVoting.sol";

contract KlerosBridgeTest is Test {
    PNKTheosisOracle oracle;
    TheosisWeightedVoting voting;
    CathedralKlerosBridgeWithVoting bridge;

    address owner = address(1);
    address updater = address(2);
    address juror1 = address(3);
    address juror2 = address(4);

    function setUp() public {
        vm.startPrank(owner);

        oracle = new PNKTheosisOracle();
        oracle.setUpdater(updater);

        // 50% max influence from Theosis
        voting = new TheosisWeightedVoting(address(oracle), 50);

        bridge = new CathedralKlerosBridgeWithVoting(address(voting));

        // Setup court 1 as Cathedral Court, court 2 as normal court
        bridge.setCourtStatus(1, true);
        bridge.setCourtStatus(2, false);

        vm.stopPrank();
    }

    function testOracleUpdate() public {
        vm.prank(updater);
        oracle.updateTheosis(juror1, 8000); // 0.8 Theosis

        assertEq(oracle.getTheosis(juror1), 8000);
    }

    function testTheosisWeightedVoting() public {
        vm.prank(updater);
        oracle.updateTheosis(juror1, 10000); // 1.0 Theosis (Max)

        uint256 baseWeight = 1000;

        // At max Theosis (10000) and 50 multiplier, added weight should be 50% of 1000 = 500
        uint256 effectiveWeight = voting.getEffectiveWeight(juror1, baseWeight);
        assertEq(effectiveWeight, 1500);

        // Juror 2 with 0 Theosis
        uint256 effectiveWeight2 = voting.getEffectiveWeight(juror2, baseWeight);
        assertEq(effectiveWeight2, 1000);
    }

    function testBridgeVotingPowerResolution() public {
        vm.prank(updater);
        oracle.updateTheosis(juror1, 5000); // 0.5 Theosis

        // Setup dispute 100 in court 1 (Cathedral Court)
        vm.prank(owner);
        bridge.createCathedralDispute(100, 1, "metadata");

        // Setup dispute 200 in court 2 (Normal Court)
        vm.prank(owner);
        bridge.createCathedralDispute(200, 2, "metadata");

        uint256 basePower = 1000;

        // In Cathedral Court, Theosis applies: 5000/10000 * 50% = 25% added power = 250
        uint256 powerCourt1 = bridge.getJurorVotingPower(100, juror1, basePower);
        assertEq(powerCourt1, 1250);

        // In Normal Court, Theosis does NOT apply
        uint256 powerCourt2 = bridge.getJurorVotingPower(200, juror1, basePower);
        assertEq(powerCourt2, 1000);
    }
}
