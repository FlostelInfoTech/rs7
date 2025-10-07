//! ADT (Admit/Discharge/Transfer) message builders

use super::{generate_control_id, MessageBuilder};
use crate::{
    error::Result,
    field::Field,
    message::Message,
    segment::Segment,
    Version,
};

/// Builder for ADT messages
pub struct AdtBuilder {
    base: MessageBuilder,
}

impl AdtBuilder {
    /// Create a new ADT message builder
    pub fn new(version: Version, trigger_event: &str) -> Self {
        Self {
            base: MessageBuilder::new(version, "ADT", trigger_event),
        }
    }

    /// Create ADT^A01 - Admit/Visit Notification
    pub fn a01(version: Version) -> AdtA01Builder {
        AdtA01Builder::new(version)
    }

    /// Create ADT^A02 - Transfer a Patient
    pub fn a02(version: Version) -> AdtA02Builder {
        AdtA02Builder::new(version)
    }

    /// Create ADT^A03 - Discharge/End Visit
    pub fn a03(version: Version) -> AdtA03Builder {
        AdtA03Builder::new(version)
    }

    /// Create ADT^A04 - Register a Patient
    pub fn a04(version: Version) -> AdtA04Builder {
        AdtA04Builder::new(version)
    }

    /// Create ADT^A08 - Update Patient Information
    pub fn a08(version: Version) -> AdtA08Builder {
        AdtA08Builder::new(version)
    }
}

/// Builder for ADT^A01 - Admit/Visit Notification
pub struct AdtA01Builder {
    base: MessageBuilder,
    sending_app: String,
    sending_facility: String,
    receiving_app: String,
    receiving_facility: String,
    control_id: Option<String>,
    processing_id: String,
    patient_id: Option<String>,
    patient_name: Option<(String, String)>, // (family, given)
    dob: Option<String>,
    sex: Option<String>,
    patient_class: Option<String>,
    assigned_location: Option<String>,
    attending_doctor: Option<String>,
    admit_datetime: Option<String>,
}

impl AdtA01Builder {
    /// Create a new ADT^A01 builder
    pub fn new(version: Version) -> Self {
        Self {
            base: MessageBuilder::new(version, "ADT", "A01"),
            sending_app: String::new(),
            sending_facility: String::new(),
            receiving_app: String::new(),
            receiving_facility: String::new(),
            control_id: None,
            processing_id: "P".to_string(),
            patient_id: None,
            patient_name: None,
            dob: None,
            sex: None,
            patient_class: None,
            assigned_location: None,
            attending_doctor: None,
            admit_datetime: None,
        }
    }

    /// Set sending application
    pub fn sending_application(mut self, app: &str) -> Self {
        self.sending_app = app.to_string();
        self
    }

    /// Set sending facility
    pub fn sending_facility(mut self, facility: &str) -> Self {
        self.sending_facility = facility.to_string();
        self
    }

    /// Set receiving application
    pub fn receiving_application(mut self, app: &str) -> Self {
        self.receiving_app = app.to_string();
        self
    }

    /// Set receiving facility
    pub fn receiving_facility(mut self, facility: &str) -> Self {
        self.receiving_facility = facility.to_string();
        self
    }

    /// Set message control ID (auto-generated if not set)
    pub fn control_id(mut self, id: &str) -> Self {
        self.control_id = Some(id.to_string());
        self
    }

    /// Set processing ID (defaults to "P" for Production)
    pub fn processing_id(mut self, id: &str) -> Self {
        self.processing_id = id.to_string();
        self
    }

    /// Set patient ID
    pub fn patient_id(mut self, id: &str) -> Self {
        self.patient_id = Some(id.to_string());
        self
    }

    /// Set patient name
    pub fn patient_name(mut self, family: &str, given: &str) -> Self {
        self.patient_name = Some((family.to_string(), given.to_string()));
        self
    }

    /// Set date of birth (format: YYYYMMDD)
    pub fn date_of_birth(mut self, dob: &str) -> Self {
        self.dob = Some(dob.to_string());
        self
    }

    /// Set administrative sex
    pub fn sex(mut self, sex: &str) -> Self {
        self.sex = Some(sex.to_string());
        self
    }

