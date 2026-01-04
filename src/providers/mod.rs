use async_trait::async_trait;
use anyhow::Result;
use std::path::Path;

pub mod mistral;

// 统一的图像结构
#[derive(Debug, Clone)]
pub struct OcrImage {
    pub id: String,
    pub base64: String,
}

// 统一的页面结构
#[derive(Debug, Clone)]
pub struct OcrPage {
    pub number: usize,
    pub markdown: String,
    pub images: Vec<OcrImage>,
}

// 统一的结果结构
#[derive(Debug, Clone)]
pub struct OcrResult {
    pub pages: Vec<OcrPage>,
}

// 核心接口：所有 OCR 供应商都必须实现这个 Trait
#[async_trait]
pub trait OcrProvider: Send + Sync {
    // 获取供应商的唯一 ID (如 "mistral", "openai")
    fn id(&self) -> &str;
    
    // 获取显示名称 (如 "Mistral AI")
    fn name(&self) -> &str;

    // 处理单个文件，返回标准化的结果
    async fn process_file(&self, file_path: &Path) -> Result<OcrResult>;
}
