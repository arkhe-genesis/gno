import re
with open('substrato-7001/src/webhooks/polar_handler.rs', 'r') as f:
    content = f.read()

content = content.replace('.route("/webhooks/polar/dlq", axum::routing::get(dlq_handler))', '.route("/webhooks/polar/dlq", axum::routing::get(dlq_handler).with_state(dlq))')

content = content.replace('.with_state(dlq)', '')

with open('substrato-7001/src/webhooks/polar_handler.rs', 'w') as f:
    f.write(content)
