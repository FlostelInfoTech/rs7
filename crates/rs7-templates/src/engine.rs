//! Template engine for creating messages from templates.

use crate::{Error, MessageTemplate, Result};
use rs7_core::{Component, Delimiters, Field, Message, Repetition, Segment};
use std::collections::HashMap;

/// Template engine for creating messages from templates
pub struct TemplateEngine {
    /// Variables for substitution
    variables: HashMap<String, String>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Set a variable value
    pub fn set_variable(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }

    /// Set multiple variables
    pub fn set_variables(&mut self, variables: HashMap<String, String>) {
        self.variables.extend(variables);
    }

    /// Get a variable value
    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Create a message from a template
    pub fn create_message(&self, template: &MessageTemplate) -> Result<Message> {
        // Merge template default variables with instance variables
        let mut all_vars = template.variables.clone().unwrap_or_default();
        all_vars.extend(self.variables.clone());

        // Create message with segments
        let mut segments = Vec::new();

        for seg_template in &template.segments {
            let segment = self.create_segment(seg_template, &all_vars)?;
            segments.push(segment);
        }

        // Create the message
        let message = Message {
            segments,
            delimiters: Delimiters::default(),
        };

        Ok(message)
    }

    /// Create a segment from a segment template
    fn create_segment(
        &self,
        seg_template: &crate::SegmentTemplate,
        variables: &HashMap<String, String>,
    ) -> Result<Segment> {
        let mut fields = Vec::new();

        // Segment ID is field 0
        fields.push(Field::from_value(seg_template.id.clone()));

        // Process field templates if present
        if let Some(field_templates) = &seg_template.fields {
            // Get the maximum field position to size the fields vector
            let max_pos = field_templates.keys().max().copied().unwrap_or(0);

            // Create fields up to max position (1-indexed in template, 0-indexed in vector)
            for pos in 1..=max_pos {
                if let Some(field_template) = field_templates.get(&pos) {
                    let field = self.create_field(field_template, variables)?;
                    fields.push(field);
                } else {
                    // Empty field for positions without templates
                    fields.push(Field::new());
                }
            }
        }

        Ok(Segment {
            id: seg_template.id.clone(),
            fields,
        })
    }

    /// Create a field from a field template
    fn create_field(
        &self,
        field_template: &crate::FieldTemplate,
        variables: &HashMap<String, String>,
    ) -> Result<Field> {
        // Handle component templates if present
        if let Some(components) = &field_template.components {
            let mut comp_vec = Vec::new();

            for comp_template in components {
                let comp_value = if let Some(placeholder) = &comp_template.placeholder {
                    self.substitute_variables(placeholder, variables)?
                } else if let Some(default) = &comp_template.default {
                    default.clone()
                } else if comp_template.required {
                    return Err(Error::substitution(
                        "Required component has no placeholder or default value",
                    ));
                } else {
                    String::new()
                };

                comp_vec.push(Component::from_value(comp_value));
            }

            // Create a field with one repetition containing the components
            let mut field = Field::new();
            field.add_repetition(Repetition { components: comp_vec });
            Ok(field)
        } else {
            // Simple field with single value
            let value = if let Some(placeholder) = &field_template.placeholder {
                self.substitute_variables(placeholder, variables)?
            } else if let Some(default) = &field_template.default {
                default.clone()
            } else if field_template.required {
                return Err(Error::substitution(
                    "Required field has no placeholder or default value",
                ));
            } else {
                return Ok(Field::new()); // Empty field
            };

            Ok(Field::from_value(value))
        }
    }

    /// Substitute variables in a placeholder string
    /// Supports {{variable_name}} syntax
    fn substitute_variables(
        &self,
        placeholder: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String> {
        let mut result = placeholder.to_string();

        // Find all {{variable}} patterns
        let re = regex::Regex::new(r"\{\{([^}]+)\}\}")
            .map_err(|e| Error::substitution(format!("Regex error: {}", e)))?;

        for cap in re.captures_iter(placeholder) {
            let var_name = &cap[1].trim();
            let var_value = variables
                .get(*var_name)
                .ok_or_else(|| Error::substitution(format!("Variable '{}' not found", var_name)))?;

            result = result.replace(&format!("{{{{{}}}}}", var_name), var_value);
        }

        Ok(result)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentTemplate, FieldTemplate, SegmentTemplate};

    #[test]
    fn test_variable_substitution() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "John Doe".to_string());
        vars.insert("id".to_string(), "12345".to_string());

