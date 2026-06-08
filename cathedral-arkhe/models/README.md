# Modelos GGUF

## Download

### Llama 2 7B (recomendado para desenvolvimento)

```bash
# Via HuggingFace
huggingface-cli download meta-llama/Llama-2-7b-hf --local-dir ./hf_model

# Converter para GGUF
python scripts/convert_to_gguf.py ./hf_model --quant q4_k_m
```

### TinyLlama 1.1B (para testes rápidos)

```bash
huggingface-cli download TinyLlama/TinyLlama-1.1B-Chat-v1.0 --local-dir ./tinyllama
python scripts/convert_to_gguf.py ./tinyllama --quant q4_k_m --ctx 2048
```

## Formatos suportados

| Quant | Bits/param | Tamanho 7B | Qualidade |
|-------|-----------|-----------|----------|
| Q4_K_M | 4.5 | ~4.1 GB | Excelente |
| Q5_K_M | 5.5 | ~4.9 GB | Muito boa |
| Q6_K | 6.0 | ~5.4 GB | Boa |
| Q8_0 | 8.0 | ~7.2 GB | Quase idêntico ao F16 |
| F16 | 16.0 | ~14.4 GB | Original (sem quantização) |

## Estrutura GGUF

```
┌──────────────────┐
│ Header (24 bytes) │  magic + version + tensor_count + metadata_count
├──────────────────┤
│ Metadata KV      │  key(string) + type(uint32) + value
├──────────────────┤
│ Tensor Info Array  │  name(string) + dims + type_code + offset
├──────────────────┤
│ Tensor Data       │  pesos quantizados (alinhados a 32 bytes)
└──────────────────┘
```