    /// Set patient class (I=Inpatient, O=Outpatient, E=Emergency, etc.)
    pub fn patient_class(mut self, class: &str) -> Self {
        self.patient_class = Some(class.to_string());
        self
    }

    /// Set assigned patient location
    pub fn assigned_location(mut self, location: &str) -> Self {
        self.assigned_location = Some(location.to_string());
        self
    }

    /// Set attending doctor
    pub fn attending_doctor(mut self, doctor: &str) -> Self {
        self.attending_doctor = Some(doctor.to_string());
        self
    }

    /// Set admit date/time
    pub fn admit_datetime(mut self, datetime: &str) -> Self {
        self.admit_datetime = Some(datetime.to_string());
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

        // Create EVN segment
        let evn = self.base.create_evn("A01", None)?;
        self.base.message.add_segment(evn);

        // Create PID segment
        let mut pid = Segment::new("PID");
        pid.add_field(Field::from_value("1")); // PID-1: Set ID

        // PID-2: Patient ID (External) - often empty
        pid.add_field(Field::from_value(""));

        // PID-3: Patient Identifier List
        if let Some(id) = &self.patient_id {
            pid.add_field(Field::from_value(id));
        } else {
            pid.add_field(Field::from_value(""));
        }

        // PID-4: Alternate Patient ID - empty
        pid.add_field(Field::from_value(""));

        // PID-5: Patient Name
        if let Some((family, given)) = &self.patient_name {
            let name = format!("{}^{}", family, given);
            pid.add_field(Field::from_value(&name));
        } else {
            pid.add_field(Field::from_value(""));
        }

        // PID-6: Mother's Maiden Name - empty
        pid.add_field(Field::from_value(""));

        // PID-7: Date of Birth
        pid.add_field(Field::from_value(self.dob.as_deref().unwrap_or("")));

        // PID-8: Sex
        pid.add_field(Field::from_value(self.sex.as_deref().unwrap_or("")));

        self.base.message.add_segment(pid);

        // Create PV1 segment if patient class is provided
        if let Some(class) = &self.patient_class {
            let mut pv1 = Segment::new("PV1");
            pv1.add_field(Field::from_value("1")); // PV1-1: Set ID
            pv1.add_field(Field::from_value(class)); // PV1-2: Patient Class

            // PV1-3: Assigned Patient Location
            pv1.add_field(Field::from_value(self.assigned_location.as_deref().unwrap_or("")));

            // PV1-4: Admission Type - empty for now
            pv1.add_field(Field::from_value(""));

            // PV1-5: Preadmit Number - empty
            pv1.add_field(Field::from_value(""));

            // PV1-6: Prior Patient Location - empty
            pv1.add_field(Field::from_value(""));

            // PV1-7: Attending Doctor
            pv1.add_field(Field::from_value(self.attending_doctor.as_deref().unwrap_or("")));

            self.base.message.add_segment(pv1);
        }

        Ok(self.base.build())
    }
}

/// Builder for ADT^A02 - Transfer a Patient
pub struct AdtA02Builder {
    base: AdtA01Builder,
}

impl AdtA02Builder {
    pub fn new(version: Version) -> Self {
        let mut base = AdtA01Builder::new(version);
        base.base = MessageBuilder::new(version, "ADT", "A02");
        Self { base }
    }

    // Delegate all methods to AdtA01Builder
    pub fn sending_application(mut self, app: &str) -> Self {
        self.base = self.base.sending_application(app);
        self
    }

    pub fn sending_facility(mut self, facility: &str) -> Self {
        self.base = self.base.sending_facility(facility);
        self
    }

    pub fn receiving_application(mut self, app: &str) -> Self {
        self.base = self.base.receiving_application(app);
        self
    }

    pub fn receiving_facility(mut self, facility: &str) -> Self {
        self.base = self.base.receiving_facility(facility);
        self
    }

    pub fn patient_id(mut self, id: &str) -> Self {
        self.base = self.base.patient_id(id);
        self
    }

    pub fn patient_name(mut self, family: &str, given: &str) -> Self {
        self.base = self.base.patient_name(family, given);
        self
    }

    pub fn patient_class(mut self, class: &str) -> Self {
        self.base = self.base.patient_class(class);
        self
    }

    pub fn assigned_location(mut self, location: &str) -> Self {
        self.base = self.base.assigned_location(location);
        self
    }

