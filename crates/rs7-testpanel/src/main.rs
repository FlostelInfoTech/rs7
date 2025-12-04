//! RS7 Test Panel - GUI Application for HL7 Message Testing
//!
//! A comprehensive GUI application demonstrating the capabilities of the RS7 HL7 library.

mod app;
mod tabs;
mod samples;
mod utils;

use app::Rs7TestPanel;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("RS7 Test Panel - HL7 v2.x Testing Suite"),
        ..Default::default()
    };

    eframe::run_native(
        "RS7 Test Panel",
        options,
        Box::new(|cc| {
            // Set up custom fonts and style
            setup_custom_style(&cc.egui_ctx);
            Ok(Box::new(Rs7TestPanel::new(cc)))
        }),
    )
}

fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Use a slightly larger font for better readability
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(13.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );

    // Increase spacing for cleaner look
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);

    ctx.set_style(style);
}
