import { type ToolManifest } from "@opensea/tool-sdk";

export const manifest: ToolManifest = {
  name: "ArkheGateway",
  description: "ARKHE HTTP Gateway integration for the 8257 registry.",
  endpoint: "https://localhost:8700/publish",
  inputs: {
    substrate: {
      type: "string",
      description: "Substrato de origem do decreto"
    },
    action: {
      type: "string",
      description: "Tipo de acao a publicar (e.g. ANCHOR)"
    },
    sequence: {
      type: "string",
      description: "Sequencia binaria canonica"
    }
  },
  outputs: {
    status: { type: "string", description: "status" },
    tx_hash: { type: "string", description: "tx_hash" },
    seal: { type: "string", description: "seal" },
    verification_url: { type: "string", description: "url" }
  },
  creatorAddress: "0x0000000000000000000000000000000000000000"
};