        let result = engine.substitute_variables("Patient {{name}} has ID {{id}}", &vars);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Patient John Doe has ID 12345");
    }

    #[test]
    fn test_missing_variable() {
        let engine = TemplateEngine::new();
        let vars = HashMap::new();

        let result = engine.substitute_variables("Patient {{name}}", &vars);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Variable 'name' not found"));
    }

    #[test]
    fn test_create_simple_message() {
        let mut template = MessageTemplate::new("Test", "2.5", "ADT", "A01");

        // Add MSH segment with sending application
        let mut msh_segment = SegmentTemplate::new("MSH").required();
        msh_segment.add_field(
            3,
            FieldTemplate::new()
                .required()
                .with_placeholder("{{sending_app}}"),
        );
        template.add_segment(msh_segment);

        // Create engine with variables
        let mut engine = TemplateEngine::new();
        engine.set_variable("sending_app", "TestApp");

        let message = engine.create_message(&template);
        assert!(message.is_ok());

        let msg = message.unwrap();
        assert_eq!(msg.segments.len(), 1);
        assert_eq!(msg.segments[0].id, "MSH");
        assert_eq!(msg.segments[0].fields[3].value(), Some("TestApp"));
    }

    #[test]
    fn test_create_message_with_default_values() {
        let mut template = MessageTemplate::new("Test", "2.5", "ADT", "A01");

        // Add PID segment with default sex
        let mut pid_segment = SegmentTemplate::new("PID");
        pid_segment.add_field(
            8,
            FieldTemplate::new().with_default("U"), // Unknown
        );
        template.add_segment(pid_segment);

        let engine = TemplateEngine::new();
        let message = engine.create_message(&template);
        assert!(message.is_ok());

        let msg = message.unwrap();
        assert_eq!(msg.segments[0].fields[8].value(), Some("U"));
    }

    #[test]
    fn test_create_message_with_components() {
        let mut template = MessageTemplate::new("Test", "2.5", "ADT", "A01");

        // Add PID segment with patient name components
        let mut pid_segment = SegmentTemplate::new("PID");

        let name_field = FieldTemplate::new()
            .required()
            .with_components(vec![
                ComponentTemplate::new(1)
                    .required()
                    .with_placeholder("{{last_name}}"),
                ComponentTemplate::new(2)
                    .required()
                    .with_placeholder("{{first_name}}"),
            ]);

        pid_segment.add_field(5, name_field);
        template.add_segment(pid_segment);

        let mut engine = TemplateEngine::new();
        engine.set_variable("last_name", "Doe");
        engine.set_variable("first_name", "John");

        let message = engine.create_message(&template);
        assert!(message.is_ok());

        let msg = message.unwrap();
        assert_eq!(msg.segments[0].fields[5].repetitions.len(), 1);
        assert_eq!(msg.segments[0].fields[5].repetitions[0].components.len(), 2);
        assert_eq!(
            msg.segments[0].fields[5].repetitions[0].components[0].value(),
            Some("Doe")
        );
        assert_eq!(
            msg.segments[0].fields[5].repetitions[0].components[1].value(),
            Some("John")
        );
    }

    #[test]
    fn test_missing_required_field() {
        let mut template = MessageTemplate::new("Test", "2.5", "ADT", "A01");

        let mut pid_segment = SegmentTemplate::new("PID");
        pid_segment.add_field(
            3,
            FieldTemplate::new().required(), // No placeholder or default
        );
        template.add_segment(pid_segment);

        let engine = TemplateEngine::new();
        let result = engine.create_message(&template);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_default_variables() {
        let mut template = MessageTemplate::new("Test", "2.5", "ADT", "A01");

        // Set template-level default variable
        template.add_variable("facility", "DefaultHospital");

        let mut msh_segment = SegmentTemplate::new("MSH");
        msh_segment.add_field(
            4,
            FieldTemplate::new().with_placeholder("{{facility}}"),
        );
        template.add_segment(msh_segment);

        // Create engine without setting facility variable
        let engine = TemplateEngine::new();
        let message = engine.create_message(&template);

        assert!(message.is_ok());
        let msg = message.unwrap();
        assert_eq!(msg.segments[0].fields[4].value(), Some("DefaultHospital"));
    }

    #[test]
    fn test_engine_variables_override_template_defaults() {
        let mut template = MessageTemplate::new("Test", "2.5", "ADT", "A01");
        template.add_variable("facility", "DefaultHospital");

        let mut msh_segment = SegmentTemplate::new("MSH");
        msh_segment.add_field(
            4,
            FieldTemplate::new().with_placeholder("{{facility}}"),
        );
        template.add_segment(msh_segment);

        // Engine variable should override template default
        let mut engine = TemplateEngine::new();
        engine.set_variable("facility", "CustomHospital");

        let message = engine.create_message(&template);
        assert!(message.is_ok());

        let msg = message.unwrap();
        assert_eq!(msg.segments[0].fields[4].value(), Some("CustomHospital"));
    }
}
