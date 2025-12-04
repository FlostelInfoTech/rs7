//! Message Builder Tab - Build HL7 messages visually

use egui::{self, RichText, Color32};
use rs7_core::builders::adt::AdtA01Builder;
use rs7_core::Version;

pub struct BuilderTab {
    // Message type selection
    message_type: MessageType,

    // ADT A01 fields
    patient_id: String,
    patient_id_system: String,
    family_name: String,
    given_name: String,
    middle_name: String,
    birth_date: String,
    gender: Gender,
    street_address: String,
    city: String,
    state: String,
    zip_code: String,
    country: String,
    phone_home: String,
    phone_work: String,

    // Visit info
    patient_class: PatientClass,
    attending_doctor_id: String,
    attending_doctor_name: String,
    room: String,
    bed: String,

    // Message header
    sending_app: String,
    sending_facility: String,
    receiving_app: String,
    receiving_facility: String,

    // Output
    built_message: Option<String>,
    build_error: Option<String>,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum MessageType {
    #[default]
    AdtA01,
    AdtA04,
    AdtA08,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum Gender {
    #[default]
    Male,
    Female,
    Other,
    Unknown,
}

impl Gender {
    fn code(&self) -> &str {
        match self {
            Gender::Male => "M",
            Gender::Female => "F",
            Gender::Other => "O",
            Gender::Unknown => "U",
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum PatientClass {
    #[default]
    Inpatient,
    Outpatient,
    Emergency,
    Observation,
}

impl PatientClass {
    fn code(&self) -> &str {
        match self {
            PatientClass::Inpatient => "I",
            PatientClass::Outpatient => "O",
            PatientClass::Emergency => "E",
            PatientClass::Observation => "B",
        }
    }
}

impl Default for BuilderTab {
    fn default() -> Self {
        Self {
            message_type: MessageType::AdtA01,
            patient_id: "12345678".to_string(),
            patient_id_system: "HOSPITAL".to_string(),
            family_name: "DOE".to_string(),
            given_name: "JOHN".to_string(),
            middle_name: "A".to_string(),
            birth_date: "1980-01-15".to_string(),
            gender: Gender::Male,
            street_address: "123 MAIN STREET".to_string(),
            city: "SPRINGFIELD".to_string(),
            state: "IL".to_string(),
            zip_code: "62701".to_string(),
            country: "USA".to_string(),
            phone_home: "555-123-4567".to_string(),
            phone_work: "555-987-6543".to_string(),
            patient_class: PatientClass::Inpatient,
            attending_doctor_id: "1234567890".to_string(),
            attending_doctor_name: "SMITH, JOHN D MD".to_string(),
            room: "ICU".to_string(),
            bed: "101-A".to_string(),
            sending_app: "TEST_APP".to_string(),
            sending_facility: "TEST_FACILITY".to_string(),
            receiving_app: "RECEIVING_APP".to_string(),
            receiving_facility: "RECEIVING_FAC".to_string(),
            built_message: None,
            build_error: None,
        }
    }
}

impl BuilderTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Message Builder");
        ui.label("Build HL7 messages using the fluent builder API.");
        ui.add_space(10.0);

        ui.columns(2, |columns| {
            // Left column: Form
            columns[0].group(|ui| {
                egui::ScrollArea::vertical()
                    .id_salt("builder_form")
                    .max_height(650.0)
                    .show(ui, |ui| {
                        // Message Type
                        ui.heading("Message Type");
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.message_type, MessageType::AdtA01, "ADT^A01 (Admit)");
                            ui.selectable_value(&mut self.message_type, MessageType::AdtA04, "ADT^A04 (Register)");
                            ui.selectable_value(&mut self.message_type, MessageType::AdtA08, "ADT^A08 (Update)");
                        });

                        ui.add_space(10.0);
                        ui.separator();

                        // Message Header
                        ui.heading("Message Header");
                        ui.horizontal(|ui| {
                            ui.label("Sending App:");
                            ui.text_edit_singleline(&mut self.sending_app);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Sending Facility:");
                            ui.text_edit_singleline(&mut self.sending_facility);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Receiving App:");
                            ui.text_edit_singleline(&mut self.receiving_app);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Receiving Facility:");
                            ui.text_edit_singleline(&mut self.receiving_facility);
                        });

                        ui.add_space(10.0);
                        ui.separator();

                        // Patient Information
                        ui.heading("Patient Information");

                        ui.horizontal(|ui| {
                            ui.label("Patient ID:");
                            ui.text_edit_singleline(&mut self.patient_id);
                            ui.label("System:");
                            ui.text_edit_singleline(&mut self.patient_id_system);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Family Name:");
                            ui.text_edit_singleline(&mut self.family_name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Given Name:");
                            ui.text_edit_singleline(&mut self.given_name);
                            ui.label("Middle:");
                            ui.add_sized([60.0, 20.0], egui::TextEdit::singleline(&mut self.middle_name));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Birth Date (YYYY-MM-DD):");
                            ui.text_edit_singleline(&mut self.birth_date);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Gender:");
                            ui.selectable_value(&mut self.gender, Gender::Male, "Male");
                            ui.selectable_value(&mut self.gender, Gender::Female, "Female");
                            ui.selectable_value(&mut self.gender, Gender::Other, "Other");
                            ui.selectable_value(&mut self.gender, Gender::Unknown, "Unknown");
                        });

                        ui.add_space(10.0);
                        ui.separator();

                        // Address
                        ui.heading("Address");
                        ui.horizontal(|ui| {
                            ui.label("Street:");
                            ui.add_sized([300.0, 20.0], egui::TextEdit::singleline(&mut self.street_address));
                        });
                        ui.horizontal(|ui| {
                            ui.label("City:");
                            ui.text_edit_singleline(&mut self.city);
                            ui.label("State:");
                            ui.add_sized([40.0, 20.0], egui::TextEdit::singleline(&mut self.state));
                        });
                        ui.horizontal(|ui| {
                            ui.label("ZIP Code:");
                            ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.zip_code));
                            ui.label("Country:");
                            ui.add_sized([60.0, 20.0], egui::TextEdit::singleline(&mut self.country));
                        });

                        ui.add_space(10.0);
                        ui.separator();

                        // Contact
                        ui.heading("Contact Information");
                        ui.horizontal(|ui| {
                            ui.label("Home Phone:");
                            ui.text_edit_singleline(&mut self.phone_home);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Work Phone:");
                            ui.text_edit_singleline(&mut self.phone_work);
                        });

                        ui.add_space(10.0);
                        ui.separator();

                        // Visit Information
                        ui.heading("Visit Information");
                        ui.horizontal(|ui| {
                            ui.label("Patient Class:");
                            ui.selectable_value(&mut self.patient_class, PatientClass::Inpatient, "Inpatient");
                            ui.selectable_value(&mut self.patient_class, PatientClass::Outpatient, "Outpatient");
                            ui.selectable_value(&mut self.patient_class, PatientClass::Emergency, "Emergency");
                            ui.selectable_value(&mut self.patient_class, PatientClass::Observation, "Observation");
                        });

                        ui.horizontal(|ui| {
                            ui.label("Room:");
                            ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.room));
                            ui.label("Bed:");
                            ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.bed));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Attending Dr ID:");
                            ui.text_edit_singleline(&mut self.attending_doctor_id);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Attending Dr Name:");
                            ui.text_edit_singleline(&mut self.attending_doctor_name);
                        });

                        ui.add_space(20.0);

                        // Build button
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Build Message").strong()).clicked() {
                                self.build_message();
                            }
                            if ui.button("Reset to Defaults").clicked() {
                                *self = Self::default();
                            }
                        });
                    });
            });

            // Right column: Output
            columns[1].group(|ui| {
                ui.heading("Generated Message");

                if let Some(ref error) = self.build_error {
                    ui.colored_label(Color32::RED, format!("Error: {}", error));
                } else if let Some(ref message) = self.built_message {
                    ui.label(format!("Size: {} bytes", message.len()));

                    if ui.button("Copy to Clipboard").clicked() {
                        ui.output_mut(|o| o.copied_text = message.clone());
                    }

                    ui.add_space(10.0);

                    egui::ScrollArea::vertical()
                        .id_salt("built_message")
                        .max_height(580.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut message.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(35)
                                    .interactive(false)
                            );
                        });
                } else {
                    ui.label("Fill in the form and click 'Build Message' to generate an HL7 message.");
                    ui.add_space(20.0);
                    ui.label("The builder uses the fluent API from rs7-core:");
                    ui.add_space(10.0);
                    ui.code("AdtA01Builder::new()\n    .sending_application(\"APP\")\n    .patient_id(\"12345\")\n    .patient_name(name)\n    .build()");
                }
            });
        });
    }

    fn build_message(&mut self) {
        let result = self.try_build_message();
        match result {
            Ok(msg) => {
                self.built_message = Some(msg);
                self.build_error = None;
            }
            Err(e) => {
                self.build_error = Some(e);
                self.built_message = None;
            }
        }
    }

    fn try_build_message(&self) -> Result<String, String> {
        // Format birth date from YYYY-MM-DD to YYYYMMDD
        let birth_date_hl7 = self.birth_date.replace("-", "");

        // Note: Patient name is constructed directly in the builder call below

        // Build attending doctor field
        let attending_doctor = format!("{}^{}", self.attending_doctor_id, self.attending_doctor_name);

        // Build assigned location
        let assigned_location = format!("{}^{}", self.room, self.bed);

        // Build the message based on type
        let message = match self.message_type {
            MessageType::AdtA01 | MessageType::AdtA04 | MessageType::AdtA08 => {
                AdtA01Builder::new(Version::V2_5_1)
                    .sending_application(&self.sending_app)
                    .sending_facility(&self.sending_facility)
                    .receiving_application(&self.receiving_app)
                    .receiving_facility(&self.receiving_facility)
                    .patient_id(&self.patient_id)
                    .patient_name(&self.family_name, &self.given_name)
                    .date_of_birth(&birth_date_hl7)
                    .sex(self.gender.code())
                    .patient_class(self.patient_class.code())
                    .assigned_location(&assigned_location)
                    .attending_doctor(&attending_doctor)
                    .build()
                    .map_err(|e| format!("Build error: {}", e))?
            }
        };

        Ok(message.encode())
    }
}
