use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _proto_root = PathBuf::from("proto");

    // Arquivos proto do tapd
    let _proto_files = [
        "proto/taprootassets.proto",
    ];

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("taprootassets.rs");
    std::fs::write(&dest_path, "
        pub mod taproot_assets_client {
            use tonic::transport::Channel;
            #[derive(Clone)] pub struct TaprootAssetsClient<T> { _marker: std::marker::PhantomData<T> }
            impl TaprootAssetsClient<Channel> {
                pub fn new(_c: Channel) -> Self { Self { _marker: std::marker::PhantomData } }
                pub async fn get_info(&mut self, _req: tonic::Request<super::GetInfoRequest>) -> Result<tonic::Response<super::GetInfoResponse>, tonic::Status> { unimplemented!() }
                pub async fn list_assets(&mut self, _req: tonic::Request<super::ListAssetsRequest>) -> Result<tonic::Response<super::ListAssetsResponse>, tonic::Status> { unimplemented!() }
                pub async fn list_balances(&mut self, _req: tonic::Request<super::ListBalancesRequest>) -> Result<tonic::Response<super::ListBalancesResponse>, tonic::Status> { unimplemented!() }
                pub async fn new_addr(&mut self, _req: tonic::Request<super::NewAddrRequest>) -> Result<tonic::Response<super::NewAddrResponse>, tonic::Status> { unimplemented!() }
                pub async fn send_asset(&mut self, _req: tonic::Request<super::SendAssetRequest>) -> Result<tonic::Response<super::SendAssetResponse>, tonic::Status> { unimplemented!() }
                pub async fn burn_asset(&mut self, _req: tonic::Request<super::BurnAssetRequest>) -> Result<tonic::Response<super::BurnAssetResponse>, tonic::Status> { unimplemented!() }
                pub async fn verify_proof(&mut self, _req: tonic::Request<super::VerifyProofRequest>) -> Result<tonic::Response<super::VerifyProofResponse>, tonic::Status> { unimplemented!() }
            }
        }
        pub mod asset_wallet_client {
            use tonic::transport::Channel;
            #[derive(Clone)] pub struct AssetWalletClient<T> { _marker: std::marker::PhantomData<T> }
            impl AssetWalletClient<Channel> {
                pub fn new(_c: Channel) -> Self { Self { _marker: std::marker::PhantomData } }
                pub async fn mint_asset(&mut self, _req: tonic::Request<super::CreateAssetRequest>) -> Result<tonic::Response<super::CreateAssetResponse>, tonic::Status> { unimplemented!() }
            }
        }
        pub mod universe_client {
            use tonic::transport::Channel;
            #[derive(Clone)] pub struct UniverseClient<T> { _marker: std::marker::PhantomData<T> }
            impl UniverseClient<Channel> {
                pub fn new(_c: Channel) -> Self { Self { _marker: std::marker::PhantomData } }
                pub async fn query_universe(&mut self, _req: tonic::Request<super::QueryUniverseRequest>) -> Result<tonic::Response<super::QueryUniverseResponse>, tonic::Status> { unimplemented!() }
            }
        }
        #[derive(Default)] pub struct GetInfoRequest {}
        #[derive(Default)] pub struct GetInfoResponse {}
        #[derive(Default)] pub struct ListAssetsRequest { pub with_witness: bool, pub include_spent: bool }
        #[derive(Default)] pub struct ListAssetsResponse {}
        #[derive(Default)] pub struct ListBalancesRequest { pub asset_id: Vec<u8>, pub group_key: Vec<u8> }
        #[derive(Default)] pub struct ListBalancesResponse {}
        #[derive(Default)] pub struct NewAddrRequest { pub asset_id: Vec<u8>, pub amount: u64 }
        #[derive(Default)] pub struct NewAddrResponse {}
        #[derive(Default)] pub struct SendAssetRequest { pub tap_addrs: Vec<String>, pub fee_rate: u32 }
        #[derive(Default)] pub struct SendAssetResponse {}
        #[derive(Default)] pub struct BurnAssetRequest { pub asset_id: Vec<u8>, pub amount: u64 }
        #[derive(Default)] pub struct BurnAssetResponse {}
        #[derive(Default)] pub struct VerifyProofRequest { pub proof: Vec<u8> }
        #[derive(Default)] pub struct VerifyProofResponse {}
        #[derive(Default)] pub struct QueryUniverseRequest { pub id: Option<universe_request::Id> }
        pub mod universe_request { pub enum Id { AssetId(Vec<u8>) } }
        #[derive(Default)] pub struct QueryUniverseResponse { pub asset_leaves: Vec<AssetLeaf> }
        #[derive(Default)] pub struct AssetLeaf {}
        #[derive(Default)] pub struct CreateAssetRequest { pub asset: Option<Asset> }
        #[derive(Default)] pub struct Asset { pub name: String, pub amount: u64, pub asset_type: u32, pub asset_meta: Option<AssetMeta> }
        #[derive(Default)] pub struct AssetMeta { pub data: Vec<u8>, pub type_url: String, pub meta_hash: Vec<u8> }
        #[derive(Default)] pub struct CreateAssetResponse { pub pending_batch: Option<PendingBatch> }
        #[derive(Default)] pub struct PendingBatch { pub assets: Vec<Asset> }
        #[derive(Default)] pub struct IssueAssetRequest {}
    ").unwrap();

    Ok(())
}
