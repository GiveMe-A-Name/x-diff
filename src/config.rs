use anyhow::{anyhow, Context, Ok, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write};
use tokio::{fs, join};

use crate::{diff_text, ExtraArgs, RequestProfile};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>,
}

impl DiffConfig {
    pub fn from_yaml(content: &str) -> Result<Self> {
        let config: Self = serde_yaml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }

    pub async fn load_yaml(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    pub fn from_json(content: &str) -> Result<Self> {
        let config: Self = serde_json::from_str(content)?;
        config.validate()?;
        Ok(config)
    }

    pub async fn load_json(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        Self::from_json(&content)
    }

    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }

    fn validate(&self) -> Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .context(format!("failed to validate profile `{}`", name.to_string()))?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffProfile {
    pub request_first: RequestProfile,
    pub request_second: RequestProfile,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub response: Option<ResponseProfile>,
}

impl DiffProfile {
    pub fn new(
        req1: RequestProfile,
        req2: RequestProfile,
        response_profile: ResponseProfile,
    ) -> Self {
        Self {
            request_first: req1,
            request_second: req2,
            response: Some(response_profile),
        }
    }

    pub async fn diff(&self, args: ExtraArgs) -> Result<()> {
        let (res1, res2) = {
            let (res1, res2) = join!(
                self.request_first.send(&args),
                self.request_second.send(&args),
            );
            (res1?, res2?)
        };

        let (text1, text2) = {
            let (txt1, txt2) = join!(
                res1.filter_text(&self.response),
                res2.filter_text(&self.response)
            );
            (txt1?, txt2?)
        };
        let result = diff_text(text1, text2)?;
        let mut stdout = std::io::stdout().lock();
        stdout.write(result.as_bytes()).map_err(|e| anyhow!(e))?;
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        self.request_first
            .validate()
            .context("request_first failed to validate")?;
        self.request_second
            .validate()
            .context("request_second config parser failed")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}

impl ResponseProfile {
    pub fn new(skip_headers: Vec<String>, skip_body: Vec<String>) -> Self {
        Self {
            skip_headers,
            skip_body,
        }
    }
}
