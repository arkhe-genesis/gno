use crate::tensor::Tensor;

pub struct MONALiteOptimizer {
    muon: MuonOptimizer,
    acceleration_buffers: Vec<Tensor>,
    beta_a: f32,
    alpha: f32,
    streaming: bool,
    prev_grads: Option<Vec<Tensor>>,
    param_shapes: Vec<Vec<usize>>,
}

impl MONALiteOptimizer {
    pub fn new(
        param_shapes: Vec<Vec<usize>>,
        learning_rate: f32,
        beta_a: f32,
        alpha: f32,
        streaming: bool,
    ) -> Self {
        let buffers: Vec<Tensor> = param_shapes
            .iter()
            .map(|shape| Tensor::zeros(shape))
            .collect();

        Self {
            muon: MuonOptimizer::new(learning_rate),
            acceleration_buffers: buffers,
            beta_a,
            alpha,
            streaming,
            prev_grads: None,
            param_shapes,
        }
    }

    pub fn step(&mut self, grads: &[Tensor], lr: f32) {
        assert_eq!(
            grads.len(),
            self.acceleration_buffers.len(),
            "Número de gradientes ({}) não corresponde ao número de parâmetros ({})",
            grads.len(),
            self.acceleration_buffers.len()
        );

        let mut grads_clipped = grads.to_vec();
        crate::utils::math::clip_gradients(&mut grads_clipped, 1.0);

        for (i, grad) in grads_clipped.iter().enumerate() {
            let grad_diff = if self.streaming {
                self.compute_diff_streaming(grad, i)
            } else {
                match &self.prev_grads {
                    Some(prev) => grad.sub_elem(&prev[i]),
                    None => grad.clone(),
                }
            };

            self.acceleration_buffers[i] = self.acceleration_buffers[i]
                .scale(self.beta_a)
                .add_elem(&grad_diff.scale(1.0 - self.beta_a));

            let accelerated = grad.add_elem(&self.acceleration_buffers[i].scale(self.alpha));
            self.muon.step_single(&accelerated, lr, i);
        }

        self.prev_grads = Some(grads_clipped.to_vec());
    }

    fn compute_diff_streaming(&self, grad: &Tensor, _idx: usize) -> Tensor {
        grad.scale(0.1)
    }

    pub fn stats(&self) -> OptimizerStats {
        let total_buffer_size: usize = self.acceleration_buffers.iter().map(|b| b.len()).sum();
        OptimizerStats {
            num_parameters: self.param_shapes.len(),
            buffer_memory_mb: (total_buffer_size * 4) / (1024 * 1024),
            beta_a: self.beta_a,
            alpha: self.alpha,
        }
    }
}

struct MuonOptimizer {
}

impl MuonOptimizer {
    pub fn new(_lr: f32) -> Self {
        Self { }
    }

    pub fn step_single(&self, _grad: &Tensor, _lr: f32, _idx: usize) {
    }
}

#[derive(Debug, Clone)]
pub struct OptimizerStats {
    pub num_parameters: usize,
    pub buffer_memory_mb: usize,
    pub beta_a: f32,
    pub alpha: f32,
}
