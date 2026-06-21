use crate::tensor::Tensor;
use crate::utils::math::{sigmoid, rms_norm};

pub struct ManifoldConstrainedHyperConnections {
    pub expansion_rate: usize,
    pub hidden_size: usize,
    pub phi_pre: Tensor,
    pub phi_post: Tensor,
    pub phi_res: Tensor,
    pub alpha_pre: f32,
    pub alpha_post: f32,
    pub alpha_res: f32,
    pub bias_pre: Tensor,
    pub bias_post: Tensor,
    pub bias_res: Tensor,
}

impl ManifoldConstrainedHyperConnections {
    pub fn new(hidden_size: usize, expansion_rate: usize) -> Self {
        let n = hidden_size;
        let c = expansion_rate;

        Self {
            expansion_rate: c,
            hidden_size: n,
            phi_pre: Tensor::randn(&[c * n, n]),
            phi_post: Tensor::randn(&[c * n, n]),
            phi_res: Tensor::randn(&[c * n, n * n]),
            alpha_pre: 0.5,
            alpha_post: 1.0,
            alpha_res: 1.0,
            bias_pre: Tensor::zeros(&[c * n]),
            bias_post: Tensor::zeros(&[c * n]),
            bias_res: Tensor::zeros(&[c * n]),
        }
    }

    pub fn forward<F>(&self, x: &Tensor, layer_fn: F) -> Tensor
    where
        F: Fn(&Tensor) -> Tensor,
    {
        let x_flat = x.flatten();

        let x_norm = rms_norm(&x_flat, 1e-6);
        let x_norm_2d = x_norm.reshape(&[1, self.hidden_size]);

        let h_pre_raw = x_norm_2d.matmul(&self.phi_pre.t())
            .scale(self.alpha_pre)
            .add_elem(&self.bias_pre.reshape(&[1, self.expansion_rate * self.hidden_size]));
        let _h_pre = sigmoid(&h_pre_raw);

        let h_post_raw = x_norm_2d.matmul(&self.phi_post.t())
            .scale(self.alpha_post)
            .add_elem(&self.bias_post.reshape(&[1, self.expansion_rate * self.hidden_size]));
        let _h_post = sigmoid(&h_post_raw).scale(2.0);

        let h_res_raw = x_norm_2d.matmul(&self.phi_res.t())
            .scale(self.alpha_res)
            .add_elem(&self.bias_res.reshape(&[1, self.expansion_rate * self.hidden_size]));

        let h_res_flat = h_res_raw.flatten();
        let h_res_2d = h_res_flat.reshape(&[self.expansion_rate, self.expansion_rate]);
        let h_res = sinkhorn_knopp(&h_res_2d, 10);

        let pre_transformed = self.project_pre(x);
        let layer_output = layer_fn(&pre_transformed);
        let post_transformed = self.project_post(&layer_output);
        let residual = self.project_res(x, &h_res);

        residual.add_elem(&post_transformed)
    }

    fn project_pre(&self, x: &Tensor) -> Tensor {
        let x_2d = x.reshape(&[1, self.hidden_size]);
        let proj = x_2d.matmul(&self.phi_pre.t());
        let biased = proj.add_elem(&self.bias_pre.reshape(&[1, self.expansion_rate * self.hidden_size]));
        sigmoid(&biased)
    }

    fn project_post(&self, x: &Tensor) -> Tensor {
        let x_2d = x.reshape(&[1, self.expansion_rate * self.hidden_size]);
        let proj = x_2d.matmul(&self.phi_post);
        let biased = proj.add_elem(&self.bias_post.reshape(&[1, self.hidden_size]));
        biased.scale(2.0)
    }

    fn project_res(&self, x: &Tensor, _h_res: &Tensor) -> Tensor {
        let x_2d = x.reshape(&[1, self.hidden_size]);
        let transformed = x_2d.matmul(&self.phi_res.t());
        transformed.add_elem(&self.bias_res.reshape(&[1, self.expansion_rate * self.hidden_size]))
    }

    pub fn num_parameters(&self) -> usize {
        let phi_pre_params = self.expansion_rate * self.hidden_size * self.hidden_size;
        let phi_post_params = self.expansion_rate * self.hidden_size * self.hidden_size;
        let phi_res_params = self.expansion_rate * self.hidden_size * self.hidden_size * self.hidden_size;
        let bias_params = 3 * self.expansion_rate * self.hidden_size;
        phi_pre_params + phi_post_params + phi_res_params + bias_params
    }
}

pub fn sinkhorn_knopp(m: &Tensor, iterations: usize) -> Tensor {
    let mut w = m.clone();
    let shape = w.shape();
    assert_eq!(shape.len(), 2, "Sinkhorn-Knopp requer matriz 2D");
    let rows = shape[0];
    let cols = shape[1];

    for _ in 0..iterations {
        for i in 0..rows {
            let mut row_sum = 0.0f32;
            for j in 0..cols {
                row_sum += w.get(&[i, j]);
            }
            if row_sum > 0.0 {
                for j in 0..cols {
                    let val = w.get(&[i, j]);
                    w.set(&[i, j], val / row_sum);
                }
            }
        }

        for j in 0..cols {
            let mut col_sum = 0.0f32;
            for i in 0..rows {
                col_sum += w.get(&[i, j]);
            }
            if col_sum > 0.0 {
                for i in 0..rows {
                    let val = w.get(&[i, j]);
                    w.set(&[i, j], val / col_sum);
                }
            }
        }
    }

    w
}
