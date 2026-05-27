// @arkhe/sdk — TypeScript SDK for ERC-8257 + 870-G
import { ERC8257_ABI, type ERC8257Tool } from './bindings';
import { createPublicClient, http } from 'viem';

export class ArkheClient {
  private client;
  constructor(private contractAddress: string) {
    this.client = createPublicClient({ /* ... */ });
  }

  async getTool(hash: `0x${string}`): Promise<ERC8257Tool> {
    return this.client.readContract({
      address: this.contractAddress,
      abi: ERC8257_ABI,
      functionName: 'getTool',
      args: [hash],
    }) as Promise<ERC8257Tool>;
  }

  // Full autocompletion available in your IDE
}
