//! RS7 Test Panel - GUI Application for HL7 Message Testing
//!
//! A comprehensive GUI application demonstrating the capabilities of the RS7 HL7 library.
//!
//! ## Features
//!
//! - **Parser**: Parse and analyze HL7 v2.x messages with tree view
//! - **Builder**: Visually build HL7 messages using the fluent API
//! - **Terser**: Access and modify fields using path notation
//! - **Validator**: Validate messages against HL7 schemas
//! - **MLLP**: Send and receive messages via MLLP protocol
//! - **FHIR**: Convert between HL7 v2.x and FHIR R4
//! - **XML**: Convert between ER7 and XML formats

mod app;
mod logo;
mod tabs;
mod samples;
mod utils;

use app::Rs7TestPanel;

fn main() -> eframe::Result<()> {
    // Configure logging for debug builds
    #[cfg(debug_assertions)]
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([900.0, 650.0])
            .with_title("RS7 Test Panel - HL7 v2.x Testing Suite")
            .with_app_id("rs7-testpanel"),
        centered: true,
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
        egui::FontId::new(18.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
    );

    // Increase spacing for cleaner look
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 5.0);
    style.spacing.window_margin = egui::Margin::same(12);
    style.spacing.indent = 20.0;

    // Improve visual appearance - use CornerRadius (renamed from Rounding in egui 0.33)
    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
    style.visuals.window_corner_radius = egui::CornerRadius::same(8);

    ctx.set_style(style);
}
