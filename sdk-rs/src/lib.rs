pub mod pb {
    tonic::include_proto!("cathedral.v1");
}

pub mod prometheus;
pub mod event;

pub use prometheus::PrometheusAdapter;
pub use event::PrometheusEvent;
