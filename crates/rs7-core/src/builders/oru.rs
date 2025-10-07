//! ORU (Observation Result) message builders

use super::{generate_control_id, MessageBuilder};
use crate::{
    error::Result,
    field::Field,
    message::Message,
    segment::Segment,
    Version,
};

/// Builder for ORU^R01 - Unsolicited Observation Message
pub struct OruR01Builder {
    base: MessageBuilder,
    sending_app: String,
    sending_facility: String,
    receiving_app: String,
    receiving_facility: String,
    control_id: Option<String>,
    processing_id: String,
    patient_id: Option<String>,
    patient_name: Option<(String, String)>,
    order_control: String,
    filler_order_number: Option<String>,
    observations: Vec<Observation>,
}

pub struct Observation {
    pub set_id: u32,
    pub value_type: String,
    pub identifier: String,
    pub value: String,
    pub units: Option<String>,
    pub status: String,
}

impl OruR01Builder {
    /// Create a new ORU^R01 builder
    pub fn new(version: Version) -> Self {
        Self {
            base: MessageBuilder::new(version, "ORU", "R01"),
            sending_app: String::new(),
            sending_facility: String::new(),
            receiving_app: String::new(),
            receiving_facility: String::new(),
            control_id: None,
            processing_id: "P".to_string(),
            patient_id: None,
            patient_name: None,
            order_control: "RE".to_string(),
            filler_order_number: None,
            observations: Vec::new(),
        }
    }

    pub fn sending_application(mut self, app: &str) -> Self {
        self.sending_app = app.to_string();
        self
    }

    pub fn sending_facility(mut self, facility: &str) -> Self {
        self.sending_facility = facility.to_string();
        self
    }

    pub fn receiving_application(mut self, app: &str) -> Self {
        self.receiving_app = app.to_string();
        self
    }

    pub fn receiving_facility(mut self, facility: &str) -> Self {
        self.receiving_facility = facility.to_string();
        self
    }

    pub fn control_id(mut self, id: &str) -> Self {
        self.control_id = Some(id.to_string());
        self
    }

    pub fn patient_id(mut self, id: &str) -> Self {
        self.patient_id = Some(id.to_string());
        self
    }

    pub fn patient_name(mut self, family: &str, given: &str) -> Self {
        self.patient_name = Some((family.to_string(), given.to_string()));
        self
    }

    pub fn filler_order_number(mut self, number: &str) -> Self {
        self.filler_order_number = Some(number.to_string());
        self
    }

    /// Add an observation
    pub fn add_observation(mut self, obs: Observation) -> Self {
        self.observations.push(obs);
        self
    }

    /// Build the message
    pub fn build(mut self) -> Result<Message> {
        let control_id = self.control_id.unwrap_or_else(generate_control_id);

        // Create MSH segment
        let msh = self.base.create_msh(
            &self.sending_app,
            &self.sending_facility,
            &self.receiving_app,
            &self.receiving_facility,
            &control_id,
            &self.processing_id,
        )?;
        self.base.message.add_segment(msh);

        // Create PID segment
        let mut pid = Segment::new("PID");
        pid.add_field(Field::from_value("1"));
        pid.add_field(Field::from_value(""));
        pid.add_field(Field::from_value(self.patient_id.as_deref().unwrap_or("")));
        pid.add_field(Field::from_value(""));

        if let Some((family, given)) = &self.patient_name {
            let name = format!("{}^{}", family, given);
            pid.add_field(Field::from_value(&name));
        } else {
            pid.add_field(Field::from_value(""));
        }

        self.base.message.add_segment(pid);

        // Create OBR segment
        let mut obr = Segment::new("OBR");
        obr.add_field(Field::from_value("1")); // OBR-1: Set ID
        obr.add_field(Field::from_value("")); // OBR-2: Placer Order Number
        obr.add_field(Field::from_value(self.filler_order_number.as_deref().unwrap_or(""))); // OBR-3: Filler Order Number
        obr.add_field(Field::from_value("")); // OBR-4: Universal Service ID

        self.base.message.add_segment(obr);

        // Create OBX segments for observations
        for obs in &self.observations {
            let mut obx = Segment::new("OBX");
            obx.add_field(Field::from_value(&obs.set_id.to_string())); // OBX-1: Set ID
            obx.add_field(Field::from_value(&obs.value_type)); // OBX-2: Value Type
            obx.add_field(Field::from_value(&obs.identifier)); // OBX-3: Observation Identifier
            obx.add_field(Field::from_value("")); // OBX-4: Observation Sub-ID
            obx.add_field(Field::from_value(&obs.value)); // OBX-5: Observation Value

            if let Some(units) = &obs.units {
                obx.add_field(Field::from_value(units)); // OBX-6: Units
            } else {
                obx.add_field(Field::from_value(""));
            }

            obx.add_field(Field::from_value("")); // OBX-7: References Range
            obx.add_field(Field::from_value("")); // OBX-8: Abnormal Flags
            obx.add_field(Field::from_value("")); // OBX-9: Probability
            obx.add_field(Field::from_value("")); // OBX-10: Nature of Abnormal Test
            obx.add_field(Field::from_value(&obs.status)); // OBX-11: Observation Result Status

            self.base.message.add_segment(obx);
        }

        Ok(self.base.build())
    }
}
