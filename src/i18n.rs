use std::collections::HashMap;

#[derive(Clone)]
pub struct I18n {
    current_lang: String,
    translations: HashMap<String, HashMap<String, String>>,
}

impl I18n {
    pub fn new(lang: &str) -> Self {
        let mut translations = HashMap::new();
        
        let mut zh = HashMap::new();
        zh.insert("app_title".into(), "OCR-eg ".into());
        zh.insert("header_title".into(), "OCR-eg".into());
        zh.insert("language".into(), "语言".into());
        zh.insert("drop_area_hint".into(), "拖放PDF或图像文件到此处，或点击选择".into());
        zh.insert("queue_label".into(), "任务队列".into());
        zh.insert("add_files".into(), "添加文件".into());
        zh.insert("remove_selected".into(), "移除选中".into());
        zh.insert("clear_queue".into(), "清空队列".into());
        zh.insert("output_settings".into(), "输出设置".into());
        zh.insert("save_location".into(), "保存位置".into());
        zh.insert("browse_button".into(), "浏览".into());
        zh.insert("progress_label".into(), "处理进度".into());
        zh.insert("total_progress".into(), "总体进度".into());
        zh.insert("current_file".into(), "当前文件".into());
        zh.insert("ready".into(), "准备就绪".into());
        zh.insert("start_process".into(), "开始处理".into());
        zh.insert("set_api_key".into(), "设置 API Key".into());
        zh.insert("browse_results".into(), "浏览结果".into());
        zh.insert("copyright".into(), "".into());
        zh.insert("api_key_title".into(), "设置 Mistral API Key".into());
        zh.insert("api_key_prompt".into(), "请输入您的 Mistral API Key (用于驱动识别引擎)".into());
        zh.insert("no_api_key".into(), "还没有 API Key?".into());
        zh.insert("apply_here".into(), "由此申请".into());
        zh.insert("api_activation_note".into(), "请确保您的 API 已激活且有余额".into());
        zh.insert("api_security_note".into(), "您的 Key 将保存在本地配置文件中".into());
        zh.insert("save".into(), "保存".into());
        zh.insert("cancel".into(), "取消".into());
        zh.insert("show".into(), "显示".into());
        zh.insert("hide".into(), "隐藏".into());
        zh.insert("ocr_result_dir".into(), "ocr_结果_".into());
        zh.insert("success_all_files_done".into(), "所有文件处理完成！".into());
        
        translations.insert("zh_CN".into(), zh);
        
        let mut en = HashMap::new();
        en.insert("app_title".into(), "OCR-eg".into());
        en.insert("header_title".into(), "OCR-eg".into());
        en.insert("language".into(), "Language".into());
        en.insert("drop_area_hint".into(), "Drop PDF or image files here, or click to select".into());
        en.insert("queue_label".into(), "Task Queue".into());
        en.insert("add_files".into(), "Add Files".into());
        en.insert("remove_selected".into(), "Remove Selected".into());
        en.insert("clear_queue".into(), "Clear Queue".into());
        en.insert("output_settings".into(), "Output Settings".into());
        en.insert("save_location".into(), "Save Location".into());
        en.insert("browse_button".into(), "Browse".into());
        en.insert("progress_label".into(), "Progress".into());
        en.insert("total_progress".into(), "Total Progress".into());
        en.insert("current_file".into(), "Current File".into());
        en.insert("ready".into(), "Ready".into());
        en.insert("start_process".into(), "Start Processing".into());
        en.insert("set_api_key".into(), "Set API Key".into());
        en.insert("browse_results".into(), "Browse Results".into());
        en.insert("copyright".into(), "".into());
        en.insert("api_key_title".into(), "Set Mistral API Key".into());
        en.insert("api_key_prompt".into(), "Please enter your Mistral API Key (to power the engine)".into());
        en.insert("no_api_key".into(), "Don't have an API Key?".into());
        en.insert("apply_here".into(), "Apply here".into());
        en.insert("api_activation_note".into(), "Ensure your API is activated and has balance".into());
        en.insert("api_security_note".into(), "Your key will be saved locally".into());
        en.insert("save".into(), "Save".into());
        en.insert("cancel".into(), "Cancel".into());
        en.insert("show".into(), "Show".into());
        en.insert("hide".into(), "Hide".into());
        en.insert("ocr_result_dir".into(), "lumi_ocr_results_".into());
        en.insert("success_all_files_done".into(), "All files processed successfully!".into());
        
        translations.insert("en_US".into(), en);
        
        Self {
            current_lang: lang.to_string(),
            translations,
        }
    }
    
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.translations.get(&self.current_lang)
            .and_then(|m| m.get(key))
            .map(|s| s.as_str())
            .unwrap_or(key)
    }
    
    pub fn set_lang(&mut self, lang: &str) {
        self.current_lang = lang.to_string();
    }
}
