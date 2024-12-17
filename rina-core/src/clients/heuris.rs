use anyhow;
use reqwest;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
pub struct HeurisClient {
    api_key: String,
}

impl HeurisClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn generate_image(&self, image_prompt: String) -> Result<Vec<u8>, anyhow::Error> {
        let client = reqwest::Client::builder().build()?;
        let deadline = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 300;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", self.api_key).parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        let body = json!({
            "model_input": {
                "SD": {
                    "width": 512,
                    "height": 768,
                    "prompt": image_prompt,
                    "neg_prompt": "worst quality, bad quality, umbrella, blurry face, anime, illustration",
                    "num_iterations": 50,
                    "guidance_scale": 7.5
                }
            },
            "model_id": "BluePencilRealistic",
            "deadline": deadline,
            "priority": 1,
            "job_id": format!("job_{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())
        });

        let request = client
            .request(
                reqwest::Method::POST,
                "http://sequencer.heurist.xyz/submit_job",
            )
            .headers(headers)
            .json(&body);

        let response = request.send().await?;
        let image_url = response.text().await?.trim_matches('"').to_string();
        
        self.prepare_image_for_tweet(&image_url).await
    }

    pub async fn prepare_image_for_tweet(&self, image_url: &str) -> Result<Vec<u8>, anyhow::Error> {
        let client = reqwest::Client::new();
        let response = client.get(image_url).send().await?;

        Ok(response.bytes().await?.to_vec())
    }
}
