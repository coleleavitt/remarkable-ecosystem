//! Notion API client

use reqwest::{Client, header};
use thiserror::Error;
use serde_json::json;

use super::types::*;

const NOTION_API_URL: &str = "https://api.notion.com/v1";
const NOTION_VERSION: &str = "2022-06-28";

#[derive(Error, Debug)]
pub enum NotionError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("API error: {code} - {message}")]
    Api { code: String, message: String },
    
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Rate limited")]
    RateLimited,
}

/// Notion API client
pub struct NotionClient {
    client: Client,
    config: NotionConfig,
}

impl NotionClient {
    /// Create a new Notion client
    pub fn new(config: NotionConfig) -> Result<Self, NotionError> {
        if config.token.is_empty() {
            return Err(NotionError::Config("API token is required".to_string()));
        }
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Notion-Version",
            header::HeaderValue::from_static(NOTION_VERSION),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", config.token))
                .map_err(|_| NotionError::Config("Invalid token format".to_string()))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        
        let client = Client::builder()
            .default_headers(headers)
            .build()?;
        
        Ok(Self { client, config })
    }
    
    /// Get a page by ID
    pub async fn get_page(&self, page_id: &str) -> Result<NotionPage, NotionError> {
        let url = format!("{}/pages/{}", NOTION_API_URL, page_id);
        
        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }
    
    /// Create a new page
    pub async fn create_page(&self, request: CreatePageRequest) -> Result<NotionPage, NotionError> {
        let url = format!("{}/pages", NOTION_API_URL);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Update a page's properties
    pub async fn update_page(
        &self,
        page_id: &str,
        properties: serde_json::Value,
    ) -> Result<NotionPage, NotionError> {
        let url = format!("{}/pages/{}", NOTION_API_URL, page_id);
        
        let body = json!({
            "properties": properties
        });
        
        let response = self.client
            .patch(&url)
            .json(&body)
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Archive (delete) a page
    pub async fn archive_page(&self, page_id: &str) -> Result<NotionPage, NotionError> {
        let url = format!("{}/pages/{}", NOTION_API_URL, page_id);
        
        let body = json!({
            "archived": true
        });
        
        let response = self.client
            .patch(&url)
            .json(&body)
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Append blocks to a page
    pub async fn append_blocks(
        &self,
        page_id: &str,
        blocks: Vec<NotionBlock>,
    ) -> Result<(), NotionError> {
        let url = format!("{}/blocks/{}/children", NOTION_API_URL, page_id);
        
        let body = json!({
            "children": blocks
        });
        
        let response = self.client
            .patch(&url)
            .json(&body)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let error: serde_json::Value = response.json().await?;
            Err(NotionError::Api {
                code: error["code"].as_str().unwrap_or("unknown").to_string(),
                message: error["message"].as_str().unwrap_or("Unknown error").to_string(),
            })
        }
    }
    
    /// Get blocks from a page
    pub async fn get_blocks(&self, page_id: &str) -> Result<Vec<serde_json::Value>, NotionError> {
        let url = format!("{}/blocks/{}/children", NOTION_API_URL, page_id);
        
        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = self.handle_response(response).await?;
        
        Ok(data["results"].as_array().cloned().unwrap_or_default())
    }
    
    /// Delete all blocks from a page
    pub async fn clear_blocks(&self, page_id: &str) -> Result<(), NotionError> {
        let blocks = self.get_blocks(page_id).await?;
        
        for block in blocks {
            if let Some(block_id) = block["id"].as_str() {
                let url = format!("{}/blocks/{}", NOTION_API_URL, block_id);
                self.client.delete(&url).send().await?;
            }
        }
        
        Ok(())
    }
    
    /// Query a database
    pub async fn query_database(
        &self,
        database_id: &str,
        query: DatabaseQuery,
    ) -> Result<Vec<NotionPage>, NotionError> {
        let url = format!("{}/databases/{}/query", NOTION_API_URL, database_id);
        
        let response = self.client
            .post(&url)
            .json(&query)
            .send()
            .await?;
        
        let data: serde_json::Value = self.handle_response(response).await?;
        
        let pages: Vec<NotionPage> = data["results"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(pages)
    }
    
    /// Upload a file (returns external URL - Notion doesn't support direct uploads via API)
    /// In practice, you'd upload to S3/Cloudflare and use that URL
    pub fn create_image_url(&self, svg_data_url: &str) -> ImageSource {
        // For now, use external URL
        // In production, upload to a hosting service
        ImageSource::External {
            external: ExternalUrl {
                url: svg_data_url.to_string(),
            },
        }
    }
    
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, NotionError> {
        let status = response.status();
        
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(NotionError::RateLimited);
        }
        
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(NotionError::NotFound("Resource not found".to_string()));
        }
        
        if !status.is_success() {
            let error: serde_json::Value = response.json().await?;
            return Err(NotionError::Api {
                code: error["code"].as_str().unwrap_or("unknown").to_string(),
                message: error["message"].as_str().unwrap_or("Unknown error").to_string(),
            });
        }
        
        let data: T = response.json().await?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        let config = NotionConfig::new(String::new());
        let result = NotionClient::new(config);
        assert!(matches!(result, Err(NotionError::Config(_))));
    }
}
