use echoes_core::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await.map_err(|e| anyhow::anyhow!("{}", e))
}
