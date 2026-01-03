mod mistral_api;
mod pdf_utils;
mod config;
mod i18n;

use std::path::{Path, PathBuf};
use eframe::egui;
use mistral_api::MistralClient;
use config::{AppConfig, load_config, save_config};
use i18n::I18n;
use tokio::sync::mpsc;
use base64::{engine::general_purpose, Engine as _};

struct AppState {
    config: AppConfig,
    i18n: I18n,
    file_queue: Vec<PathBuf>,
    output_path: PathBuf,
    total_progress: f32,
    current_file_progress: f32,
    status_message: String,
    is_processing: bool,
    show_api_modal: bool,
    temp_api_key: String,
    show_key: bool,
    last_output_dirs: Vec<PathBuf>,
    receiver: Option<mpsc::Receiver<ProgressUpdate>>,
}

impl AppState {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let config = load_config();
        let i18n = I18n::new(&config.language);
        let output_path = std::env::current_dir().unwrap_or_default();
        let status_message = i18n.t("ready").to_string();
        
        Self {
            config,
            i18n,
            file_queue: Vec::new(),
            output_path,
            total_progress: 0.0,
            current_file_progress: 0.0,
            status_message,
            is_processing: false,
            show_api_modal: false,
            temp_api_key: String::new(),
            show_key: false,
            last_output_dirs: Vec::new(),
            receiver: None,
        }
    }
}

enum ProgressUpdate {
    Total(f32),
    Current(f32),
    Message(String),
    Finished(Vec<PathBuf>),
    Error(String),
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut finished_dirs = None;
        let mut error_msg = None;

        if let Some(ref mut rx) = self.receiver {
            while let Ok(update) = rx.try_recv() {
                match update {
                    ProgressUpdate::Total(p) => self.total_progress = p,
                    ProgressUpdate::Current(p) => self.current_file_progress = p,
                    ProgressUpdate::Message(m) => self.status_message = m,
                    ProgressUpdate::Finished(dirs) => {
                        finished_dirs = Some(dirs);
                    }
                    ProgressUpdate::Error(e) => {
                        error_msg = Some(e);
                    }
                }
            }
        }

        if let Some(dirs) = finished_dirs {
            self.last_output_dirs = dirs;
            self.is_processing = false;
            self.status_message = self.i18n.t("success_all_files_done").to_string();
            self.receiver = None;
        }

        if let Some(e) = error_msg {
            self.status_message = e;
            self.is_processing = false;
            self.receiver = None;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_header(ui);
            ui.add_space(10.0);
            
            self.render_drop_area(ui);
            ui.add_space(10.0);
            
            self.render_queue(ui);
            ui.add_space(10.0);
            
            self.render_output_settings(ui);
            ui.add_space(10.0);
            
            self.render_progress(ui);
            ui.add_space(10.0);
            
            self.render_buttons(ui);
        });

        if self.show_api_modal {
            self.render_api_modal(ctx);
        }
        
        if self.is_processing {
            ctx.request_repaint();
        }
    }
}

