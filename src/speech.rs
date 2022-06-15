use crate::{Config, Toggle};

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

pub async fn speak(word: impl AsRef<str>, config: &Config) -> anyhow::Result<()> {
    if config.check(Toggle::WithSpeech) {
        __speak(word).await
    } else {
        Ok(())
    }
}
