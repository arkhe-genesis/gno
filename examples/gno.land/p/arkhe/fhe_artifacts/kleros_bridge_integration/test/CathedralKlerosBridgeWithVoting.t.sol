// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../contracts/CathedralKlerosBridgeWithVoting.sol";
import "../contracts/PNKTheosisOracle.sol";

// Mock VeaInbox for testing
contract MockVeaInbox is IVeaInbox {
    uint64 public nonce;

    event MessageSent(address to, bytes4 fnSelector, bytes data);

    function sendMessage(address _to, bytes4 _fnSelector, bytes memory _data) external returns (uint64) {
        nonce++;
        emit MessageSent(_to, _fnSelector, _data);
        return nonce;
    }
}

contract CathedralKlerosBridgeWithVotingTest is Test {
    CathedralKlerosBridgeWithVoting bridge;
    PNKTheosisOracle oracle;
    MockVeaInbox veaInbox;

    address juror1 = address(0x1);
    address juror2 = address(0x2);
    address juror3 = address(0x4);
    address rbbTarget = address(0x3);

    function setUp() public {
        veaInbox = new MockVeaInbox();
        oracle = new PNKTheosisOracle();
        bridge = new CathedralKlerosBridgeWithVoting(address(veaInbox), rbbTarget, address(oracle));

        // Set initial Theosis scores
        oracle.updateScore(juror1, 100); // Low score
        oracle.updateScore(juror2, 900); // High score
        oracle.updateScore(juror3, 500); // Medium score
    }

    function testGetVotingWeight() public view {
        uint256 weight1 = bridge.getVotingWeight(juror1);
        uint256 weight2 = bridge.getVotingWeight(juror2);

        // BASE_WEIGHT = 1000, MAX_MULTIPLIER = 5000
        // Weight = 1000 + (Score * 4000 / 1000)
        // Juror1 (100): 1000 + 400 = 1400
        // Juror2 (900): 1000 + 3600 = 4600
        assertEq(weight1, 1400);
        assertEq(weight2, 4600);
        assertTrue(weight2 > weight1);
    }

    function testCastWeightedVoteAndDoubleVoting() public {
        uint256 disputeId = 1;
        bridge.createDispute(disputeId);

        vm.startPrank(juror1);
        bridge.castWeightedVote(disputeId, 1);
        vm.stopPrank();

        (bool active, uint256 totalVotes, uint256 winningChoice, uint256 maxVotes) = bridge.disputes(disputeId);
        assertTrue(active);
        assertEq(totalVotes, 1400);
        assertEq(winningChoice, 1);
        assertEq(maxVotes, 1400);

        // Try to vote again, should fail
        vm.startPrank(juror1);
        vm.expectRevert("Juror already voted");
        bridge.castWeightedVote(disputeId, 2);
        vm.stopPrank();

        vm.startPrank(juror2);
        bridge.castWeightedVote(disputeId, 2);
        vm.stopPrank();

        (, totalVotes, winningChoice, maxVotes) = bridge.disputes(disputeId);
        assertEq(totalVotes, 1400 + 4600);
        assertEq(winningChoice, 2); // Juror2's vote outweighs Juror1's
        assertEq(maxVotes, 4600);
    }

    function testRelayDecisionToRBB() public {
        uint256 disputeId = 2;
        bridge.createDispute(disputeId);

        vm.startPrank(juror1);
        bridge.castWeightedVote(disputeId, 1); // weight 1400
        vm.stopPrank();

        vm.startPrank(juror2);
        bridge.castWeightedVote(disputeId, 2); // weight 4600
        vm.stopPrank();

        vm.startPrank(juror3);
        bridge.castWeightedVote(disputeId, 1); // weight 3000 -> option 1 total = 4400
        vm.stopPrank();

        // Option 2 wins with 4600 > 4400

        bridge.relayDecisionToRBB(disputeId);

        (bool active, , uint256 winningChoice, ) = bridge.disputes(disputeId);
        assertFalse(active); // Dispute should be marked inactive after relaying
        assertEq(winningChoice, 2);

        assertEq(veaInbox.nonce(), 1);
    }
}
