//! Logo loading and rendering for FLOSTEL INFOTECH branding

use egui::{Color32, ColorImage, TextureHandle, TextureOptions};
use once_cell::sync::OnceCell;
use std::sync::Arc;

/// The embedded SVG logo data
const LOGO_SVG: &str = include_str!("../assets/flostel.svg");

/// Cached logo texture
static LOGO_TEXTURE: OnceCell<Arc<LogoData>> = OnceCell::new();

/// Logo data including the rendered image
pub struct LogoData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Get or create the logo texture
pub fn get_logo_texture(ctx: &egui::Context) -> Option<TextureHandle> {
    // Try to render the SVG to pixels - use larger size for better quality
    let logo_data = LOGO_TEXTURE.get_or_init(|| {
        Arc::new(render_svg_to_pixels(LOGO_SVG, 128, 128).unwrap_or_else(|| LogoData {
            width: 1,
            height: 1,
            pixels: vec![0, 0, 0, 0],
        }))
    });

    if logo_data.width <= 1 {
        return None;
    }

    // Create the texture
    let image = ColorImage::from_rgba_unmultiplied(
        [logo_data.width as usize, logo_data.height as usize],
        &logo_data.pixels,
    );

    Some(ctx.load_texture("flostel_logo", image, TextureOptions::LINEAR))
}

/// Render an SVG string to RGBA pixels
fn render_svg_to_pixels(svg_data: &str, width: u32, height: u32) -> Option<LogoData> {
    // Create a font database with system fonts for text rendering
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    // Set fallback fonts - use Arial which is available on Windows
    fontdb.set_sans_serif_family("Arial");
    fontdb.set_serif_family("Times New Roman");

    // Parse the SVG with font database in options
    // Use Arial as default font family for any unresolved fonts
    let options = usvg::Options {
        fontdb: std::sync::Arc::new(fontdb),
        font_family: "Arial".to_string(),
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(svg_data, &options).ok()?;

    // Create a pixmap to render into
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;

    // Calculate the transform to fit the SVG into the target size
    let svg_size = tree.size();
    let scale_x = width as f32 / svg_size.width();
    let scale_y = height as f32 / svg_size.height();
    let scale = scale_x.min(scale_y);

    let transform = tiny_skia::Transform::from_scale(scale, scale);

    // Render the SVG
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert to RGBA bytes
    let pixels: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|p| [p.red(), p.green(), p.blue(), p.alpha()])
        .collect();

    Some(LogoData {
        width,
        height,
        pixels,
    })
}

/// Display the company logo in the UI
/// Returns true if the logo was displayed, false if fallback text should be used
pub fn show_logo(ui: &mut egui::Ui, size: f32) -> bool {
    if let Some(texture) = get_logo_texture(ui.ctx()) {
        // Display the texture at the requested size
        ui.image(egui::load::SizedTexture::new(
            texture.id(),
            egui::vec2(size, size),
        ));
        true
    } else {
        // Fallback to text logo
        show_text_logo(ui, size);
        false
    }
}

/// Display a text-based logo (fallback when SVG loading fails)
pub fn show_text_logo(ui: &mut egui::Ui, size: f32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("F").size(size).strong().color(Color32::from_rgb(0, 128, 128)));
        ui.add_space(-size * 0.25);
        ui.label(egui::RichText::new("S").size(size * 0.875).strong().color(Color32::from_rgb(255, 0, 0)));
    });
}

/// Display the company name
pub fn show_company_name(ui: &mut egui::Ui, size: f32) {
    ui.label(
        egui::RichText::new("FLOSTEL INFOTECH")
            .size(size)
            .strong()
            .color(Color32::from_rgb(0, 128, 128)),
    );
}
