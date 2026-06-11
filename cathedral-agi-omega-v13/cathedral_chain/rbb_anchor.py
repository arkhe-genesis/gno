"""
cathedral_chain/rbb_anchor.py
Ancoragem de decisões de governança na RBB Chain (testnet/local).
Selo: RBB-ANCHOR-v1.0.0-2026-06-10
"""
import hashlib
import json
import time
from typing import Dict, Optional
from dataclasses import dataclass, asdict
from web3 import Web3

@dataclass
class GovernanceAnchor:
    """Registro imutável de uma decisão de governança."""
    proposal_id: str
    action: str
    target_node: str
    approved_by: list
    timestamp: int
    tx_hash: str

class RBBAnchor:
    def __init__(self, rpc_url: str, contract_address: str, private_key: str):
        self.w3 = Web3(Web3.HTTPProvider(rpc_url))
        self.contract_address = Web3.to_checksum_address(contract_address)
        self.private_key = private_key
        self.account = self.w3.eth.account.from_key(private_key)
        self.abi = [
            {
                "anonymous": False,
                "inputs": [
                    {"indexed": True, "name": "proposalId", "type": "string"},
                    {"indexed": False, "name": "action", "type": "string"},
                    {"indexed": False, "name": "targetNode", "type": "string"},
                    {"indexed": False, "name": "approvers", "type": "string[]"},
                    {"indexed": False, "name": "timestamp", "type": "uint256"}
                ],
                "name": "ProposalExecuted",
                "type": "event"
            },
            {
                "inputs": [
                    {"internalType": "string", "name": "proposalId", "type": "string"},
                    {"internalType": "string", "name": "action", "type": "string"},
                    {"internalType": "string", "name": "targetNode", "type": "string"},
                    {"internalType": "string[]", "name": "approvers", "type": "string[]"}
                ],
                "name": "anchorProposal",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            }
        ]
        self.contract = self.w3.eth.contract(address=self.contract_address, abi=self.abi)

    def anchor_proposal(self, proposal_id: str, action: str, target_node: str, approvers: list) -> str:
        """Ancora uma proposta aprovada na blockchain."""
        nonce = self.w3.eth.get_transaction_count(self.account.address)
        tx = self.contract.functions.anchorProposal(
            proposal_id, action, target_node, approvers
        ).build_transaction({
            'from': self.account.address,
            'nonce': nonce,
            'gas': 200000,
            'gasPrice': self.w3.eth.gas_price
        })
        signed = self.account.sign_transaction(tx)
        tx_hash = self.w3.eth.send_raw_transaction(signed.rawTransaction)
        print(f"[RBB] Proposta {proposal_id} ancorada: {tx_hash.hex()}")
        return tx_hash.hex()

    def get_proposal_events(self, proposal_id: str) -> Optional[Dict]:
        """Recupera eventos de uma proposta específica."""
        event_filter = self.contract.events.ProposalExecuted.create_filter(
            fromBlock=0,
            argument_filters={"proposalId": proposal_id}
        )
        events = event_filter.get_all_entries()
        if events:
            return dict(events[0]['args'])
        return None
