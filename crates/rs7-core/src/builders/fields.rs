//! Complex field builders for HL7 composite data types
//!
//! This module provides builder patterns for creating complex HL7 fields with multiple components.
//! These builders help construct properly formatted composite fields following HL7 v2.x standards.

use crate::field::Field;

/// Builder for XPN (Extended Person Name) data type
///
/// XPN structure (components separated by ^):
/// 1. Family Name
/// 2. Given Name
/// 3. Second/Middle Name
/// 4. Suffix (e.g., Jr., Sr., III)
/// 5. Prefix (e.g., Dr., Mr., Mrs.)
/// 6. Degree (e.g., MD, PhD)
/// 7. Name Type Code (L=Legal, A=Alias, etc.)
/// 8. Name Representation Code
/// 9. Name Context
/// 10. Name Validity Range
/// 11. Name Assembly Order
/// 12. Effective Date
/// 13. Expiration Date
/// 14. Professional Suffix
///
/// # Example
/// ```
/// use rs7_core::builders::fields::XpnBuilder;
///
/// let name = XpnBuilder::new()
///     .family_name("DOE")
///     .given_name("JOHN")
///     .middle_name("ALBERT")
///     .suffix("JR")
///     .prefix("DR")
///     .degree("MD")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct XpnBuilder {
    family_name: Option<String>,
    given_name: Option<String>,
    middle_name: Option<String>,
    suffix: Option<String>,
    prefix: Option<String>,
    degree: Option<String>,
    name_type_code: Option<String>,
    name_representation_code: Option<String>,
}

impl XpnBuilder {
    /// Create a new XPN builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set family name (last name)
    pub fn family_name(mut self, name: &str) -> Self {
        self.family_name = Some(name.to_string());
        self
    }

    /// Set given name (first name)
    pub fn given_name(mut self, name: &str) -> Self {
        self.given_name = Some(name.to_string());
        self
    }

    /// Set middle name or second name
    pub fn middle_name(mut self, name: &str) -> Self {
        self.middle_name = Some(name.to_string());
        self
    }

    /// Set name suffix (Jr., Sr., III, etc.)
    pub fn suffix(mut self, suffix: &str) -> Self {
        self.suffix = Some(suffix.to_string());
        self
    }

    /// Set name prefix (Dr., Mr., Mrs., etc.)
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Set degree (MD, PhD, etc.)
    pub fn degree(mut self, degree: &str) -> Self {
        self.degree = Some(degree.to_string());
        self
    }

    /// Set name type code (L=Legal, A=Alias, etc.)
    pub fn name_type_code(mut self, code: &str) -> Self {
        self.name_type_code = Some(code.to_string());
        self
    }

    /// Set name representation code
    pub fn name_representation_code(mut self, code: &str) -> Self {
        self.name_representation_code = Some(code.to_string());
        self
    }

    /// Build the XPN field
    pub fn build(self) -> String {
        let components = vec![
            self.family_name.unwrap_or_default(),
            self.given_name.unwrap_or_default(),
            self.middle_name.unwrap_or_default(),
            self.suffix.unwrap_or_default(),
            self.prefix.unwrap_or_default(),
            self.degree.unwrap_or_default(),
            self.name_type_code.unwrap_or_default(),
            self.name_representation_code.unwrap_or_default(),
        ];

        // Trim trailing empty components
        let mut result = components.join("^");
        while result.ends_with('^') {
            result.pop();
        }
        result
    }

    /// Build as Field
    pub fn build_field(self) -> Field {
        Field::from_value(&self.build())
    }
}