impl AppState {
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(self.i18n.t("header_title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut lang = self.config.language.clone();
                let mut changed = false;
                egui::ComboBox::from_label(self.i18n.t("language"))
                    .selected_text(if lang == "zh_CN" { "简体中文" } else { "English" })
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut lang, "zh_CN".into(), "简体中文").clicked() { changed = true; }
                        if ui.selectable_value(&mut lang, "en_US".into(), "English").clicked() { changed = true; }
                    });
                
                if changed && lang != self.config.language {
                    self.config.language = lang;
                    self.i18n.set_lang(&self.config.language);
                    let _ = save_config(&self.config);
                }
            });
        });
        ui.separator();
    }

    fn render_drop_area(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .rounding(5.0)
            .inner_margin(20.0);

        frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                if ui.button(self.i18n.t("drop_area_hint")).clicked() {
                    if let Some(files) = rfd::FileDialog::new()
                        .add_filter("Supported files", &["pdf", "jpg", "jpeg", "png", "bmp", "tiff", "tif"])
                        .pick_files() {
                        for file in files {
                            if !self.file_queue.contains(&file) {
                                self.file_queue.push(file);
                            }
                        }
                    }
                }
                ui.label("📄➡️📋");
            });
        });
        
        let dropped_files = ui.input(|i| i.raw.dropped_files.clone());
        for file in dropped_files {
            if let Some(path) = file.path {
                if !self.file_queue.contains(&path) {
                    self.file_queue.push(path);
                }
            }
        }
    }

    fn render_queue(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(self.i18n.t("queue_label"));
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                let mut to_remove = None;
                for (i, file) in self.file_queue.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(file.file_name().unwrap_or_default().to_string_lossy());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    });
                }
                if let Some(i) = to_remove {
                    self.file_queue.remove(i);
                }
            });
            
            ui.horizontal(|ui| {
                if ui.button(self.i18n.t("add_files")).clicked() {
                     if let Some(files) = rfd::FileDialog::new()
                        .add_filter("Supported files", &["pdf", "jpg", "jpeg", "png", "bmp", "tiff", "tif"])
                        .pick_files() {
                        for file in files {
                            if !self.file_queue.contains(&file) {
                                self.file_queue.push(file);
                            }
                        }
                    }
                }
                if ui.button(self.i18n.t("clear_queue")).clicked() {
                    self.file_queue.clear();
                }
            });
        });
    }

    fn render_output_settings(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(self.i18n.t("output_settings"));
            ui.horizontal(|ui| {
                ui.label(self.i18n.t("save_location"));
                let mut path_str = self.output_path.to_string_lossy().to_string();
                if ui.text_edit_singleline(&mut path_str).changed() {
                    self.output_path = PathBuf::from(path_str);
                }
                if ui.button(self.i18n.t("browse_button")).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.output_path = path;
                    }
                }
            });
        });
    }

    fn render_progress(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(self.i18n.t("progress_label"));
            ui.label(self.i18n.t("total_progress"));
            ui.add(egui::ProgressBar::new(self.total_progress).show_percentage());
            ui.label(self.i18n.t("current_file"));
            ui.add(egui::ProgressBar::new(self.current_file_progress).show_percentage());
            ui.label(&self.status_message);
        });
    }

    fn render_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let start_btn = ui.add_enabled(!self.is_processing && !self.file_queue.is_empty(), egui::Button::new(self.i18n.t("start_process")));
            if start_btn.clicked() {
                if self.config.api_key.is_none() {
                    self.show_api_modal = true;
                } else {
                    self.start_processing(ui.ctx().clone());
                }
            }

            if ui.button(self.i18n.t("set_api_key")).clicked() {
                self.temp_api_key = self.config.api_key.clone().unwrap_or_default();
                self.show_api_modal = true;
            }

            let browse_btn = ui.add_enabled(!self.last_output_dirs.is_empty(), egui::Button::new(self.i18n.t("browse_results")));
            if browse_btn.clicked() {
                for dir in &self.last_output_dirs {
                    let _ = open::that(dir);
                }
            }
        });
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.label(self.i18n.t("copyright"));
        });
    }

    fn render_api_modal(&mut self, ctx: &egui::Context) {
        egui::Window::new(self.i18n.t("api_key_title"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(self.i18n.t("api_key_prompt"));
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.temp_api_key).password(!self.show_key));
                    if ui.button(if self.show_key { self.i18n.t("hide") } else { self.i18n.t("show") }).clicked() {
                        self.show_key = !self.show_key;
                    }
                });
                ui.hyperlink_to(self.i18n.t("apply_here"), "https://console.mistral.ai/");
                ui.label(self.i18n.t("api_activation_note"));
                
                ui.horizontal(|ui| {
                    if ui.button(self.i18n.t("save")).clicked() {
                        if !self.temp_api_key.trim().is_empty() {
                            self.config.api_key = Some(self.temp_api_key.trim().to_string());
                            let _ = save_config(&self.config);
                            self.show_api_modal = false;
                        }
                    }
                    if ui.button(self.i18n.t("cancel")).clicked() {
                        self.show_api_modal = false;
                    }
                });
            });
    }

    fn start_processing(&mut self, ctx: egui::Context) {
        self.is_processing = true;
        self.last_output_dirs.clear();
        let api_key = self.config.api_key.clone().unwrap();
        let files = self.file_queue.clone();
        let output_base = self.output_path.clone();
        let ocr_prefix = self.i18n.t("ocr_result_dir").to_string();

        let (tx, rx) = mpsc::channel(100);
        self.receiver = Some(rx);

        tokio::spawn(async move {
            let client = MistralClient::new(api_key);
            let total_files = files.len();
            let mut results = Vec::new();

            for (i, file_path) in files.iter().enumerate() {
                let _ = tx.send(ProgressUpdate::Total((i as f32) / (total_files as f32))).await;
                let _ = tx.send(ProgressUpdate::Message(format!("Processing {}...", file_path.file_name().unwrap_or_default().to_string_lossy()))).await;
                
                match process_single_file(&client, file_path, &output_base, &ocr_prefix, &tx).await {
                    Ok(out_dir) => results.push(out_dir),
                    Err(e) => {
                        let _ = tx.send(ProgressUpdate::Error(format!("Error: {}", e))).await;
                        return;
                    }
                }
                let _ = tx.send(ProgressUpdate::Current(1.0)).await;
            }
            
            let _ = tx.send(ProgressUpdate::Total(1.0)).await;
            let _ = tx.send(ProgressUpdate::Finished(results)).await;
            ctx.request_repaint();
        });
    }
}

