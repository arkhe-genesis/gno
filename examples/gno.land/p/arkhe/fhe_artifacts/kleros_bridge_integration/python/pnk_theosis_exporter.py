#!/usr/bin/env python3
"""
Prometheus Exporter for PNK Theosis Oracle.
Connects to the Oracle via Web3 and exposes metrics.
"""

import time
import os
import json
from prometheus_client import start_http_server, Gauge
from web3 import Web3

# Web3 Configuration
RPC_URL = os.environ.get("RPC_URL", "http://localhost:8545")
ORACLE_ADDRESS = os.environ.get("ORACLE_ADDRESS")
POLL_INTERVAL = int(os.environ.get("POLL_INTERVAL", "15"))

# Prometheus Metrics
THEOSIS_SCORE = Gauge('kleros_juror_theosis_score', 'Theosis score of a PNK Juror', ['juror_address'])

def load_abi():
    # Load ABI from artifacts
    try:
        with open("../artifacts/contracts/PNKTheosisOracle.sol/PNKTheosisOracle.json") as f:
            return json.load(f)["abi"]
    except FileNotFoundError:
        # Fallback minimal ABI for testing
        return [
            {
                "inputs": [{"internalType": "address", "name": "juror", "type": "address"}],
                "name": "getTheosisScore",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]

def main():
    if not ORACLE_ADDRESS:
        print("Warning: ORACLE_ADDRESS not set. Exporter will start but cannot fetch real data.")

    start_http_server(8000)
    print("Prometheus metrics available on port 8000 /metrics")

    if ORACLE_ADDRESS:
        w3 = Web3(Web3.HTTPProvider(RPC_URL))
        oracle_contract = w3.eth.contract(address=ORACLE_ADDRESS, abi=load_abi())

        # List of interesting jurors to monitor
        jurors_to_monitor = [
            "0x1111111111111111111111111111111111111111",
            "0x2222222222222222222222222222222222222222"
        ]

        while True:
            try:
                for juror in jurors_to_monitor:
                    score = oracle_contract.functions.getTheosisScore(juror).call()
                    THEOSIS_SCORE.labels(juror_address=juror).set(score)
            except Exception as e:
                print(f"Error fetching data: {e}")

            time.sleep(POLL_INTERVAL)
    else:
        while True:
            time.sleep(POLL_INTERVAL)

if __name__ == "__main__":
    main()