/// Builder for XAD (Extended Address) data type
///
/// XAD structure (components separated by ^):
/// 1. Street Address (with & for sub-components)
/// 2. Other Designation
/// 3. City
/// 4. State or Province
/// 5. Zip or Postal Code
/// 6. Country
/// 7. Address Type (H=Home, O=Office, etc.)
/// 8. Other Geographic Designation
/// 9. County/Parish Code
/// 10. Census Tract
/// 11. Address Representation Code
/// 12. Address Validity Range
///
/// # Example
/// ```
/// use rs7_core::builders::fields::XadBuilder;
///
/// let address = XadBuilder::new()
///     .street_address("123 Main St")
///     .city("Springfield")
///     .state("IL")
///     .postal_code("62701")
///     .country("USA")
///     .address_type("H")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct XadBuilder {
    street_address: Option<String>,
    other_designation: Option<String>,
    city: Option<String>,
    state: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    address_type: Option<String>,
    other_geographic_designation: Option<String>,
    county_parish_code: Option<String>,
}

impl XadBuilder {
    /// Create a new XAD builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set street address (line 1)
    pub fn street_address(mut self, address: &str) -> Self {
        self.street_address = Some(address.to_string());
        self
    }

    /// Set other designation (line 2)
    pub fn other_designation(mut self, designation: &str) -> Self {
        self.other_designation = Some(designation.to_string());
        self
    }

    /// Set city
    pub fn city(mut self, city: &str) -> Self {
        self.city = Some(city.to_string());
        self
    }

    /// Set state or province
    pub fn state(mut self, state: &str) -> Self {
        self.state = Some(state.to_string());
        self
    }

    /// Set ZIP or postal code
    pub fn postal_code(mut self, code: &str) -> Self {
        self.postal_code = Some(code.to_string());
        self
    }

    /// Set country
    pub fn country(mut self, country: &str) -> Self {
        self.country = Some(country.to_string());
        self
    }

    /// Set address type (H=Home, O=Office, B=Billing, etc.)
    pub fn address_type(mut self, addr_type: &str) -> Self {
        self.address_type = Some(addr_type.to_string());
        self
    }

    /// Set other geographic designation
    pub fn other_geographic_designation(mut self, designation: &str) -> Self {
        self.other_geographic_designation = Some(designation.to_string());
        self
    }

    /// Set county/parish code
    pub fn county_parish_code(mut self, code: &str) -> Self {
        self.county_parish_code = Some(code.to_string());
        self
    }

    /// Build the XAD field
    pub fn build(self) -> String {
        let components = vec![
            self.street_address.unwrap_or_default(),
            self.other_designation.unwrap_or_default(),
            self.city.unwrap_or_default(),
            self.state.unwrap_or_default(),
            self.postal_code.unwrap_or_default(),
            self.country.unwrap_or_default(),
            self.address_type.unwrap_or_default(),
            self.other_geographic_designation.unwrap_or_default(),
            self.county_parish_code.unwrap_or_default(),
        ];

        // Trim trailing empty components
        let mut result = components.join("^");
        while result.ends_with('^') {
            result.pop();
        }
        result
    }

    /// Build as Field
    pub fn build_field(self) -> Field {
        Field::from_value(&self.build())
    }
}

/// Builder for XTN (Extended Telecommunication Number) data type
///
/// XTN structure (components separated by ^):
/// 1. Telephone Number (formatted or unformatted)
/// 2. Telecommunication Use Code (PRN=Primary, ORN=Other, etc.)
/// 3. Telecommunication Equipment Type (PH=Phone, FX=Fax, etc.)
/// 4. Email Address
/// 5. Country Code
/// 6. Area/City Code
/// 7. Local Number
/// 8. Extension
/// 9. Any Text
///
/// # Example
/// ```
/// use rs7_core::builders::fields::XtnBuilder;
///
/// let phone = XtnBuilder::new()
///     .phone_number("(555) 123-4567")
///     .use_code("PRN")
///     .equipment_type("PH")
///     .build();
///
/// let email = XtnBuilder::new()
///     .email("john.doe@example.com")
///     .use_code("NET")
///     .equipment_type("Internet")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct XtnBuilder {
    phone_number: Option<String>,
    use_code: Option<String>,
    equipment_type: Option<String>,
    email: Option<String>,
    country_code: Option<String>,
    area_code: Option<String>,
    local_number: Option<String>,
    extension: Option<String>,
    any_text: Option<String>,
}

