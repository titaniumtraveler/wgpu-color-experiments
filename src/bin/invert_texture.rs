use image::Rgba;

fn main() -> anyhow::Result<()> {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/images/happy-little-tree.png"
    ));
    let img = image::load_from_memory(bytes)?;
    let mut rgba = img.to_rgba8();
    for Rgba([r, g, b, _a]) in rgba.pixels_mut() {
        *r = 0xFF - *r;
        *g = 0xFF - *g;
        *b = 0xFF - *b;
    }
    rgba.save_with_format(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/images/happy-little-tree-inverted.png"
        ),
        image::ImageFormat::Png,
    )?;

    Ok(())
}
