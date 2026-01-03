# OCR-eg

**OCR-eg** 是一款基于 Rust 开发的OCR 客户端，方便您使用[Mistral AI OCR API](https://mistral.ai/news/mistral-ocr/) 处理文本

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-orange.svg)
![Rust](https://img.shields.io/badge/built%20with-Rust-red.svg)


##  快速开始

### 方式一：下载现成版本 (推荐)
前往 [Releases](https://github.com/daydalek/OCR-eg/releases) 页面，下载对应您系统的压缩包，解压后即可运行。

### 方式二：手动编译
确保您已安装 [Rust 环境](https://rustup.rs/)：

```bash
git clone https://github.com/daydalek/OCR-eg.git
cd OCR-eg
cargo build --release
```
编译产物位于 `target/release/` 目录下。

## 使用步骤

1. **配置引擎**：首次运行点击“设置 API Key”，填入您的 Mistral AI API Key。
2. **导入文件**：直接将 PDF 或图片拖入软件窗口，或点击添加。
3. **一键识别**：点击“开始处理”，软件将自动完成上传、识别、下载及 Markdown 合并。
4. **管理结果**：处理完成后，点击“浏览结果”即可查看生成的 Markdown 文件和提取的图片。


## 免责声明

这是一个个人开发的开源项目，并非 Mistral AI 官方产品。使用本软件需要您自行承担 Mistral API 产生的相关费用，或者您也可以申请Mistral AI的Free Tier API来使用。

