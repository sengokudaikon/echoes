use echoes_core::run;

fn main() -> anyhow::Result<()> {
    run().map_err(|e| anyhow::anyhow!("{}", e))
}
