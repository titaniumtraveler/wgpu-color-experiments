#![allow(dead_code, clippy::identity_op)]

use trig_const::pow;

/// remaps a value from `0..=255` to `0.0..=1.0`
const fn to_f32(c: u32) -> f32 {
    c as f32 / 255.0
}

/// remaps a value from `0..=255` to `0.0..=1.0`
const fn to_f64(c: u32) -> f64 {
    c as f64 / 255.0
}

/// converts a color component (like r, g, b) into the equivalent srgb color component
const fn srgb_to_f32(c: u32) -> f32 {
    pow((to_f64(c) + 0.055) / 1.055,2.4) as f32
}

/// converts a color component (like r, g, b) into the equivalent srgb color component
fn srgb_to_f64(c: u32) -> f64 {
    ((to_f64(c) + 0.055) / 1.055).powf(2.4)
}

/// Accepts color in the format of `0xRRGGBB`
pub const fn rgb_to_f32x3(c: u32) -> [f32;3] {
    let r = (c & 0xFF0000) >> 16;
    let g = (c & 0x00FF00) >> 8;
    let b = (c & 0x0000FF) >> 0;

    [srgb_to_f32(r), srgb_to_f32(g), srgb_to_f32(b),]
}

/// Accepts color in the format of `0xRRGGBB`
pub fn rgb_to_wgpu_color(c: u32) -> wgpu::Color {
    let r = (c & 0xFF0000) >> 16;
    let g = (c & 0x00FF00) >> 8;
    let b = (c & 0x0000FF) >> 0;

    wgpu::Color {
        r: srgb_to_f64(r),
        g: srgb_to_f64(g),
        b: srgb_to_f64(b),
        a: 1.0,
    }
}

/// Accepts color in the format of `0xRRGGBBAA`
pub fn rgba(c: u32) -> wgpu::Color {
    let r = c & 0xFF000000 >> 24;
    let g = c & 0x00FF0000 >> 16;
    let b = c & 0x0000FF00 >> 8;
    let a = c & 0x000000FF >> 0;

    wgpu::Color {
        r: srgb_to_f64(r),
        g: srgb_to_f64(g),
        b: srgb_to_f64(b),
        a: to_f64(a),
    }
}

