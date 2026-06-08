# Cathedral ARKHE

> *Recursive Self-Improvement with cryptographic verification, agentic adaptation, and provable safety guarantees.*

## Instalação

```bash
# Mínimo (simulação)
pip install cathedral-arkhe

# Inferência real
pip install cathedral-arkhe[llm]

# Tudo
pip install cathedral-arkhe[all]
```

## Arquitetura

```
                ┌────────────┐
                │  PLAN        │ ← AgenticLoop 1096
                │     ↓        │
                │  INFER       │ ← LlamaCppBridge 1094.2
                │     ↓        │
                │  ZKML        │ ← ZKMLBridge 1095.1 (ezkl)
                │     ↓        │
                │  STETHOSCOPE  │ ← Stethoscope 1081.1
                │     ↓        │
                │  THEOSIS     │ ← VectorTheosis 1091.2 (RKHS + φ²)
                │     ↓        │
                │  KLEROS      │ ← Kleros 1085.1 (bifurcação + ZK)
                │     ↓        │
                │  ANCHOR      │ ← TemporalChain 1097 (Merkle)
                │     ↓        │
                │  LEARN       │ ← Lições → LoRA 1098
                │     ↓        │
                │  LORA-TUNE   │ ← LoRAFineTuner 1098 (peft/trl)
                │     ↓        │
                │  HOT-RELOAD  │ ← LlamaCppBridge hot-reload
                │     ↓        │
                │  GARAK SCAN  │ ← GarakBridge 1099 (segurança)
                └────────────┘
```

## Substratos

| Nº | Nome | Versão | Função |
|---|------|--------|---------|
| 1094.1 | GGUF Bridge | v3.0.0 | mmap zero-copy de tensores GGUF |
| 1094.2 | LlamaCpp Bridge | v3.0.0 | Inferência com logits + LoRA hot-reload |
| 1091.2 | VectorTheosis | v4.0.0 | TEE via RKHS, Theosis via φ² |
| 1081.1 | Stethoscope | v3.0.0 | Trajetória de logits + FFT |
| 1085.1 | Kleros | v2.0.0 | Adjudicação com bifurcação + ZK |
| 1095.1 | ZKML Bridge | v2.0.0 | ezkl real: ONNX → circuito → prova → verificação |
| 1096 | Agentic Loop | v1.0.0 | ReAct + Reflection + auto-lessons |
| 1097 | TemporalChain | v2.0.0 | Merkle tree + ZK-Rollup |
| 1098 | LoRA Fine-Tuner | v1.0.0 | Lessons → SFT/DPO → adapter → hot-reload |
| 1099 | Garak Bridge | v1.0.0 | Scan de segurança adaptativo |

## Pesquisa

- [ICLR 2026 Workshop RSI](https://openreview.net/forum?id=ICLR-2026-WS-RSI) — self-improvement em produção
- [OECD 2026 Agentic AI](https://oecd.ai/en/publications/oecd-sti-2026) — governança multi-camada
- [Chen et al. 2026] — ZKML: provas ZK para GPT-2 em ~1h, verificação 18s
- [EZKL/Modulus 2026](https://github.com/zkonduit/ezkl) — zk-SNARKs para ML on-chain
- [Garak (NVIDIA)](https://github.com/NVIDIA/garak) — scanner de segurança LLM

## Licença

Apache-2.0 (porções Copyright NVIDIA CORPORATION & AFFILIATES)