async fn process_single_file(
    client: &MistralClient,
    path: &Path,
    output_base: &Path,
    ocr_prefix: &str,
    tx: &mpsc::Sender<ProgressUpdate>
) -> anyhow::Result<PathBuf> {
    let mut actual_path = path.to_path_buf();
    let is_img = pdf_utils::is_image_file(path);
    let mut _temp_pdf_dir = None;

    if is_img {
        let _ = tx.send(ProgressUpdate::Message("Converting image to PDF...".into())).await;
        let temp_dir = tempfile::tempdir()?;
        let pdf_path = temp_dir.path().join("converted.pdf");
        pdf_utils::convert_image_to_pdf(path, &pdf_path)?;
        actual_path = pdf_path;
        _temp_pdf_dir = Some(temp_dir);
    }

    let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let out_dir = output_base.join(format!("{}{}", ocr_prefix, file_stem));
    std::fs::create_dir_all(&out_dir)?;

    let size_mb = pdf_utils::get_pdf_size_mb(&actual_path)?;
    if size_mb <= 45.0 {
        process_chunk(client, &actual_path, &out_dir, 0, tx).await?;
    } else {
        let _ = tx.send(ProgressUpdate::Message("Splitting large PDF...".into())).await;
        let (chunks, _temp_dir) = pdf_utils::split_pdf(&actual_path, 45.0)?;
        let mut partial_files = Vec::new();
        let mut page_offset = 0;
        
        for (i, chunk) in chunks.iter().enumerate() {
            let _ = tx.send(ProgressUpdate::Message(format!("Processing chunk {}/{}:..", i+1, chunks.len()))).await;
            let partial_file = process_chunk(client, chunk, &out_dir, page_offset, tx).await?;
            partial_files.push(partial_file);
            
            let doc = ::lopdf::Document::load(chunk)?;
            page_offset += doc.get_pages().len() as u32;
        }
        
        merge_results(&out_dir, &partial_files)?;
    }

    Ok(out_dir)
}

