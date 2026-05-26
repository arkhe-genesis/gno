import click
import requests
import json
import yaml

@click.group()
def cli():
    """ARKHE-Z CLI — Interação com a Blockchain Z via Bridge"""
    pass

@cli.command()
@click.option('--sequence', required=True, help='Sequência canônica (36 bits)')
@click.option('--bridge-url', default='http://localhost:8700', help='URL da Bridge API')
@click.option('--format', 'fmt', type=click.Choice(['json', 'yaml']), default='json', help='Formato de saída')
def publish(sequence, bridge_url, fmt):
    """Publica a sequência na blockchain através da Bridge e exibe o decreto."""
    payload = {"sequence": sequence, "metadata": {"glosa": "245", "n": 5, "k": 2}}
    try:
        resp = requests.post(f"{bridge_url}/publish", json=payload)
        resp.raise_for_status()
        receipt = resp.json()
        # Gera o decreto estruturado
        decree = {
            "substrate": "870-B-GLOSA245",
            "action": "ANCHORED",
            "tx_hash": receipt["tx_hash"],
            "sequence_hash": receipt["sequence_hash"],
            "sequence": receipt["sequence"],
            "block_number": receipt["block_number"],
            "phi_c": 0.850,
            "ghost_threshold": 0.577,
            "metadata": receipt.get("metadata", {}),
            "timestamp": "2026-05-26T00:00:00Z",
            "keeper": "ψ"
        }
        if fmt == 'yaml':
            print(yaml.dump(decree, allow_unicode=True, sort_keys=False))
        else:
            print(json.dumps(decree, indent=2, ensure_ascii=False))
    except requests.exceptions.RequestException as e:
        click.echo(f"Erro na comunicação com a Bridge: {e}")

@cli.command()
@click.option("--sequence", required=True, help="Sequência binária B(2,5)")
@click.option("--rpc", required=True, help="URL do nó Ethereum")
@click.option("--private-key", required=True, help="Chave privada do Arquiteto")
@click.option("--contract", required=True, help="Endereço do Glosa245Anchor")
def anchor(sequence, rpc, private_key, contract):
    """Ancora a sequência canônica no contrato Glosa245Anchor."""
    from web3 import Web3
    w3 = Web3(Web3.HTTPProvider(rpc))
    account = w3.eth.account.from_key(private_key)
    expected_hash = Web3.keccak(text=sequence)
    # Construir transação
    contract_abi = [
        {
            "inputs": [{"internalType": "string", "name": "sequence", "type": "string"},
                       {"internalType": "bytes32", "name": "expectedHash", "type": "bytes32"}],
            "name": "anchorSequence",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]  # ABI do contrato
    anchor_contract = w3.eth.contract(address=contract, abi=contract_abi)
    tx = anchor_contract.functions.anchorSequence(sequence, expected_hash).build_transaction({
        'from': account.address,
        'nonce': w3.eth.get_transaction_count(account.address),
        'gas': 200000,
        'gasPrice': w3.eth.gas_price
    })
    signed_tx = account.sign_transaction(tx)
    tx_hash = w3.eth.send_raw_transaction(signed_tx.rawTransaction)
    click.echo(f"Transação enviada: {tx_hash.hex()}")

if __name__ == "__main__":
    cli()