impl XtnBuilder {
    /// Create a new XTN builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set formatted or unformatted phone number
    pub fn phone_number(mut self, number: &str) -> Self {
        self.phone_number = Some(number.to_string());
        self
    }

    /// Set telecommunication use code (PRN=Primary, ORN=Other Residence, WPN=Work, VHN=Vacation Home, etc.)
    pub fn use_code(mut self, code: &str) -> Self {
        self.use_code = Some(code.to_string());
        self
    }

    /// Set equipment type (PH=Phone, FX=Fax, CP=Cell Phone, Internet=Email, etc.)
    pub fn equipment_type(mut self, eq_type: &str) -> Self {
        self.equipment_type = Some(eq_type.to_string());
        self
    }

    /// Set email address
    pub fn email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }

    /// Set country code
    pub fn country_code(mut self, code: &str) -> Self {
        self.country_code = Some(code.to_string());
        self
    }

    /// Set area/city code
    pub fn area_code(mut self, code: &str) -> Self {
        self.area_code = Some(code.to_string());
        self
    }

    /// Set local number
    pub fn local_number(mut self, number: &str) -> Self {
        self.local_number = Some(number.to_string());
        self
    }

    /// Set extension
    pub fn extension(mut self, ext: &str) -> Self {
        self.extension = Some(ext.to_string());
        self
    }

    /// Set any text
    pub fn any_text(mut self, text: &str) -> Self {
        self.any_text = Some(text.to_string());
        self
    }

    /// Build the XTN field
    pub fn build(self) -> String {
        let components = vec![
            self.phone_number.unwrap_or_default(),
            self.use_code.unwrap_or_default(),
            self.equipment_type.unwrap_or_default(),
            self.email.unwrap_or_default(),
            self.country_code.unwrap_or_default(),
            self.area_code.unwrap_or_default(),
            self.local_number.unwrap_or_default(),
            self.extension.unwrap_or_default(),
            self.any_text.unwrap_or_default(),
        ];

        // Trim trailing empty components
        let mut result = components.join("^");
        while result.ends_with('^') {
            result.pop();
        }
        result
    }

    /// Build as Field
    pub fn build_field(self) -> Field {
        Field::from_value(&self.build())
    }
}

/// Builder for CX (Extended Composite ID with Check Digit) data type
///
/// CX structure (components separated by ^):
/// 1. ID Number
/// 2. Check Digit
/// 3. Check Digit Scheme
/// 4. Assigning Authority (HD)
/// 5. Identifier Type Code (MR=Medical Record, PI=Patient ID, etc.)
/// 6. Assigning Facility
/// 7. Effective Date
/// 8. Expiration Date
/// 9. Assigning Jurisdiction
/// 10. Assigning Agency or Department
///
/// # Example
/// ```
/// use rs7_core::builders::fields::CxBuilder;
///
/// let patient_id = CxBuilder::new("12345")
///     .identifier_type_code("MR")
///     .assigning_authority("Hospital")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct CxBuilder {
    id_number: String,
    check_digit: Option<String>,
    check_digit_scheme: Option<String>,
    assigning_authority: Option<String>,
    identifier_type_code: Option<String>,
    assigning_facility: Option<String>,
    effective_date: Option<String>,
    expiration_date: Option<String>,
}

impl CxBuilder {
    /// Create a new CX builder with required ID number
    pub fn new(id_number: &str) -> Self {
        Self {
            id_number: id_number.to_string(),
            check_digit: None,
            check_digit_scheme: None,
            assigning_authority: None,
            identifier_type_code: None,
            assigning_facility: None,
            effective_date: None,
            expiration_date: None,
        }
    }

    /// Set check digit
    pub fn check_digit(mut self, digit: &str) -> Self {
        self.check_digit = Some(digit.to_string());
        self
    }

    /// Set check digit scheme (M10=Mod 10, M11=Mod 11, etc.)
    pub fn check_digit_scheme(mut self, scheme: &str) -> Self {
        self.check_digit_scheme = Some(scheme.to_string());
        self
    }

    /// Set assigning authority
    pub fn assigning_authority(mut self, authority: &str) -> Self {
        self.assigning_authority = Some(authority.to_string());
        self
    }

