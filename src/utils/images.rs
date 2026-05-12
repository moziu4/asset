use image::ImageFormat;
use std::io::Cursor;
use bytes::Bytes;

pub fn resize_image(data: &[u8], width: u32, height: u32) -> anyhow::Result<Bytes> {
    let img = image::load_from_memory(data)?;
    let resized = img.thumbnail(width, height);
    
    let mut buffer = Cursor::new(Vec::new());
    // Convertimos siempre a AVIF como se solicita
    resized.write_to(&mut buffer, ImageFormat::Avif)?;
    
    Ok(Bytes::from(buffer.into_inner()))
}
