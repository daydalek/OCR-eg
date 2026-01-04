use async_trait::async_trait;
use anyhow::Result;
use reqwest::{Client, multipart};
use std::path::Path;
use serde::{Deserialize, Serialize};
use super::{OcrProvider, OcrResult, OcrPage, OcrImage};

// --- Mistral API 特定的数据结构 (内部使用) ---
#[derive(Debug, Serialize, Deserialize)]
struct FileResponse {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SignedUrlResponse {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralImage {
    id: String,
    #[serde(rename = "image_base64")]
    image_base64: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralPage {
    index: u32,
    markdown: String,
    images: Vec<MistralImage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralResponse {
    pages: Vec<MistralPage>,
}

#[derive(Debug, Serialize)]
struct DocumentUrl {
    #[serde(rename = "type")]
    doc_type: String,
    document_url: String,
}

#[derive(Debug, Serialize)]
struct OcrRequest {
    model: String,
    document: DocumentUrl,
    include_image_base64: bool,
}

// --- Provider 实现 ---

pub struct MistralProvider {
    client: Client,
    api_key: String,
}

impl MistralProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    async fn upload_file(&self, path: &Path) -> Result<FileResponse> {
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
            return Err(anyhow::anyhow!("Mistral Upload failed: {}", error_text));
        }
        
        Ok(response.json().await?)
    }

    async fn get_signed_url(&self, file_id: &str) -> Result<String> {
        let response = self.client.get(format!("https://api.mistral.ai/v1/files/{}/url", file_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Mistral Signed URL failed: {}", error_text));
        }
        
        let res: SignedUrlResponse = response.json().await?;
        Ok(res.url)
    }

    async fn call_ocr_api(&self, document_url: String) -> Result<MistralResponse> {
        let request = OcrRequest {
            model: "mistral-ocr-latest".to_string(),
            document: DocumentUrl {
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
            return Err(anyhow::anyhow!("Mistral OCR failed: {}", error_text));
        }
        
        Ok(response.json().await?)
    }
}

#[async_trait]
impl OcrProvider for MistralProvider {
    fn id(&self) -> &str {
        "mistral"
    }

    fn name(&self) -> &str {
        "Mistral AI"
    }

    async fn process_file(&self, file_path: &Path) -> Result<OcrResult> {
        // 1. Upload
        let file_res = self.upload_file(file_path).await?;
        
        // 2. Get URL
        let url = self.get_signed_url(&file_res.id).await?;
        
        // 3. Process
        let mistral_res = self.call_ocr_api(url).await?;
        
        // 4. Convert to Standard Result
        let mut pages = Vec::new();
        for p in mistral_res.pages {
            let mut images = Vec::new();
            for img in p.images {
                if let Some(b64) = img.image_base64 {
                    // Normalize base64 string (remove data prefix if present)
                    let clean_b64 = if b64.contains(",") {
                        b64.split(',').nth(1).unwrap_or("").to_string()
                    } else {
                        b64
                    };
                    
                    images.push(OcrImage {
                        id: img.id,
                        base64: clean_b64,
                    });
                }
            }
            
            pages.push(OcrPage {
                number: p.index as usize,
                markdown: p.markdown,
                images,
            });
        }
        
        Ok(OcrResult { pages })
    }
}
