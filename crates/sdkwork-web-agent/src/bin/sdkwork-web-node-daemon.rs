#[tokio::main]
async fn main() -> anyhow::Result<()> {
    sdkwork_web_node_daemon::run().await
}
