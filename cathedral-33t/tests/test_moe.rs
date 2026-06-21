use cathedral_arkhe_33t::moe::{MoELayer, HierarchicalRouter};
use cathedral_arkhe_33t::config::MoEConfig;
use cathedral_arkhe_33t::tensor::Tensor;

#[test]
fn test_moe_layer_forward() {
    let config = MoEConfig {
        num_experts: 16,
        top_k: 4,
        hidden_size: 8,
        intermediate_size: 32,
        capacity_factor: 1.25,
        load_balancing_loss_coef: 0.01,
    };
    let mut moe = MoELayer::new(&config);
    let x = Tensor::randn(&[2, 8]);
    let (output, aux_loss) = moe.forward(&x);
    assert_eq!(output.shape(), vec![2, 8]);
    assert!(aux_loss >= 0.0);
}

#[test]
fn test_hierarchical_router_route() {
    let router = HierarchicalRouter::new(4096, 8, 16);
    let x = Tensor::randn(&[2, 16]);
    let routing = router.route(&x);
    assert_eq!(routing.len(), 2);
    assert_eq!(routing[0].len(), 8);
    assert_eq!(routing[1].len(), 8);
}
