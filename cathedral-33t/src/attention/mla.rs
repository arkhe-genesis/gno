use crate::tensor::Tensor;
use crate::config::AttentionConfig;

pub struct MultiHeadLatentAttention {
    latent_dim: usize,
    head_dim: usize,
    num_heads: usize,
    kv_compression: Tensor,
    kv_decompression: Tensor,
}

impl MultiHeadLatentAttention {
    pub fn new(config: &AttentionConfig) -> Self {
        let d_model = config.num_heads * config.head_dim;
        Self {
            latent_dim: config.mla_latent_dim,
            head_dim: config.head_dim,
            num_heads: config.num_heads,
            kv_compression: Tensor::randn(&[d_model, config.mla_latent_dim]),
            kv_decompression: Tensor::randn(&[config.mla_latent_dim, d_model]),
        }
    }

    pub fn forward(&self, x: &Tensor, kv_cache: Option<&Tensor>) -> (Tensor, Tensor) {
        let compressed = x.matmul(&self.kv_compression);

        let new_kv = if let Some(cache) = kv_cache {
            Tensor::concat(&[cache, &compressed], 0)
        } else {
            compressed.clone()
        };

        let decompressed = new_kv.matmul(&self.kv_decompression);
        (decompressed, new_kv)
    }

    pub fn compression_ratio(&self) -> f32 {
        (self.num_heads * self.head_dim) as f32 / self.latent_dim as f32
    }
}
