import re
with open('substrato-7001/src/webhooks/polar_handler.rs', 'r') as f:
    content = f.read()

content = content.replace('''pub fn create_webhook_router(
    handler: Arc<PolarWebhookHandler>,
    dlq: DeadLetterQueue,
) -> Router {
    Router::new()
        .route("/webhooks/polar", post(webhook_handler))
        .route("/webhooks/polar/dlq", axum::routing::get(dlq_handler))
        .route("/health", axum::routing::get(health_handler))
        .with_state(handler)

}''', '''pub fn create_webhook_router(
    handler: Arc<PolarWebhookHandler>,
    dlq: DeadLetterQueue,
) -> Router {
    Router::new()
        .route("/webhooks/polar", post(webhook_handler).with_state(handler))
        .route("/webhooks/polar/dlq", axum::routing::get(dlq_handler).with_state(dlq))
        .route("/health", axum::routing::get(health_handler))
}''')

with open('substrato-7001/src/webhooks/polar_handler.rs', 'w') as f:
    f.write(content)
