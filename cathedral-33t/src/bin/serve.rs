use cathedral_arkhe_33t::CathedralConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let _config = CathedralConfig::default();

    Ok(())
}
