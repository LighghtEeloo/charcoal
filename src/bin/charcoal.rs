use charcoal_dict::App;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    App::main().await
}
