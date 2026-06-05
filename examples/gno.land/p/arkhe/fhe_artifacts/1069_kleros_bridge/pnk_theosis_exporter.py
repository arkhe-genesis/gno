#!/usr/bin/env python3
"""
Prometheus Exporter for PNK Theosis Oracle (Substrato 1069)
Connects to the EVM contract and exports the Theosis metrics of jurors
for Grafana visualization.
"""

import os
import time
import json
from web3 import Web3
from prometheus_client import start_http_server, Gauge

# Configs
RPC_URL = os.getenv("EVM_RPC_URL", "http://127.0.0.1:8545")
ORACLE_ADDRESS = os.getenv("ORACLE_ADDRESS", "0x0000000000000000000000000000000000000000")

# Setup Web3
w3 = Web3(Web3.HTTPProvider(RPC_URL))

# Mock ABI for getting Theosis
ABI = [
    {
        "inputs": [{"internalType": "address", "name": "juror", "type": "address"}],
        "name": "getTheosis",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    }
]

contract = w3.eth.contract(address=w3.to_checksum_address(ORACLE_ADDRESS), abi=ABI)

# Prometheus Metrics
JUROR_THEOSIS = Gauge('kleros_juror_theosis', 'Theosis score of a juror', ['juror_address'])

# In a real scenario, this would query an indexer/subgraph for active jurors
KNOWN_JURORS = [
    "0x1111111111111111111111111111111111111111",
    "0x2222222222222222222222222222222222222222",
    "0x3333333333333333333333333333333333333333"
]

def update_metrics():
    for juror in KNOWN_JURORS:
        try:
            # For testing without real network, we mock this if w3 is not connected
            if w3.is_connected():
                theosis_raw = contract.functions.getTheosis(w3.to_checksum_address(juror)).call()
                # Assuming 0-10000 scale
                theosis_normalized = theosis_raw / 10000.0
            else:
                # Mock data if not connected
                import random
                theosis_normalized = random.uniform(0.3, 0.9)

            JUROR_THEOSIS.labels(juror_address=juror).set(theosis_normalized)
            print(f"Updated {juror}: {theosis_normalized}")
        except Exception as e:
            print(f"Error updating juror {juror}: {e}")

if __name__ == '__main__':
    print(f"Starting PNK Theosis Exporter on port 8000. Target Oracle: {ORACLE_ADDRESS}")
    start_http_server(8000)

    while True:
        update_metrics()
        time.sleep(15)
