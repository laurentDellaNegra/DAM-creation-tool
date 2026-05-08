//! Minimal OKLCH color space utilities.
//!
//! OKLCH provides perceptually uniform color interpolation.
//! Use [`oklch`] to define palette colors from OKLCH coordinates.

use egui::Color32;

/// Convert OKLCH (Lightness, Chroma, Hue) to [`Color32`].
///
/// - `l`: Lightness, 0.0 (black) to 1.0 (white)
/// - `c`: Chroma, 0.0 (gray) to ~0.4 (vivid)
/// - `h`: Hue angle in degrees, 0.0 to 360.0
pub fn oklch(l: f32, c: f32, h: f32) -> Color32 {
    let h_rad = h.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();
    oklab_to_color32(l, a, b)
}

/// Convert OKLab to [`Color32`].
fn oklab_to_color32(l: f32, a: f32, b: f32) -> Color32 {
    // OKLab → LMS (approximate inverse)
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    // LMS → linear sRGB
    let r = 4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3;
    let g = -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_38 * s3;
    let bl = -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3;

    Color32::from_rgb(linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(bl))
}

/// Linear RGB component to sRGB gamma-encoded byte.
fn linear_to_srgb(x: f32) -> u8 {
    let y = if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (y.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}
