use bytes::Bytes;

use reqwest::{header::CONTENT_TYPE, Url};

pub struct Voicevox {
    host: Url,
    client: reqwest::Client,
}

impl Voicevox {
    pub fn new(host: Url) -> Self {
        let client = reqwest::Client::new();
        Voicevox { host, client }
    }

    pub async fn tts(&self, text: &str, speaker_id: u32) -> Bytes {
        let mut url: Url = self.host.clone();
        url.path_segments_mut().unwrap().push("audio_query");
        url.query_pairs_mut()
            .clear()
            .append_pair("text", text)
            .append_pair("speaker", &speaker_id.to_string());

        let resp = self.client.post(url).send().await.unwrap();
        let query_text = resp.text().await.unwrap();

        let mut url = self.host.clone();
        url.path_segments_mut().unwrap().push("synthesis");
        url.query_pairs_mut()
            .clear()
            .append_pair("speaker", &speaker_id.to_string());

        let resp = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .body(query_text)
            .send()
            .await
            .unwrap();

        resp.bytes().await.unwrap()
    }
}
