//! Main application state and UI rendering

use eframe::egui::{self, Color32, RichText, Stroke};
use crate::logo;
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

impl ActiveTab {
    fn label(&self) -> &str {
        match self {
            ActiveTab::Parser => "Parser",
            ActiveTab::Builder => "Builder",
            ActiveTab::Terser => "Terser",
            ActiveTab::Validator => "Validator",
            ActiveTab::Mllp => "MLLP",
            ActiveTab::Fhir => "FHIR",
            ActiveTab::Xml => "XML",
        }
    }

    fn icon(&self) -> &str {
        match self {
            ActiveTab::Parser => "\u{1F50D}",      // Magnifying glass
            ActiveTab::Builder => "\u{1F3D7}",     // Construction
            ActiveTab::Terser => "\u{1F4CD}",      // Pin/locator
            ActiveTab::Validator => "\u{2705}",    // Check mark
            ActiveTab::Mllp => "\u{1F4E1}",        // Antenna/network
            ActiveTab::Fhir => "\u{1F525}",        // Fire (FHIR)
            ActiveTab::Xml => "\u{1F4C4}",         // Document
        }
    }

    fn tooltip(&self) -> &str {
        match self {
            ActiveTab::Parser => "Parse and analyze HL7 messages (Ctrl+1)",
            ActiveTab::Builder => "Build HL7 messages visually (Ctrl+2)",
            ActiveTab::Terser => "Access fields using path notation (Ctrl+3)",
            ActiveTab::Validator => "Validate messages against schemas (Ctrl+4)",
            ActiveTab::Mllp => "Send/receive via MLLP protocol (Ctrl+5)",
            ActiveTab::Fhir => "Convert to/from FHIR R4 (Ctrl+6)",
            ActiveTab::Xml => "Convert between ER7 and XML (Ctrl+7)",
        }
    }

    fn all() -> &'static [ActiveTab] {
        &[
            ActiveTab::Parser,
            ActiveTab::Builder,
            ActiveTab::Terser,
            ActiveTab::Validator,
            ActiveTab::Mllp,
            ActiveTab::Fhir,
            ActiveTab::Xml,
        ]
    }
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
    show_shortcuts: bool,
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
            show_shortcuts: false,
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::Num1) {
                    self.active_tab = ActiveTab::Parser;
                } else if i.key_pressed(egui::Key::Num2) {
                    self.active_tab = ActiveTab::Builder;
                } else if i.key_pressed(egui::Key::Num3) {
                    self.active_tab = ActiveTab::Terser;
                } else if i.key_pressed(egui::Key::Num4) {
                    self.active_tab = ActiveTab::Validator;
                } else if i.key_pressed(egui::Key::Num5) {
                    self.active_tab = ActiveTab::Mllp;
                } else if i.key_pressed(egui::Key::Num6) {
                    self.active_tab = ActiveTab::Fhir;
                } else if i.key_pressed(egui::Key::Num7) {
                    self.active_tab = ActiveTab::Xml;
                }
            }
            if i.key_pressed(egui::Key::F1) {
                self.show_shortcuts = !self.show_shortcuts;
            }
        });
    }
}

impl eframe::App for Rs7TestPanel {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Top menu bar with branding
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Logo/branding
                ui.add_space(4.0);
                ui.label(RichText::new("RS7").strong().size(16.0).color(Color32::from_rgb(59, 130, 246)));
                ui.separator();

