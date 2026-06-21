use crate::tensor::{Tensor, TensorDtype};

pub fn sigmoid(x: &Tensor) -> Tensor {
    x.sigmoid()
}

pub fn rms_norm(x: &Tensor, eps: TensorDtype) -> Tensor {
    x.rms_norm(eps)
}

pub fn layer_norm(x: &Tensor, eps: TensorDtype) -> Tensor {
    x.layer_norm(eps)
}

pub fn softmax(x: &Tensor, axis: usize) -> Tensor {
    x.softmax(axis)
}

pub fn gelu(x: &Tensor) -> Tensor {
    x.gelu()
}

pub fn relu(x: &Tensor) -> Tensor {
    x.relu()
}

pub fn swiglu_clamp(gate: &Tensor, up: &Tensor, clamp_limit: TensorDtype) -> Tensor {
    gate.swiglu_clamp(gate, up, clamp_limit)
}

pub fn clip_gradients(grads: &mut [Tensor], max_norm: TensorDtype) {
    let mut total_norm = 0.0f32;
    for grad in grads.iter() {
        total_norm += grad.mapv(|v| v * v).sum_all();
    }
    total_norm = total_norm.sqrt();

    if total_norm > max_norm {
        let scale = max_norm / total_norm;
        for grad in grads.iter_mut() {
            *grad = grad.scale(scale);
        }
    }
}

pub fn cosine_lr_schedule(
    step: u64,
    warmup_steps: u64,
    total_steps: u64,
    max_lr: f64,
    min_lr: f64,
) -> f64 {
    if step < warmup_steps {
        max_lr * (step as f64 / warmup_steps as f64)
    } else {
        let progress = (step - warmup_steps) as f64 / (total_steps - warmup_steps) as f64;
        min_lr + (max_lr - min_lr) * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos())
    }
}
