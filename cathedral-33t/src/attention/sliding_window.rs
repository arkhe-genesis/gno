use crate::tensor::Tensor;
use crate::config::AttentionConfig;

pub struct SlidingWindowAttention {
    window_size: usize,
}

impl SlidingWindowAttention {
    pub fn new(config: &AttentionConfig) -> Self {
        Self {
            window_size: config.sliding_window_size,
        }
    }

    pub fn forward(&self, x: &Tensor, _kv_cache: Option<&Tensor>) -> Tensor {
        let seq_len = x.shape()[0];
        if seq_len <= self.window_size {
            return x.clone();
        }
        x.slice_axis(0, seq_len - self.window_size)
    }
}
