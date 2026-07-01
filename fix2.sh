cat << 'INNER_EOF' > patch_metrics.py
with open('substrato-7001/src/metrics_exporter.rs', 'r') as f:
    content = f.read()

content = content.replace('.with_http_listener(format!("0.0.0.0:{}", port));', '.with_http_listener(format!("0.0.0.0:{}", port).parse::<std::net::SocketAddr>().unwrap());')

with open('substrato-7001/src/metrics_exporter.rs', 'w') as f:
    f.write(content)
INNER_EOF
python3 patch_metrics.py
