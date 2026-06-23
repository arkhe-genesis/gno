pub struct ReputationRouter {
    pub base_score: f64,
}

impl ReputationRouter {
    pub fn new() -> Self {
        Self { base_score: 85.0 }
    }

    pub async fn route(&self, did: &str) -> f64 {
        if did.contains("alpha") {
            95.0
        } else if did.contains("delta") {
            40.0
        } else {
            self.base_score
        }
    }

    pub fn get_thresholds(&self) -> Thresholds {
        Thresholds {
            pro: 90.0,
            plus: 70.0,
            standard: 50.0,
            lite: 0.0,
        }
    }
}

pub struct Thresholds {
    pub pro: f64,
    pub plus: f64,
    pub standard: f64,
    pub lite: f64,
}
