//! Converters for transforming HL7 v2.x segments to FHIR resources

pub mod patient;
pub mod observation;
pub mod practitioner;
pub mod encounter;
pub mod diagnostic_report;
pub mod allergy_intolerance;
pub mod medication;
pub mod condition;
pub mod procedure;

pub use patient::PatientConverter;
pub use observation::ObservationConverter;
pub use practitioner::PractitionerConverter;
pub use encounter::EncounterConverter;
pub use diagnostic_report::DiagnosticReportConverter;
pub use allergy_intolerance::AllergyIntoleranceConverter;
pub use medication::MedicationConverter;
pub use condition::ConditionConverter;
pub use procedure::ProcedureConverter;
