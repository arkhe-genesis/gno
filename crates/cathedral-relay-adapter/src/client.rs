//! Cliente para o indexador Ponder (GraphQL)
//! Selo: CATHEDRAL-RELAY-CLIENT-v1.0.0-2026-06-22

use alloy::primitives::{Address, U256};
use reqwest::Client;
use serde_json::{json, Value};
use tracing::debug;

use crate::types::{VaultInfo, BalanceInfo, AssetAddress};
use crate::error::RelayError;
use crate::Result;

pub struct RelayIndexerClient {
    client: Client,
    base_url: String,
}

impl RelayIndexerClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    /// Consulta o saldo de um usuário em um vault específico
    pub async fn get_yield_earned(
        &self,
        user: Address,
        asset: AssetAddress,
    ) -> Result<U256> {
        let query = r#"
            query GetYieldEarned($user: String!, $asset: String!) {
                userVaults(where: { user: $user, asset: $asset }) {
                    totalYieldEarned
                }
            }
        "#;
        let variables = json!({
            "user": format!("0x{}", hex::encode(user)),
            "asset": format!("0x{}", hex::encode(asset.0)),
        });

        let response = self.graphql_query(query, variables).await?;
        let data = response["data"]["userVaults"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v["totalYieldEarned"].as_str())
            .ok_or(RelayError::Indexer("No data found".to_string()))?;

        let value = U256::from_str_radix(&data[2..], 16)
            .map_err(|e| RelayError::Indexer(e.to_string()))?;
        Ok(value)
    }

    /// Lista todos os vaults disponíveis
    pub async fn list_vaults(&self) -> Result<Vec<VaultInfo>> {
        let query = r#"
            query ListVaults {
                vaults {
                    address
                    asset
                    name
                    symbol
                    decimals
                    totalSupply
                    totalAssets
                    apy
                }
            }
        "#;
        let response = self.graphql_query(query, json!({})).await?;
        let vaults = response["data"]["vaults"]
            .as_array()
            .ok_or(RelayError::Indexer("Invalid response".to_string()))?;

        let mut result = Vec::new();
        for v in vaults {
            let info = VaultInfo {
                address: v["address"].as_str().unwrap_or("0x0").parse().unwrap(),
                asset: AssetAddress(v["asset"].as_str().unwrap_or("0x0").parse().unwrap()),
                name: v["name"].as_str().unwrap_or("").to_string(),
                symbol: v["symbol"].as_str().unwrap_or("").to_string(),
                decimals: v["decimals"].as_u64().unwrap_or(18) as u8,
                total_supply: U256::from_str_radix(v["totalSupply"].as_str().unwrap_or("0").trim_start_matches("0x"), 16).unwrap_or(U256::from(0)),
                total_assets: U256::from_str_radix(v["totalAssets"].as_str().unwrap_or("0").trim_start_matches("0x"), 16).unwrap_or(U256::from(0)),
                apy: v["apy"].as_f64(),
            };
            result.push(info);
        }
        Ok(result)
    }

    async fn graphql_query(
        &self,
        query: &str,
        variables: Value,
    ) -> Result<Value> {
        let response = self.client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "query": query,
                "variables": variables,
            }))
            .send()
            .await
            .map_err(|e| RelayError::Network(e.to_string()))?;

        let body: Value = response.json()
            .await
            .map_err(|e| RelayError::Indexer(e.to_string()))?;

        if let Some(errors) = body.get("errors") {
            return Err(RelayError::Indexer(format!("GraphQL error: {}", errors)));
        }

        Ok(body)
    }
}
