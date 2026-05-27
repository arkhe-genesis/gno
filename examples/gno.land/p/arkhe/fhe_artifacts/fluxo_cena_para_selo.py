#!/usr/bin/env python3
# fluxo_cena_para_selo.py — Scene → Embedding → Brax → Seal → ERC‑8257
# Substrato 894 · Integração completa do Ontology SDK

from arkhe_sdk.core import ArkheOntologySDK
from llm_engine import ArkheLLMEngine
from brax_simulator import ArkheBraxSimulator

# Inicializa o SDK ontológico (conexão ao registry)
sdk = ArkheOntologySDK(registry_address="0x265BB2...D2cf1")

# 1. Descrever a cena
scene_description = "Uma esfera azul sobre uma mesa inclinada a 30 graus."

# 2. Obter embedding do LLM (244.1 via stub ou real)
llm = ArkheLLMEngine("models/arkhe-os.gguf", n_gpu_layers=35)
text_response, context_embedding = llm.generate(scene_description, max_tokens=64)

print(f"Context embedding shape: {context_embedding.shape}")

# 3. Simular no Brax (stub ou real)
sim = ArkheBraxSimulator(scene="intphys_scene")
state = sim.reset()
# Ação nula para observação inicial
next_state = sim.step(state, action=np.zeros(6))
world_emb = sim.get_world_embedding(next_state)

print(f"World embedding shape: {world_emb.shape}")

# 4. Combinar embeddings e gerar artefacto SDX
artifact = {
    "@context": {
        "sdx": "https://arkhe.org/ontology/sdx#",
        "arkhe": "https://arkhe.org/ontology/841#"
    },
    "@type": "sdx:SimulationScene",
    "sdx:artifactName": f"cena-{hash(scene_description) % 10000}",
    "sdx:hasVersion": {"sdx:versionString": "1.0"},
    "sdx:description": scene_description,
    "sdx:embedding": context_embedding[:16].tolist(),     # amostra do embedding do LLM
    "sdx:worldEmbedding": world_emb[:16].tolist(),       # amostra do embedding físico
    "sdx:simulationState": str(next_state)[:100]         # representação do estado
}

# 5. Gerar selo e CID
seal = sdk.generate_seal(artifact)
artifact["arkhe:hasSeal"] = {
    "arkhe:hashAlgorithm": "SHA3-256",
    "arkhe:sealHash": seal
}
cid = hashlib.sha3_256(json.dumps(artifact, sort_keys=True, default=str).encode()).hexdigest()[:46]
artifact["sdx:ipfsCID"] = f"ipfs://{cid}"

# 6. Registar no ERC‑8257 (preparar transação)
registration_data = sdk.register_artifact(artifact)
print(f"Transação preparada para {registration_data['name']}")
print(f"  ToolHash: {registration_data['checksum']}")
print(f"  MetadataURI: {registration_data['metadataURI']}")

print("\n✅ Fluxo completo executado. Artefacto pronto para ser submetido on-chain.")
