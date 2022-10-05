use anyhow::{Ok, Result};
use reqwest::{
    header::{self, HeaderMap},
    Response,
};

use crate::ResponseProfile;

pub struct ResponseExt(pub Response);

impl ResponseExt {
    pub async fn filter_text(self, profile: &Option<ResponseProfile>) -> Result<String> {
        let response = self.0;
        let mut output = format!("{:?} {}\n", response.version(), response.status());
        let headers = response.headers();
        if let Some(profile) = profile {
            for (k, v) in headers.iter() {
                if !profile.skip_headers.iter().any(|sh| sh == k.as_str()) {
                    output.push_str(&format!("{}: {:?}\n", k, v));
                }
            }
        }
        let content_type = get_content_type(headers);
        let text = response.text().await?;
        match content_type.as_deref() {
            Some("application/json") => {
                let text = filter_json(&text, profile)?;
                output.push_str(&text);
            }
            _ => output.push_str(&text),
        }

        Ok(output)
    }

    pub fn get_header_keys(&self) -> Vec<String> {
        let header_keys = self.0.headers().keys();
        header_keys
            .into_iter()
            .map(|key| key.as_str().to_string())
            .collect()
    }
}

fn filter_json(text: &str, profile: &Option<ResponseProfile>) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(text)?;
    if let Some(profile) = profile {
        for k in profile.skip_body.iter() {
            json[k] = serde_json::json!(null);
        }
    }
    let json_text = serde_json::to_string_pretty(&json)?;
    Ok(json_text)
}

fn get_content_type(headers: &HeaderMap) -> Option<String> {
    // the flatten function can flatten the Option<Option<T>> => Option<T>
    headers
        .get(header::CONTENT_TYPE)
        // the Content-Type always be `application/json; UTF-8`
        // so split by `;`
        .and_then(|value| {
            value
                .to_str()
                .unwrap()
                .split(';')
                .next()
                .map(|str| str.to_owned())
        })
}