async fn process_chunk(
    client: &MistralClient,
    path: &Path,
    out_dir: &Path,
    page_offset: u32,
    tx: &mpsc::Sender<ProgressUpdate>
) -> anyhow::Result<PathBuf> {
    let _ = tx.send(ProgressUpdate::Current(0.1)).await;
    let upload = client.upload_file(path).await?;
    let _ = tx.send(ProgressUpdate::Current(0.3)).await;
    let url = client.get_signed_url(&upload.id).await?;
    let _ = tx.send(ProgressUpdate::Current(0.5)).await;
    let ocr = client.process_ocr(url).await?;
    let _ = tx.send(ProgressUpdate::Current(0.8)).await;
    
    let partial_md = save_ocr_results(ocr, out_dir, page_offset)?;
    Ok(partial_md)
}

fn save_ocr_results(ocr: mistral_api::OCRResponse, out_dir: &Path, page_offset: u32) -> anyhow::Result<PathBuf> {
    let images_dir = out_dir.join("images");
    std::fs::create_dir_all(&images_dir)?;
    
    let mut page_markdowns = Vec::new();
    for (i, page) in ocr.pages.into_iter().enumerate() {
        let mut md = page.markdown;
        for img in page.images {
            if let Some(base64_data) = img.image_base64 {
                let data = if base64_data.contains(",") {
                    base64_data.split(',').nth(1).unwrap_or("")
                } else {
                    &base64_data
                };
                
                let bytes = general_purpose::STANDARD.decode(data)?;
                let img_filename = format!("part{}_page{}_{}.png", page_offset, i, img.id);
                std::fs::write(images_dir.join(&img_filename), bytes)?;
                
                // Replace in markdown
                // Mistral OCR returns placeholders like ![img_id](img_id)
                let old_placeholder = format!("![{}]({})", img.id, img.id);
                let new_placeholder = format!("![{}](images/{})", img.id, img_filename);
                md = md.replace(&old_placeholder, &new_placeholder);
                
                // Sometimes it might have a leading slash
                let old_placeholder_slash = format!("![{}](/{})", img.id, img.id);
                md = md.replace(&old_placeholder_slash, &new_placeholder);
            }
        }
        let actual_page = page_offset + i as u32 + 1;
        page_markdowns.push(format!("## Page {}\n\n{}", actual_page, md));
    }
    
    let partial_md_path = out_dir.join(format!("part_{}.md", page_offset));
    std::fs::write(&partial_md_path, page_markdowns.join("\n\n"))?;
    Ok(partial_md_path)
}

fn merge_results(out_dir: &Path, partial_files: &[PathBuf]) -> anyhow::Result<()> {
    let mut complete_content = Vec::new();
    let mut sorted_files = partial_files.to_vec();
    sorted_files.sort();
    
    for file in sorted_files {
        let content = std::fs::read_to_string(file)?;
        complete_content.push(content);
    }
    
    std::fs::write(out_dir.join("complete.md"), complete_content.join("\n\n"))?;
    Ok(())
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Common CJK font paths for different OS
    let font_paths = [
        // Windows
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
        // macOS
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
    ];

    let mut font_data = None;
    for path in font_paths {
        if std::path::Path::new(path).exists() {
            if let Ok(data) = std::fs::read(path) {
                font_data = Some(data);
                break;
            }
        }
    }

    if let Some(data) = font_data {
        fonts.font_data.insert(
            "my_font".to_owned(),
            egui::FontData::from_owned(data),
        );

        // Put my_font first (highest priority) for proportional text:
        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "my_font".to_owned());

        // Put my_font first for monospace text as well:
        fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "my_font".to_owned());
        
        ctx.set_fonts(fonts);
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 800.0])
            .with_min_inner_size([600.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "OCR-eg",
        native_options,
        Box::new(|cc| Ok(Box::new(AppState::new(cc)))),
    )
}
