use std::collections::HashMap;

use reqwest::{header::HeaderMap, Response};
use serde_json::Value;

pub struct Post {}

impl Post {
    pub fn new() -> Post {
        Post {}
    }

    pub async fn post(
        &self,
        url: &str,
        headers: HeaderMap,
        data: String,
    ) -> Result<Response, reqwest::Error> {
        let client = reqwest::Client::new();

        Ok(client
            .post(url)
            .headers(headers)
            .body(data)
            .send()
            .await?)
    }

    pub async fn get(&self, url: &str) -> Result<Response, reqwest::Error> {
        Ok(reqwest::get(url).await?)
    }
}
