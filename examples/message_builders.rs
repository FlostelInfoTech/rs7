//! Example demonstrating HL7 message builders

use rs7_core::builders::adt::AdtBuilder;
use rs7_core::Version;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HL7 Message Builders Example ===\n");

    // Create ADT^A01 message
    println!("Creating ADT^A01 (Admit/Visit Notification)...");
    let message = AdtBuilder::a01(Version::V2_5)
        .sending_application("MyApp")
        .sending_facility("MyFacility")
        .receiving_application("RecApp")
        .receiving_facility("RecFacility")
        .patient_id("12345")
        .patient_name("DOE", "JOHN")
        .date_of_birth("19800101")
        .sex("M")
        .patient_class("I")
        .assigned_location("ER^101^1")
        .attending_doctor("SMITH^JAMES")
        .build()?;

    println!("{}\n", message.encode());

    // Create ADT^A08 message
    println!("Creating ADT^A08 (Update Patient Information)...");
    let update_message = AdtBuilder::a08(Version::V2_5)
        .sending_application("MyApp")
        .sending_facility("MyFacility")
        .receiving_application("RecApp")
        .receiving_facility("RecFacility")
        .patient_id("12345")
        .patient_name("DOE", "JOHN")
        .date_of_birth("19800101")
        .sex("M")
        .build()?;

    println!("{}\n", update_message.encode());

    // Create ORU^R01 message
    println!("Creating ORU^R01 (Observation Result)...");
    use rs7_core::builders::oru::{OruR01Builder, Observation};

    let oru_message = OruR01Builder::new(Version::V2_5)
        .sending_application("LabApp")
        .sending_facility("LabFacility")
        .receiving_application("EMR")
        .receiving_facility("Hospital")
        .patient_id("12345")
        .patient_name("DOE", "JOHN")
        .filler_order_number("LAB123456")
        .add_observation(Observation {
            set_id: 1,
            value_type: "NM".to_string(),
            identifier: "GLUCOSE".to_string(),
            value: "95".to_string(),
            units: Some("mg/dL".to_string()),
            status: "F".to_string(),
        })
        .add_observation(Observation {
            set_id: 2,
            value_type: "NM".to_string(),
            identifier: "CHOL".to_string(),
            value: "180".to_string(),
            units: Some("mg/dL".to_string()),
            status: "F".to_string(),
        })
        .build()?;

    println!("{}\n", oru_message.encode());

    // Create ORM^O01 message
    println!("Creating ORM^O01 (Order Message)...");
    use rs7_core::builders::orm::OrmO01Builder;

    let orm_message = OrmO01Builder::new(Version::V2_5)
        .sending_application("OrderApp")
        .sending_facility("Clinic")
        .receiving_application("LabApp")
        .receiving_facility("Lab")
        .patient_id("12345")
        .patient_name("DOE", "JOHN")
        .placer_order_number("ORD123456")
        .order_control("NW")
        .universal_service_id("CBC")
        .build()?;

    println!("{}\n", orm_message.encode());

    // Create QRY^A19 message
    println!("Creating QRY^A19 (Patient Query)...");
    use rs7_core::builders::qry::QryA19Builder;

    let qry_message = QryA19Builder::new(Version::V2_5)
        .sending_application("QueryApp")
        .sending_facility("Hospital")
        .receiving_application("MPI")
        .receiving_facility("Registry")
        .patient_id("12345")
        .build()?;

    println!("{}\n", qry_message.encode());

    println!("✓ All messages built successfully!");

    Ok(())
}
