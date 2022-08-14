use char_coal::App;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    App::main().await
}
