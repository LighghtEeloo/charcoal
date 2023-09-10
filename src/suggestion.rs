pub struct Suggestion {
    pub word: String,
}

impl Suggestion {
    pub fn new(word: String) -> Self {
        Self { word }
    }
    pub fn exec(self) -> anyhow::Result<()> {
        if which::which("dym").is_ok() {
            println!("Word not found, but..");
            let mut cmd = std::process::Command::new("dym");
            cmd.arg(self.word);
            cmd.spawn()?;
            std::thread::sleep(std::time::Duration::from_millis(700));
        } else {
            println!("Word not found.");
        }
        Ok(())
    }
}
