use crate::Config;

async fn __speak(word: impl AsRef<str>) -> anyhow::Result<()> {
    // let mut ttss = tts::Tts::default()?;
    // let tts::Features { is_speaking, .. } = ttss.supported_features();
    // if is_speaking {
    //     println!("Are we speaking? {}", ttss.is_speaking()?);
    // }
    // ttss.speak(word.word, false)?;

    let voice = google_speech::Speech::new(word, google_speech::Lang::en_us)?;
    voice.play()?;
    Ok(())
}

pub struct Speech<'a> {
    config: &'a Config,
}

impl<'a> Speech<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
    pub async fn speak(&self, word: impl AsRef<str>) -> anyhow::Result<()> {
        if self.config.speak {
            __speak(word).await
        } else {
            Ok(())
        }
    }
}
