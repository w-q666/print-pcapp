#![allow(dead_code)]

use base64::Engine;
use image::Luma;
use qrcode::QrCode;

/// 生成 QR 码并返回 PNG 图片的 base64 编码字符串。
/// 前端可直接用 `data:image/png;base64,{result}` 显示。
pub fn generate_qr_base64(content: &str) -> Result<String, String> {
    let code = QrCode::new(content.as_bytes()).map_err(|e| e.to_string())?;
    let img = code
        .render::<Luma<u8>>()
        .quiet_zone(true)
        .min_dimensions(256, 256)
        .build();

    let mut png_bytes: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::L8,
    )
    .map_err(|e| e.to_string())?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
}
