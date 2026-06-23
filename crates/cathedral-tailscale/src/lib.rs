// crates/cathedral-tailscale/src/lib.rs

pub mod client;
pub mod headscale;
pub mod psk;
pub mod derp;
pub mod metrics;
pub mod grants;

use crate::metrics::TailscaleMetrics;
use std::time::Instant;

// Mocks to pass the build since we are only implementing a skeleton based on the design doc
pub struct HeadscaleClient;
pub struct PskManager;
pub struct Did;
pub struct VerifiableCredential;
pub struct TailnetConnection;
pub struct WireGuardConfig;
pub struct Error;

impl HeadscaleClient {
    pub async fn authenticate(&self, _did: &Did, _cred: &VerifiableCredential) -> Result<String, Error> {
        Ok("identity".to_string())
    }
}

impl PskManager {
    pub async fn get_or_create(&self, _did: &Did) -> Result<String, Error> {
        Ok("psk".to_string())
    }
}

impl VerifiableCredential {
    pub fn verify(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl WireGuardConfig {
    pub fn new() -> Self { Self }
    pub fn with_psk(self, _psk: &str) -> Self { self }
    pub fn with_identity(self, _identity: &str) -> Self { self }
}

impl TailnetConnection {
    pub fn new(_cfg: WireGuardConfig) -> Self { Self }
}

/// Wrapper principal para integração Tailscale/Headscale
pub struct CathedralTailscale {
    headscale: HeadscaleClient,
    psk_manager: PskManager,
    metrics: TailscaleMetrics,
}

impl CathedralTailscale {
    /// Inicializa conexão com Headscale + autenticação DID
    pub async fn connect(
        &self,
        did: &Did,
        credential: &VerifiableCredential,
    ) -> Result<TailnetConnection, Error> {
        let start = Instant::now();
        // 1. Verificar credencial DID
        credential.verify()?;

        // 2. Obter PSK para esta conexão
        let psk = self.psk_manager.get_or_create(did).await?;

        // 3. Autenticar no Headscale via OIDC bridge
        let identity = self.headscale.authenticate(did, credential).await?;

        // 4. Configurar WireGuard com PSK
        let wg_config = WireGuardConfig::new()
            .with_psk(&psk)
            .with_identity(&identity);

        // 5. Registrar métricas
        self.metrics.handshake_latency.observe(start.elapsed().as_secs_f64());

        Ok(TailnetConnection::new(wg_config))
    }
}
