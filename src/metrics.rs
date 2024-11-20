use anyhow::Context;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;

pub fn spawn_metrics_client(prometheus_addr: SocketAddr) -> anyhow::Result<()> {
    PrometheusBuilder::new()
        .with_http_listener(prometheus_addr)
        .install()
        .context("cannot spawn prometheus client")
}
