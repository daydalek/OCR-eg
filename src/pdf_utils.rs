use std::path::{Path, PathBuf};
use anyhow::Result;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

pub fn get_pdf_size_mb<P: AsRef<Path>>(path: P) -> Result<f64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len() as f64 / (1024.0 * 1024.0))
}

pub fn split_pdf<P: AsRef<Path>>(path: P, max_size_mb: f64) -> Result<(Vec<PathBuf>, PathBuf)> {
    let path = path.as_ref();
    let temp_dir = tempfile::tempdir()?.into_path();

    let doc = ::lopdf::Document::load(path)?;
    let total_pages = doc.get_pages().len() as u32;
    
    let mut split_files = Vec::new();
    let file_size_mb = get_pdf_size_mb(path)?;
    let pages_per_mb = total_pages as f64 / file_size_mb;
    let mut pages_per_chunk = (max_size_mb * pages_per_mb * 0.9) as u32;
    if pages_per_chunk == 0 { pages_per_chunk = 1; }
    
    let mut current_page = 0;
    let mut chunk_number = 0;
    
    while current_page < total_pages {
        let end_page = std::cmp::min(current_page + pages_per_chunk, total_pages);
        
        let mut chunk_doc = doc.clone();
        let mut pages_to_delete = Vec::new();
        
        for page_num in 1..=total_pages {
            if page_num <= current_page || page_num > end_page {
                pages_to_delete.push(page_num);
            }
        }
        
        chunk_doc.delete_pages(&pages_to_delete);
        
        let chunk_path = temp_dir.join(format!("chunk_{}.pdf", chunk_number));
        chunk_doc.save(&chunk_path)?;
        
        let chunk_size_mb = get_pdf_size_mb(&chunk_path)?;
        if chunk_size_mb > max_size_mb && (end_page - current_page) > 1 {
            let _ = std::fs::remove_file(&chunk_path);
            pages_per_chunk = std::cmp::max(1, (pages_per_chunk as f64 * 0.7) as u32);
            continue;
        }
        
        split_files.push(chunk_path);
        current_page = end_page;
        chunk_number += 1;
    }
    
    Ok((split_files, temp_dir))
}

pub fn convert_image_to_pdf<P: AsRef<Path>>(image_path: P, output_path: P) -> Result<()> {
    let img = ::image::open(image_path.as_ref())?;
    let (width, height) = ::image::GenericImageView::dimensions(&img);
    
    let width_pt = width as f32 * 72.0 / 96.0;
    let height_pt = height as f32 * 72.0 / 96.0;
    
    let (doc, page1, layer1) = PdfDocument::new("PDF", Mm::from(Pt(width_pt)), Mm::from(Pt(height_pt)), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    
    let img_rgb = img.to_rgb8();
    let image_bytes = img_rgb.as_raw();
    
    let image_x_object = ImageXObject {
        width: Px(width as usize),
        height: Px(height as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: image_bytes.to_vec(),
        clipping_bbox: None,
        image_filter: None,
        smask: None,
    };
    
    let image = Image::from(image_x_object);
    image.add_to_layer(current_layer, ImageTransform::default());
    
    let file = File::create(output_path.as_ref())?;
    doc.save(&mut BufWriter::new(file))?;
    
    Ok(())
}

pub fn is_image_file<P: AsRef<Path>>(path: P) -> bool {
    let ext = path.as_ref().extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "bmp" | "tiff" | "tif")
}