    /// Set identifier type code (MR=Medical Record, PI=Patient ID, SS=Social Security, etc.)
    pub fn identifier_type_code(mut self, code: &str) -> Self {
        self.identifier_type_code = Some(code.to_string());
        self
    }

    /// Set assigning facility
    pub fn assigning_facility(mut self, facility: &str) -> Self {
        self.assigning_facility = Some(facility.to_string());
        self
    }

    /// Set effective date (YYYYMMDD format)
    pub fn effective_date(mut self, date: &str) -> Self {
        self.effective_date = Some(date.to_string());
        self
    }

    /// Set expiration date (YYYYMMDD format)
    pub fn expiration_date(mut self, date: &str) -> Self {
        self.expiration_date = Some(date.to_string());
        self
    }

    /// Build the CX field
    pub fn build(self) -> String {
        let components = vec![
            self.id_number,
            self.check_digit.unwrap_or_default(),
            self.check_digit_scheme.unwrap_or_default(),
            self.assigning_authority.unwrap_or_default(),
            self.identifier_type_code.unwrap_or_default(),
            self.assigning_facility.unwrap_or_default(),
            self.effective_date.unwrap_or_default(),
            self.expiration_date.unwrap_or_default(),
        ];

        // Trim trailing empty components
        let mut result = components.join("^");
        while result.ends_with('^') {
            result.pop();
        }
        result
    }

    /// Build as Field
    pub fn build_field(self) -> Field {
        Field::from_value(&self.build())
    }
}

/// Builder for XCN (Extended Composite ID Number and Name for Persons) data type
///
/// XCN structure (components separated by ^):
/// 1. ID Number
/// 2. Family Name
/// 3. Given Name
/// 4. Second Name/Middle Name
/// 5. Suffix
/// 6. Prefix
/// 7. Degree
/// 8. Source Table
/// 9. Assigning Authority
/// 10. Name Type Code
/// 11. Identifier Check Digit
/// 12. Check Digit Scheme
/// 13. Identifier Type Code
///
/// # Example
/// ```
/// use rs7_core::builders::fields::XcnBuilder;
///
/// let doctor = XcnBuilder::new()
///     .id_number("12345")
///     .family_name("SMITH")
///     .given_name("JAMES")
///     .prefix("DR")
///     .degree("MD")
///     .identifier_type_code("NPI")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct XcnBuilder {
    id_number: Option<String>,
    family_name: Option<String>,
    given_name: Option<String>,
    middle_name: Option<String>,
    suffix: Option<String>,
    prefix: Option<String>,
    degree: Option<String>,
    source_table: Option<String>,
    assigning_authority: Option<String>,
    name_type_code: Option<String>,
    check_digit: Option<String>,
    check_digit_scheme: Option<String>,
    identifier_type_code: Option<String>,
}

impl XcnBuilder {
    /// Create a new XCN builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set ID number
    pub fn id_number(mut self, id: &str) -> Self {
        self.id_number = Some(id.to_string());
        self
    }

    /// Set family name (last name)
    pub fn family_name(mut self, name: &str) -> Self {
        self.family_name = Some(name.to_string());
        self
    }

    /// Set given name (first name)
    pub fn given_name(mut self, name: &str) -> Self {
        self.given_name = Some(name.to_string());
        self
    }

    /// Set middle name or second name
    pub fn middle_name(mut self, name: &str) -> Self {
        self.middle_name = Some(name.to_string());
        self
    }

    /// Set name suffix (Jr., Sr., III, etc.)
    pub fn suffix(mut self, suffix: &str) -> Self {
        self.suffix = Some(suffix.to_string());
        self
    }

    /// Set name prefix (Dr., Mr., Mrs., etc.)
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Set degree (MD, PhD, etc.)
    pub fn degree(mut self, degree: &str) -> Self {
        self.degree = Some(degree.to_string());
        self
    }

    /// Set source table
    pub fn source_table(mut self, table: &str) -> Self {
        self.source_table = Some(table.to_string());
        self
    }

