import aiohttp
from typing import Optional

class ZeroExTradingModule:
    def __init__(self, api_key: str, chain_id: int, wallet_address: str, zvec_memory, world_model=None):
        self.api_key = api_key
        self.chain_id = chain_id
        self.wallet = wallet_address
        self.zvec = zvec_memory
        self.world_model = world_model
        self.base_url = f"https://api.0x.org/swap/allowance-holder"
        self.session = aiohttp.ClientSession()

    async def execute_swap(self, sell_token: str, buy_token: str, sell_amount: int, slippage_bps: int = 100) -> Optional[dict]:
        """
        Executa um swap via 0x Swap API v2.
        Inclui validação de segurança e persistência de memória.
        """
        params = {
            "chainId": self.chain_id,
            "sellToken": sell_token,
            "buyToken": buy_token,
            "sellAmount": sell_amount,
            "taker": self.wallet,
            "slippageBps": slippage_bps,
        }
        headers = {"0x-api-key": self.api_key, "0x-version": "v2"}

        async with self.session.get(f"{self.base_url}/quote", params=params, headers=headers) as resp:
            if resp.status != 200:
                return None
            quote = await resp.json()

        # 1. Validação de Segurança (Z3) - Exemplo de condição
        if "issues" in quote and quote["issues"].get("liquidityAvailable") is False:
            return None

        # 2. Aprovação de Token (AllowanceHolder)
        allowance_params = {
            "chainId": self.chain_id,
            "sellToken": sell_token,
            "taker": self.wallet,
            "sellAmount": sell_amount,
        }
        async with self.session.get(f"{self.base_url}/approval", params=allowance_params, headers=headers) as resp:
            approval = await resp.json()

        # 3. Submissão da Transação (simplificado)
        tx_hash = await self._send_transaction(quote.get("transaction", {}))
        result = {"tx_hash": tx_hash, "buy_amount": quote.get("buyAmount")}

        # 4. Pós-processamento
        embedding = await self._get_market_embedding(sell_token, buy_token, sell_amount, quote)
        if self.zvec:
            self.zvec.store_transaction_embedding(embedding, result)
        reward = self._calc_reward(result)
        if self.world_model:
            self.world_model.update_personality_from_reward(reward)  # Atualiza personalidade do RSSM

        return result

    async def _send_transaction(self, tx_data: dict) -> str:
        # Mock para submissão de transação
        return "0x_mock_tx_hash"

    async def _get_market_embedding(self, sell_token: str, buy_token: str, sell_amount: int, quote: dict) -> list:
        # Mock de embedding do mercado
        return [0.1, 0.2, 0.3]

    def _calc_reward(self, result: dict) -> float:
        # Mock de cálculo de recompensa (ex: 1.0 se lucro > 0)
        return 1.0

    async def close(self):
        await self.session.close()
