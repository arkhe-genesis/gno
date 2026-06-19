with open('substrato-7001/src/webhooks/polar_handler.rs', 'r') as f:
    content = f.read()

replacement = """pub fn create_webhook_router(
    handler: Arc<PolarWebhookHandler>,
    dlq: DeadLetterQueue,
) -> Router {
    Router::new()
        .route("/webhooks/polar", post(webhook_handler).with_state(handler))
        .route("/webhooks/polar/dlq", axum::routing::get(dlq_handler).with_state(dlq))
        .route("/health", axum::routing::get(health_handler))
}
"""

import re
content = re.sub(r'pub fn create_webhook_router\([\s\S]*?\}', replacement, content)

with open('substrato-7001/src/webhooks/polar_handler.rs', 'w') as f:
    f.write(content)
