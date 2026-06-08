# Configuração ZKML (ezkl)

## Pré-requisitos

```bash
pip install ezkl onnx onnxruntime
```

## Compilação de circuito

O GarakBridge compila automaticamente sub-modelos extraídos do GGUF
(`output_norm.weight`, `token_embd.weight`) em circuitos halo2 via:

```python
# Automático via GarakBridge:
bridge.compile_circuit("model.gguf", "output_norm.weight", input_shape=(1, 4096))
```

## Compilação manual

```python
import torch
import ezkl

# 1. Sub-modelo PyTorch
model = torch.nn.LayerNorm(4096)

# 2. Exporta ONNX
torch.onnx.export(model, torch.randn(1, 4096), "model.onnx",
                   opset_version=14)

# 3. Compila circuito
ezkl.compile_circuit("model.onnx", "model.compiled",
                     settings_path="settings.json")

# 4. Gera keys
ezkl.gen_setup_params("model.compiled", "settings.json")
ezkl.get_vk("model.compiled", "settings.json", "model.vk")
ezkl.get_pk("model.compiled", "settings.json", "model.pk")

# 5. Gera prova
ezkl.prove("model.compiled", "model.pk",
          "input.json", "output.json", "proof.bin",
          "settings.json")

# 6. Verifica
ezkl.verify("proof.bin", "model.vk", "settings.json")
```

## Deploy on-chain

```python
from web3 import Web3

# Deploy verificador como contrato Ethereum
addr = ezkl.deploy_verifier(
    vk_path="model.vk",
    rpc_url="https://ethereum-mainnet.g.alchemy.com/v2/...",
    private_key="0x...",
)
```

## Limitações

- Modelos 7B+ são inviáveis para zk-proof direto
- Solução: provar sub-modelos (LayerNorm, classifier head)
- Tempo real: ~1h (GPT-2 small), ~18s verificação
- Gas: ~200k para verificação on-chain