                ui.menu_button("File", |ui| {
                    if ui.button("\u{1F4C2} Open Message...").clicked() {
                        // TODO: File open dialog
                        ui.close();
                    }
                    if ui.button("\u{1F4BE} Save Message...").clicked() {
                        // TODO: File save dialog
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("\u{274C} Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    for tab in ActiveTab::all() {
                        if ui.button(format!("{} {}", tab.icon(), tab.label())).clicked() {
                            self.active_tab = *tab;
                            ui.close();
                        }
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("\u{2328} Keyboard Shortcuts (F1)").clicked() {
                        self.show_shortcuts = true;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("\u{2139} About RS7 Test Panel").clicked() {
                        self.show_about = true;
                        ui.close();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).weak());
                    ui.separator();
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });

        // Tab bar with icons
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                for tab in ActiveTab::all() {
                    let is_selected = self.active_tab == *tab;
                    let text = format!("{} {}", tab.icon(), tab.label());
                    let response = ui.selectable_label(is_selected, RichText::new(&text).size(13.0));
                    if response.clicked() {
                        self.active_tab = *tab;
                    }
                    response.on_hover_text(tab.tooltip());
                }
            });
            ui.add_space(6.0);
            // Draw a subtle separator line
            let rect = ui.available_rect_before_wrap();
            ui.painter().hline(
                rect.x_range(),
                rect.bottom(),
                Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            );
        });

        // Status bar with more info
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                // Company logo in status bar (small text version for status bar)
                logo::show_text_logo(ui, 14.0);
                ui.add_space(-4.0);
                ui.label(RichText::new("FLOSTEL INFOTECH").weak().small());
                ui.separator();
                ui.label(RichText::new(format!("{} {}", self.active_tab.icon(), self.active_tab.label())).weak());
                ui.separator();
                ui.label(RichText::new("Ready").weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("Press F1 for shortcuts").weak().italics());
                    ui.separator();
                    ui.label(RichText::new("HL7 v2.x Testing Suite").weak());
                });
            });
            ui.add_space(2.0);
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
                .min_width(400.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        // Company logo from SVG
                        logo::show_logo(ui, 48.0);
                        logo::show_company_name(ui, 12.0);
                        ui.add_space(10.0);

                        ui.label(RichText::new("RS7 Test Panel").heading().strong().size(24.0));
                        ui.label(RichText::new("HL7 v2.x Testing Suite").size(14.0).weak());
                        ui.add_space(15.0);

                        ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).strong());
                        ui.add_space(5.0);
                        ui.label("A comprehensive GUI application for testing");
                        ui.label("the RS7 HL7 v2.x library capabilities.");
                        ui.add_space(15.0);

                        ui.separator();
                        ui.add_space(10.0);

                        ui.label(RichText::new("Features").strong());
                        ui.add_space(5.0);
                    });

                    egui::Grid::new("features_grid")
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("\u{1F50D}");
                            ui.label("Parse and analyze HL7 messages");
                            ui.end_row();

                            ui.label("\u{1F3D7}");
                            ui.label("Build messages with visual builders");
                            ui.end_row();

                            ui.label("\u{1F4CD}");
                            ui.label("Access fields with Terser path notation");
                            ui.end_row();

                            ui.label("\u{2705}");
                            ui.label("Validate against HL7 schemas");
                            ui.end_row();

                            ui.label("\u{1F4E1}");
                            ui.label("Send/receive via MLLP protocol");
                            ui.end_row();

                            ui.label("\u{1F525}");
                            ui.label("Convert to/from FHIR R4");
                            ui.end_row();

                            ui.label("\u{1F4C4}");
                            ui.label("XML encoding/decoding");
                            ui.end_row();
                        });

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Supported HL7 Versions").strong());
                        ui.label("v2.3, v2.3.1, v2.4, v2.5, v2.5.1, v2.6, v2.7, v2.7.1");
                        ui.add_space(10.0);

                        ui.label(RichText::new("Message Types").weak());
                        ui.label(RichText::new("ADT, ORU, ORM, SIU, ACK, VXU, RDE, and more...").weak().italics());

                        ui.add_space(15.0);
                        if ui.button(RichText::new("  Close  ").size(14.0)).clicked() {
                            self.show_about = false;
                        }
                        ui.add_space(10.0);
                    });
                });
        }

        // Keyboard shortcuts dialog
        if self.show_shortcuts {
            egui::Window::new("\u{2328} Keyboard Shortcuts")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(350.0)
                .show(ctx, |ui| {
                    ui.add_space(5.0);

                    egui::Grid::new("shortcuts_grid")
                        .num_columns(2)
                        .spacing([30.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Navigation").strong().underline());
                            ui.label("");
                            ui.end_row();

                            for (i, tab) in ActiveTab::all().iter().enumerate() {
                                ui.label(format!("Ctrl+{}", i + 1));
                                ui.label(format!("{} {}", tab.icon(), tab.label()));
                                ui.end_row();
                            }

                            // Empty row for spacing (add_space not allowed in grid)
                            ui.label("");
                            ui.label("");
                            ui.end_row();

                            ui.label(RichText::new("General").strong().underline());
                            ui.label("");
                            ui.end_row();

                            ui.label("F1");
                            ui.label("Toggle this dialog");
                            ui.end_row();

                            ui.label("Ctrl+Enter");
                            ui.label("Parse message (in Parser tab)");
                            ui.end_row();
                        });

                    ui.add_space(15.0);
                    ui.vertical_centered(|ui| {
                        if ui.button("  Close  ").clicked() {
                            self.show_shortcuts = false;
                        }
                    });
                    ui.add_space(5.0);
                });
        }
    }
}