    pub fn build(self) -> Result<Message> {
        self.base.build()
    }
}

/// Builder for ADT^A03 - Discharge/End Visit
pub struct AdtA03Builder {
    base: AdtA01Builder,
}

impl AdtA03Builder {
    pub fn new(version: Version) -> Self {
        let mut base = AdtA01Builder::new(version);
        base.base = MessageBuilder::new(version, "ADT", "A03");
        Self { base }
    }

    pub fn sending_application(mut self, app: &str) -> Self {
        self.base = self.base.sending_application(app);
        self
    }

    pub fn sending_facility(mut self, facility: &str) -> Self {
        self.base = self.base.sending_facility(facility);
        self
    }

    pub fn receiving_application(mut self, app: &str) -> Self {
        self.base = self.base.receiving_application(app);
        self
    }

    pub fn receiving_facility(mut self, facility: &str) -> Self {
        self.base = self.base.receiving_facility(facility);
        self
    }

    pub fn patient_id(mut self, id: &str) -> Self {
        self.base = self.base.patient_id(id);
        self
    }

    pub fn patient_name(mut self, family: &str, given: &str) -> Self {
        self.base = self.base.patient_name(family, given);
        self
    }

    pub fn patient_class(mut self, class: &str) -> Self {
        self.base = self.base.patient_class(class);
        self
    }

    pub fn build(self) -> Result<Message> {
        self.base.build()
    }
}

/// Builder for ADT^A04 - Register a Patient
pub struct AdtA04Builder {
    base: AdtA01Builder,
}

impl AdtA04Builder {
    pub fn new(version: Version) -> Self {
        let mut base = AdtA01Builder::new(version);
        base.base = MessageBuilder::new(version, "ADT", "A04");
        Self { base }
    }

    pub fn sending_application(mut self, app: &str) -> Self {
        self.base = self.base.sending_application(app);
        self
    }

    pub fn sending_facility(mut self, facility: &str) -> Self {
        self.base = self.base.sending_facility(facility);
        self
    }

    pub fn receiving_application(mut self, app: &str) -> Self {
        self.base = self.base.receiving_application(app);
        self
    }

    pub fn receiving_facility(mut self, facility: &str) -> Self {
        self.base = self.base.receiving_facility(facility);
        self
    }

    pub fn patient_id(mut self, id: &str) -> Self {
        self.base = self.base.patient_id(id);
        self
    }

    pub fn patient_name(mut self, family: &str, given: &str) -> Self {
        self.base = self.base.patient_name(family, given);
        self
    }

    pub fn date_of_birth(mut self, dob: &str) -> Self {
        self.base = self.base.date_of_birth(dob);
        self
    }

    pub fn sex(mut self, sex: &str) -> Self {
        self.base = self.base.sex(sex);
        self
    }

    pub fn build(self) -> Result<Message> {
        self.base.build()
    }
}

/// Builder for ADT^A08 - Update Patient Information
pub struct AdtA08Builder {
    base: AdtA01Builder,
}

impl AdtA08Builder {
    pub fn new(version: Version) -> Self {
        let mut base = AdtA01Builder::new(version);
        base.base = MessageBuilder::new(version, "ADT", "A08");
        Self { base }
    }

    pub fn sending_application(mut self, app: &str) -> Self {
        self.base = self.base.sending_application(app);
        self
    }

    pub fn sending_facility(mut self, facility: &str) -> Self {
        self.base = self.base.sending_facility(facility);
        self
    }

    pub fn receiving_application(mut self, app: &str) -> Self {
        self.base = self.base.receiving_application(app);
        self
    }

    pub fn receiving_facility(mut self, facility: &str) -> Self {
        self.base = self.base.receiving_facility(facility);
        self
    }

    pub fn patient_id(mut self, id: &str) -> Self {
        self.base = self.base.patient_id(id);
        self
    }

    pub fn patient_name(mut self, family: &str, given: &str) -> Self {
        self.base = self.base.patient_name(family, given);
        self
    }

    pub fn date_of_birth(mut self, dob: &str) -> Self {
        self.base = self.base.date_of_birth(dob);
        self
    }

    pub fn sex(mut self, sex: &str) -> Self {
        self.base = self.base.sex(sex);
        self
    }

    pub fn build(self) -> Result<Message> {
        self.base.build()
    }
}
