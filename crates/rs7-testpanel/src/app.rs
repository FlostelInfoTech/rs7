//! Main application state and UI rendering

use eframe::egui;
use crate::tabs::{
    ParserTab, BuilderTab, MllpTab, ValidatorTab, FhirTab, TerserTab, XmlTab,
};

/// The active tab in the application
#[derive(Default, PartialEq, Clone, Copy)]
pub enum ActiveTab {
    #[default]
    Parser,
    Builder,
    Terser,
    Validator,
    Mllp,
    Fhir,
    Xml,
}

/// Main application state
pub struct Rs7TestPanel {
    active_tab: ActiveTab,
    parser_tab: ParserTab,
    builder_tab: BuilderTab,
    terser_tab: TerserTab,
    validator_tab: ValidatorTab,
    mllp_tab: MllpTab,
    fhir_tab: FhirTab,
    xml_tab: XmlTab,
    show_about: bool,
}

impl Rs7TestPanel {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            active_tab: ActiveTab::Parser,
            parser_tab: ParserTab::default(),
            builder_tab: BuilderTab::default(),
            terser_tab: TerserTab::default(),
            validator_tab: ValidatorTab::default(),
            mllp_tab: MllpTab::default(),
            fhir_tab: FhirTab::default(),
            xml_tab: XmlTab::default(),
            show_about: false,
        }
    }
}

impl eframe::App for Rs7TestPanel {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About RS7 Test Panel").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("RS7 v{}", env!("CARGO_PKG_VERSION")));
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });

        // Tab bar
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, ActiveTab::Parser, "Message Parser");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Builder, "Message Builder");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Terser, "Terser (Field Access)");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Validator, "Validator");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Mllp, "MLLP Client/Server");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Fhir, "FHIR Converter");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Xml, "XML Encoding");
            });
            ui.add_space(4.0);
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Ready");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("RS7 HL7 Library Test Panel");
                });
            });
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                ActiveTab::Parser => self.parser_tab.ui(ui),
                ActiveTab::Builder => self.builder_tab.ui(ui),
                ActiveTab::Terser => self.terser_tab.ui(ui),
                ActiveTab::Validator => self.validator_tab.ui(ui),
                ActiveTab::Mllp => self.mllp_tab.ui(ui, ctx),
                ActiveTab::Fhir => self.fhir_tab.ui(ui),
                ActiveTab::Xml => self.xml_tab.ui(ui),
            }
        });

        // About dialog
        if self.show_about {
            egui::Window::new("About RS7 Test Panel")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("RS7 Test Panel");
                        ui.add_space(10.0);
                        ui.label("A comprehensive GUI application for testing");
                        ui.label("the RS7 HL7 v2.x library capabilities.");
                        ui.add_space(10.0);
                        ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                        ui.add_space(10.0);
                        ui.label("Features:");
                        ui.label("- Parse and analyze HL7 messages");
                        ui.label("- Build messages with visual builders");
                        ui.label("- Access fields with Terser path notation");
                        ui.label("- Validate against HL7 schemas");
                        ui.label("- Send/receive via MLLP protocol");
                        ui.label("- Convert to/from FHIR R4");
                        ui.label("- XML encoding/decoding");
                        ui.add_space(15.0);
                        if ui.button("Close").clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }
    }
}
