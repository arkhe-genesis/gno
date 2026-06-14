import pytest
import aiohttp
from unittest.mock import AsyncMock, patch, MagicMock
from zeroex_trading_module import ZeroExTradingModule

# Helper para mockar aiohttp response
class MockResponse:
    def __init__(self, json_data, status):
        self._json_data = json_data
        self.status = status

    async def json(self):
        return self._json_data

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        pass


@pytest.mark.asyncio
async def test_execute_swap_success():
    zvec_mock = MagicMock()
    world_model_mock = MagicMock()

    module = ZeroExTradingModule(
        api_key="test_key",
        chain_id=1,
        wallet_address="0x123",
        zvec_memory=zvec_mock,
        world_model=world_model_mock
    )

    quote_data = {
        "buyAmount": "1000",
        "transaction": {"data": "0xabc"}
    }

    approval_data = {"allowanceTarget": "0xdef"}

    with patch('aiohttp.ClientSession.get') as mock_get:
        # Primeiro get é o quote, segundo é o approval
        mock_get.side_effect = [
            MockResponse(quote_data, 200),
            MockResponse(approval_data, 200)
        ]

        result = await module.execute_swap("tokenA", "tokenB", 500)

        assert result is not None
        assert result["buy_amount"] == "1000"
        assert result["tx_hash"] == "0x_mock_tx_hash"

        # Verify interactions
        zvec_mock.store_transaction_embedding.assert_called_once()
        world_model_mock.update_personality_from_reward.assert_called_once_with(1.0)

    await module.close()

@pytest.mark.asyncio
async def test_execute_swap_quote_api_error():
    module = ZeroExTradingModule(
        api_key="test_key",
        chain_id=1,
        wallet_address="0x123",
        zvec_memory=None
    )

    with patch('aiohttp.ClientSession.get') as mock_get:
        mock_get.return_value = MockResponse({}, 400) # Error status

        result = await module.execute_swap("tokenA", "tokenB", 500)

        assert result is None

    await module.close()


@pytest.mark.asyncio
async def test_execute_swap_liquidity_issue():
    module = ZeroExTradingModule(
        api_key="test_key",
        chain_id=1,
        wallet_address="0x123",
        zvec_memory=None
    )

    quote_data = {
        "issues": {"liquidityAvailable": False}
    }

    with patch('aiohttp.ClientSession.get') as mock_get:
        mock_get.return_value = MockResponse(quote_data, 200)

        result = await module.execute_swap("tokenA", "tokenB", 500)

        assert result is None

    await module.close()