    /// Set assigning authority
    pub fn assigning_authority(mut self, authority: &str) -> Self {
        self.assigning_authority = Some(authority.to_string());
        self
    }

    /// Set name type code (L=Legal, A=Alias, etc.)
    pub fn name_type_code(mut self, code: &str) -> Self {
        self.name_type_code = Some(code.to_string());
        self
    }

    /// Set check digit
    pub fn check_digit(mut self, digit: &str) -> Self {
        self.check_digit = Some(digit.to_string());
        self
    }

    /// Set check digit scheme (M10=Mod 10, M11=Mod 11, etc.)
    pub fn check_digit_scheme(mut self, scheme: &str) -> Self {
        self.check_digit_scheme = Some(scheme.to_string());
        self
    }

    /// Set identifier type code (NPI, DEA, etc.)
    pub fn identifier_type_code(mut self, code: &str) -> Self {
        self.identifier_type_code = Some(code.to_string());
        self
    }

    /// Build the XCN field
    pub fn build(self) -> String {
        let components = vec![
            self.id_number.unwrap_or_default(),
            self.family_name.unwrap_or_default(),
            self.given_name.unwrap_or_default(),
            self.middle_name.unwrap_or_default(),
            self.suffix.unwrap_or_default(),
            self.prefix.unwrap_or_default(),
            self.degree.unwrap_or_default(),
            self.source_table.unwrap_or_default(),
            self.assigning_authority.unwrap_or_default(),
            self.name_type_code.unwrap_or_default(),
            self.check_digit.unwrap_or_default(),
            self.check_digit_scheme.unwrap_or_default(),
            self.identifier_type_code.unwrap_or_default(),
        ];

        // Trim trailing empty components
        let mut result = components.join("^");
        while result.ends_with('^') {
            result.pop();
        }
        result
    }

    /// Build as Field
    pub fn build_field(self) -> Field {
        Field::from_value(&self.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xpn_builder_basic() {
        let name = XpnBuilder::new()
            .family_name("DOE")
            .given_name("JOHN")
            .build();
        assert_eq!(name, "DOE^JOHN");
    }

    #[test]
    fn test_xpn_builder_full() {
        let name = XpnBuilder::new()
            .family_name("DOE")
            .given_name("JOHN")
            .middle_name("ALBERT")
            .suffix("JR")
            .prefix("DR")
            .degree("MD")
            .name_type_code("L")
            .build();
        assert_eq!(name, "DOE^JOHN^ALBERT^JR^DR^MD^L");
    }

    #[test]
    fn test_xad_builder() {
        let address = XadBuilder::new()
            .street_address("123 Main St")
            .city("Springfield")
            .state("IL")
            .postal_code("62701")
            .country("USA")
            .address_type("H")
            .build();
        assert_eq!(address, "123 Main St^^Springfield^IL^62701^USA^H");
    }

    #[test]
    fn test_xtn_builder_phone() {
        let phone = XtnBuilder::new()
            .phone_number("(555) 123-4567")
            .use_code("PRN")
            .equipment_type("PH")
            .build();
        assert_eq!(phone, "(555) 123-4567^PRN^PH");
    }

    #[test]
    fn test_xtn_builder_email() {
        let email = XtnBuilder::new()
            .email("john.doe@example.com")
            .use_code("NET")
            .equipment_type("Internet")
            .build();
        assert_eq!(email, "^NET^Internet^john.doe@example.com");
    }

    #[test]
    fn test_cx_builder() {
        let patient_id = CxBuilder::new("12345")
            .identifier_type_code("MR")
            .assigning_authority("Hospital")
            .build();
        assert_eq!(patient_id, "12345^^^Hospital^MR");
    }

    #[test]
    fn test_xcn_builder() {
        let doctor = XcnBuilder::new()
            .id_number("12345")
            .family_name("SMITH")
            .given_name("JAMES")
            .prefix("DR")
            .degree("MD")
            .identifier_type_code("NPI")
            .build();
        assert_eq!(doctor, "12345^SMITH^JAMES^^^DR^MD^^^^^^NPI");
    }
}
