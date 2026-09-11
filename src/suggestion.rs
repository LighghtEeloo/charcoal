pub struct Suggestion {
    pub word: String,
}

impl Suggestion {
    pub fn new(word: String) -> Self {
        Self { word }
    }
    pub async fn exec(self) -> anyhow::Result<()> {
        if which::which("dym").is_ok() {
            println!("Word not found, but..");
            let mut cmd = tokio::process::Command::new("dym");
            cmd.arg(self.word);
            let status = cmd.status().await?;
            anyhow::ensure!(status.success(), "Suggestion command exited with {status}");
        } else {
            println!("Word not found.");
        }
        Ok(())
    }
}
