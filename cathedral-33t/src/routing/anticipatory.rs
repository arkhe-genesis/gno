use std::sync::{Arc, RwLock};
use crate::tensor::Tensor;
use crate::moe::RoutingIndex;

pub trait Router {
    fn route_simple(&self, token: &Tensor) -> Vec<usize>;
}

pub struct LossSpikeDetector {
    threshold: f32,
    loss_history: Vec<f32>,
    window_size: usize,
}

impl LossSpikeDetector {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            loss_history: Vec::with_capacity(100),
            window_size: 10,
        }
    }

    pub fn update(&mut self, loss: f32) {
        self.loss_history.push(loss);
        if self.loss_history.len() > self.window_size * 2 {
            self.loss_history.remove(0);
        }
    }

    pub fn detect(&mut self) -> bool {
        if self.loss_history.len() < self.window_size {
            return false;
        }

        let recent: f32 = self.loss_history.iter().rev().take(self.window_size).sum::<f32>()
            / self.window_size as f32;
        let previous: f32 = self.loss_history.iter().rev().skip(self.window_size).take(self.window_size).sum::<f32>()
            / self.window_size.min(self.loss_history.len() - self.window_size) as f32;

        if previous > 0.0 {
            let spike_ratio = (recent - previous) / previous;
            spike_ratio > self.threshold
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.loss_history.clear();
    }
}

pub struct AnticipatoryRouter<R: Router> {
    router: R,
    routing_index: Arc<RwLock<Option<RoutingIndex>>>,
    loss_spike_detector: LossSpikeDetector,
    routing_delay: usize,
    delayed_features: Vec<Tensor>,
    features_index: usize,
}

impl<R: Router> AnticipatoryRouter<R> {
    pub fn new(router: R, routing_delay: usize) -> Self {
        Self {
            router,
            routing_index: Arc::new(RwLock::new(None)),
            loss_spike_detector: LossSpikeDetector::new(0.1),
            routing_delay,
            delayed_features: Vec::with_capacity(routing_delay + 1),
            features_index: 0,
        }
    }

    pub fn route_with_anticipation(&mut self, token: &Tensor, current_loss: Option<f32>) -> Vec<usize> {
        if let Some(loss) = current_loss {
            self.loss_spike_detector.update(loss);
        }

        let features = self.compute_features(token);

        if self.loss_spike_detector.detect() {
            let index = self.compute_routing_index(token);
            *self.routing_index.write().unwrap() = Some(index);
        }

        let delayed_feat = self.get_delayed_feature(&features);
        self.route_from_index(&delayed_feat)
    }

    fn compute_features(&self, token: &Tensor) -> Tensor {
        token.clone()
    }

    fn compute_routing_index(&self, token: &Tensor) -> RoutingIndex {
        let routed = self.router.route_simple(token);
        RoutingIndex {
            expert_id: routed.first().copied().unwrap_or(0),
            weight: 1.0,
        }
    }

    fn get_delayed_feature(&mut self, token: &Tensor) -> Tensor {
        self.delayed_features.push(token.clone());

        if self.delayed_features.len() > self.routing_delay + 1 {
            self.delayed_features.remove(0);
        }

        let idx = if self.delayed_features.len() > self.routing_delay {
            self.features_index % self.delayed_features.len()
        } else {
            0
        };

        self.features_index += 1;

        self.delayed_features[idx].clone()
    }

    fn route_from_index(&self, features: &Tensor) -> Vec<usize> {
        if let Some(index) = *self.routing_index.read().unwrap() {
            vec![index.expert_id]
        } else {
            self.router.route_simple(features)
        }
    }

    pub fn reset(&mut self) {
        self.loss_spike_detector.reset();
        self.delayed_features.clear();
        self.features_index = 0;
    }
}
