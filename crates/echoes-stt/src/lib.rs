pub mod openai;
pub mod whisper;

use anyhow::Result;

pub use openai::OpenAiStt;
#[allow(unused_imports)]
pub use whisper::LocalWhisperStt;

pub trait SttProvider {
    #[allow(async_fn_in_trait)]
    async fn transcribe(&self, audio_data: Vec<u8>) -> Result<String>;
}
