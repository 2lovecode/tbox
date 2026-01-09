use serde::Serialize;
use std::path::PathBuf;
use tauri::AppHandle;
use image::{ImageReader, GenericImageView};

#[derive(Serialize)]
pub struct ImageCompressResult {
    original_size: u64,
    compressed_size: u64,
    saved_percentage: f64,
    output_path: String,
}

#[tauri::command]
pub async fn compress_image(
    _app: AppHandle,
    input_path: String,
    output_path: Option<String>,
    _quality: Option<u8>,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> Result<ImageCompressResult, String> {
    let max_width = max_width.unwrap_or(1920);
    let max_height = max_height.unwrap_or(1080);

    // 读取原始图片
    let img_decoded = ImageReader::open(&input_path)
        .map_err(|e| format!("无法打开图片: {}", e))?
        .decode()
        .map_err(|e| format!("无法解码图片: {}", e))?;

    // 获取原始文件大小
    let original_size = std::fs::metadata(&input_path)
        .map_err(|e| format!("无法读取文件信息: {}", e))?
        .len();

    // 调整尺寸
    let (width, height) = img_decoded.dimensions();
    let resized_img = if width > max_width || height > max_height {
        let ratio = (max_width as f32 / width as f32).min(max_height as f32 / height as f32);
        let new_width = (width as f32 * ratio) as u32;
        let new_height = (height as f32 * ratio) as u32;
        img_decoded.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
    } else {
        img_decoded
    };

    // 确定输出路径
    let output = if let Some(path) = output_path {
        PathBuf::from(path)
    } else {
        let mut path = PathBuf::from(&input_path);
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("compressed");
        path.set_file_name(format!("{}_compressed.jpg", stem));
        path
    };

    // 保存压缩后的图片
    // image 0.25 版本使用 save 方法
    let output_path_str = output.to_string_lossy().to_string();
    resized_img
        .save(&output_path_str)
        .map_err(|e| format!("无法保存图片: {}", e))?;

    // 读取压缩后的文件大小
    let compressed_size = std::fs::metadata(&output)
        .map_err(|e| format!("无法读取压缩后的文件信息: {}", e))?
        .len();

    let saved_percentage = ((original_size as f64 - compressed_size as f64) / original_size as f64) * 100.0;

    Ok(ImageCompressResult {
        original_size,
        compressed_size,
        saved_percentage,
        output_path: output.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn get_image_info(image_path: String) -> Result<ImageInfo, String> {
    let img = ImageReader::open(&image_path)
        .map_err(|e| format!("无法打开图片: {}", e))?
        .decode()
        .map_err(|e| format!("无法解码图片: {}", e))?;

    let metadata = std::fs::metadata(&image_path)
        .map_err(|e| format!("无法读取文件信息: {}", e))?;

    Ok(ImageInfo {
        width: img.width(),
        height: img.height(),
        format: format!("{:?}", img.color()),
        size: metadata.len(),
    })
}

#[derive(Serialize)]
pub struct ImageInfo {
    width: u32,
    height: u32,
    format: String,
    size: u64,
}
