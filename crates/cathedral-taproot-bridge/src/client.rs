// src/client.rs
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Request, metadata::MetadataValue};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub mod proto {
    tonic::include_proto!("taprootassets");
}

use proto::{
    taproot_assets_client::TaprootAssetsClient,
    asset_wallet_client::AssetWalletClient,
    universe_client::UniverseClient,
    // Mensagens principais
    GetInfoRequest, GetInfoResponse,
    ListAssetsRequest, ListAssetsResponse,
    ListBalancesRequest, ListBalancesResponse,
    NewAddrRequest, NewAddrResponse,
    SendAssetRequest, SendAssetResponse,
    BurnAssetRequest, BurnAssetResponse,
    QueryUniverseRequest, QueryUniverseResponse,
    VerifyProofRequest, VerifyProofResponse,
};

use crate::error::BridgeError;
use crate::auth::Macaroon;

/// Cliente avançado para o Taproot Assets Daemon (tapd).
#[derive(Clone)]
pub struct TaprootClient {
    /// Cliente principal do serviço TaprootAssets
    pub taproot: TaprootAssetsClient<Channel>,
    /// Cliente do serviço AssetWallet
    pub asset_wallet: AssetWalletClient<Channel>,
    /// Cliente do serviço Universe
    pub universe: UniverseClient<Channel>,
    /// Macaroon de autenticação
    macaroon: Option<Macaroon>,
}

impl TaprootClient {
    /// Conecta a um nó tapd via gRPC com autenticação completa.
    pub async fn connect(
        addr: &str,
        tls_config: Option<ClientTlsConfig>,
        macaroon_path: Option<&Path>,
    ) -> Result<Self, BridgeError> {
        let mut endpoint = tonic::transport::Endpoint::from_shared(addr.to_string())?;

        if let Some(tls) = tls_config {
            endpoint = endpoint.tls_config(tls)?;
        }

        let channel = endpoint.connect().await?;

        // Carrega macaroon
        let macaroon = if let Some(path) = macaroon_path {
            let bytes = fs::read(path)?;
            Some(Macaroon::from_bytes(bytes).map_err(|e| BridgeError::Macaroon(e.to_string()))?)
        } else {
            None
        };

        Ok(Self {
            taproot: TaprootAssetsClient::new(channel.clone()),
            asset_wallet: AssetWalletClient::new(channel.clone()),
            universe: UniverseClient::new(channel.clone()),
            macaroon,
        })
    }

    /// Adiciona macaroon aos metadados da requisição
    fn add_auth<T>(&self, mut req: Request<T>) -> Request<T> {
        if let Some(mac) = &self.macaroon {
            let mac_hex = hex::encode(mac.bytes());
            if let Ok(val) = MetadataValue::from_str(&mac_hex) {
                req.metadata_mut().insert("macaroon", val);
            }
        }
        req
    }

    pub async fn get_info(&mut self) -> Result<GetInfoResponse, BridgeError> {
        let req = GetInfoRequest {};
        let request = self.add_auth(Request::new(req));
        let response = self.taproot.get_info(request).await?;
        Ok(response.into_inner())
    }

    pub async fn list_assets(
        &mut self,
        with_witness: bool,
        include_spent: bool,
    ) -> Result<ListAssetsResponse, BridgeError> {
        let req = ListAssetsRequest {
            with_witness,
            include_spent,
            ..Default::default()
        };
        let request = self.add_auth(Request::new(req));
        let response = self.taproot.list_assets(request).await?;
        Ok(response.into_inner())
    }

    pub async fn list_balances(
        &mut self,
        _asset_id: bool,
        asset_id_val: Option<Vec<u8>>,
        group_key: Option<Vec<u8>>,
    ) -> Result<ListBalancesResponse, BridgeError> {
        let req = ListBalancesRequest {
            asset_id: asset_id_val.unwrap_or_default(),
            group_key: group_key.unwrap_or_default(),
            ..Default::default()
        };
        let request = self.add_auth(Request::new(req));
        let response = self.taproot.list_balances(request).await?;
        Ok(response.into_inner())
    }

    pub async fn new_address(
        &mut self,
        asset_id: Vec<u8>,
        amount: u64,
    ) -> Result<NewAddrResponse, BridgeError> {
        let req = NewAddrRequest {
            asset_id,
            amount,
            ..Default::default()
        };
        let request = self.add_auth(Request::new(req));
        let response = self.taproot.new_addr(request).await?;
        Ok(response.into_inner())
    }

    pub async fn send_asset(
        &mut self,
        addr: String,
        fee_rate: Option<u64>,
    ) -> Result<SendAssetResponse, BridgeError> {
        let req = SendAssetRequest {
            tap_addrs: vec![addr],
            fee_rate: fee_rate.unwrap_or(0) as u32,
            ..Default::default()
        };
        let request = self.add_auth(Request::new(req));
        let response = self.taproot.send_asset(request).await?;
        Ok(response.into_inner())
    }

    pub async fn burn_asset(
        &mut self,
        asset_id: Vec<u8>,
        amount: u64,
    ) -> Result<BurnAssetResponse, BridgeError> {
        let req = BurnAssetRequest { asset_id, amount };
        let request = self.add_auth(Request::new(req));
        let response = self.taproot.burn_asset(request).await?;
        Ok(response.into_inner())
    }

    pub async fn verify_proof(
        &mut self,
        proof: Vec<u8>,
    ) -> Result<VerifyProofResponse, BridgeError> {
        let req = VerifyProofRequest { proof };
        let request = self.add_auth(Request::new(req));
        let response = self.taproot.verify_proof(request).await?;
        Ok(response.into_inner())
    }

    pub async fn query_universe(
        &mut self,
        asset_id: Vec<u8>,
        _group_key: Option<Vec<u8>>,
    ) -> Result<QueryUniverseResponse, BridgeError> {
        let req = QueryUniverseRequest {
            id: Some(proto::universe_request::Id::AssetId(asset_id)),
            ..Default::default()
        };
        let request = self.add_auth(Request::new(req));
        let response = self.universe.query_universe(request).await?;
        Ok(response.into_inner())
    }

    /// Cria um novo ativo fungível.
    pub async fn create_fungible(
        &mut self,
        name: &str,
        supply: u64,
        metadata: &[u8],
    ) -> Result<crate::client::proto::Asset, BridgeError> {
        let req = proto::CreateAssetRequest {
            asset: Some(crate::client::proto::Asset {
                name: name.to_string(),
                amount: supply,
                asset_type: 0, // 0 = fungível
                asset_meta: Some(crate::client::proto::AssetMeta {
                    data: metadata.to_vec(),
                    type_url: String::new(),
                    meta_hash: vec![],
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let request = self.add_auth(tonic::Request::new(req));
        let response = self.asset_wallet.mint_asset(request).await?;
        let inner = response.into_inner();
        inner.pending_batch.and_then(|b| b.assets.into_iter().next())
            .ok_or_else(|| BridgeError::AssetNotFound("No asset in pending batch".to_string()))
    }
}
