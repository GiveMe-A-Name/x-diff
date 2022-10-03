use anyhow::{anyhow, Ok, Result};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method, Url,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::FromStr;

use crate::{ExtraArgs, ResponseExt};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestProfile {
    // TODO：known about http_serde
    #[serde(with = "http_serde::method", default)]
    // TODO：known about reqwest Method, Url,
    pub method: Method,
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        with = "http_serde::header_map",
        default
    )]
    pub headers: HeaderMap,
    pub body: Option<serde_json::Value>,
}

impl RequestProfile {
    pub async fn send(&self, args: &ExtraArgs) -> Result<ResponseExt> {
        let (headers, query, body) = self.generate(args)?;
        let request = Client::new().request(self.method.clone(), self.url.clone());
        let body = serde_json::to_string(&body)?;
        let response = request
            .headers(headers)
            .query(&query)
            .body(body)
            .send()
            .await
            .map_err(|err| anyhow!(err))?;
        Ok(ResponseExt(response))
    }

    fn generate(&self, args: &ExtraArgs) -> Result<(HeaderMap, Value, String)> {
        let (mut headers, mut query, mut body) = (
            self.headers.clone(),
            self.params.clone().unwrap_or(json!({})),
            self.body.clone().unwrap_or(json!({})),
        );

        // for (k, v) in &args.headers {
        // FIXME: borrowed data escapes outside of associated function`args`, escapes the associated function body here
        // TODO: known the headers.insert
        //   headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
        // }

        let extra_headers: Vec<(HeaderName, HeaderValue)> = args
            .headers
            .iter()
            .map(|(k, v)| Ok((HeaderName::from_str(k)?, HeaderValue::from_str(v)?)))
            .try_collect()?;
        headers.extend(extra_headers);

        for (k, v) in &args.query {
            query[k] = v.parse()?;
        }

        for (k, v) in &args.body {
            body[k] = v.parse()?;
        }

        let body = serde_json::to_string(&body)?;

        Ok((headers, query, body))
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(params) = self.params.as_ref() {
            if !params.is_object() {
                return Err(anyhow!(
                    "params must be an object but got - {}",
                    serde_yaml::to_string(&params)?,
                ));
            }
        }
        if let Some(body) = self.body.as_ref() {
            if !body.is_object() {
                return Err(anyhow!(
                    "params must be an object but got - {}",
                    serde_yaml::to_string(&body)?,
                ));
            }
        }
        Ok(())
    }
}
