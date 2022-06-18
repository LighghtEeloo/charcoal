use char_coal::app;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    app::main().await
}
