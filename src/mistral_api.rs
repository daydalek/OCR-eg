use serde::{Deserialize, Serialize};
use anyhow::Result;
use reqwest::{Client, multipart};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileResponse {
    pub id: String,
    pub object: String,
    pub bytes: u64,
    pub created_at: u64,
    pub filename: String,
    pub purpose: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedUrlResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OCRImage {
    pub id: String,
    #[serde(rename = "image_base64")]
    pub image_base64: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OCRPage {
    pub index: u32,
    pub markdown: String,
    pub images: Vec<OCRImage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OCRResponse {
    pub pages: Vec<OCRPage>,
    pub model: String,
    pub usage_info: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct OCRDocumentUrl {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub document_url: String,
}

#[derive(Debug, Serialize)]
pub struct OCRRequest {
    pub model: String,
    pub document: OCRDocumentUrl,
    pub include_image_base64: bool,
}

pub struct MistralClient {
    client: Client,
    api_key: String,
}

impl MistralClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn upload_file<P: AsRef<Path>>(&self, path: P) -> Result<FileResponse> {
        let path = path.as_ref();
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file.pdf")
            .to_string();
        
        let file_content = tokio::fs::read(path).await?;
        let part = multipart::Part::bytes(file_content)
            .file_name(filename);
        
        let form = multipart::Form::new()
            .part("file", part)
            .text("purpose", "ocr");
        
        let response = self.client.post("https://api.mistral.ai/v1/files")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Upload failed: {}", error_text));
        }
        
        Ok(response.json().await?)
    }

    pub async fn get_signed_url(&self, file_id: &str) -> Result<String> {
        let response = self.client.get(format!("https://api.mistral.ai/v1/files/{}/url", file_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Get signed URL failed: {}", error_text));
        }
        
        let res: SignedUrlResponse = response.json().await?;
        Ok(res.url)
    }

    pub async fn process_ocr(&self, document_url: String) -> Result<OCRResponse> {
        let request = OCRRequest {
            model: "mistral-ocr-latest".to_string(),
            document: OCRDocumentUrl {
                doc_type: "document_url".to_string(),
                document_url,
            },
            include_image_base64: true,
        };
        
        let response = self.client.post("https://api.mistral.ai/v1/ocr")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OCR processing failed: {}", error_text));
        }
        
        Ok(response.json().await?)
    }
}
