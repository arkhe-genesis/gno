sed -i 's/let event_type = payload\["type"\].as_str().unwrap_or("unknown");/let event_type = payload["type"].as_str().unwrap_or("unknown").to_string();/g' substrato-7001/src/webhooks/polar_handler.rs
sed -i 's/handler_clone.process_with_retry(event_type, data).await;/handler_clone.process_with_retry(\&event_type, data).await;/g' substrato-7001/src/webhooks/polar_handler.rs

cat << 'INNER_EOF' > patch_router.py
import re
with open('substrato-7001/src/webhooks/polar_handler.rs', 'r') as f:
    content = f.read()

content = content.replace('.route("/webhooks/polar/dlq", axum::routing::get(dlq_handler))', '.route("/webhooks/polar/dlq", axum::routing::get(dlq_handler).with_state(dlq))')

content = content.replace('.with_state(dlq)', '')

with open('substrato-7001/src/webhooks/polar_handler.rs', 'w') as f:
    f.write(content)
INNER_EOF
python3 patch_router.py
