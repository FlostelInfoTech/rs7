///! Terser path parsing module
///!
///! This module handles parsing of terser path strings like "PID-5-1" or "OBX(2)-3-1"

use rs7_core::error::{Error, Result};

/// Parsed terser path
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerserPath {
    pub segment_id: String,
    pub segment_index: usize,
    pub field_index: usize,
    pub repetition_index: usize,
    pub component_index: Option<usize>,
    pub subcomponent_index: Option<usize>,
}

impl TerserPath {
    /// Parse a terser path string
    ///
    /// Format: SEGMENT[(index)]-FIELD[(rep)]-COMPONENT-SUBCOMPONENT
    ///
    /// Examples:
    /// - PID-5 (field 5 of first PID)
    /// - PID-5-1 (field 5, component 1)
    /// - PID-5-1-2 (field 5, component 1, subcomponent 2)
    /// - OBX(2)-5 (field 5 of third OBX, 0-indexed)
    /// - PID-11(1)-1 (field 11, second repetition, component 1)
    pub fn parse(path: &str) -> Result<Self> {
        let parts: Vec<&str> = path.split('-').collect();

        if parts.is_empty() {
            return Err(Error::terser_path("Empty path"));
        }

        // Parse segment part (e.g., "PID" or "OBX(2)")
        let (segment_id, segment_index) = Self::parse_segment_part(parts[0])?;

        // Parse field part (e.g., "5" or "11(1)")
        if parts.len() < 2 {
            return Err(Error::terser_path("Missing field index"));
        }

        let (field_index, repetition_index) = Self::parse_field_part(parts[1])?;

        // Parse component index (optional)
        let component_index = if parts.len() >= 3 {
            Some(Self::parse_index(parts[2])?)
        } else {
            None
        };

        // Parse subcomponent index (optional)
        let subcomponent_index = if parts.len() >= 4 {
            Some(Self::parse_index(parts[3])?)
        } else {
            None
        };

        Ok(TerserPath {
            segment_id,
            segment_index,
            field_index,
            repetition_index,
            component_index,
            subcomponent_index,
        })
    }

    /// Parse segment part (e.g., "PID" or "OBX(2)")
    fn parse_segment_part(part: &str) -> Result<(String, usize)> {
        if let Some(paren_pos) = part.find('(') {
            let id = part[..paren_pos].to_string();
            let index_str = &part[paren_pos + 1..];

            if let Some(close_paren) = index_str.find(')') {
                let index = index_str[..close_paren]
                    .parse::<usize>()
                    .map_err(|_| Error::terser_path("Invalid segment index"))?;
                Ok((id, index))
            } else {
                Err(Error::terser_path("Missing closing parenthesis"))
            }
        } else {
            Ok((part.to_string(), 0))
        }
    }

    /// Parse field part (e.g., "5" or "11(1)")
    fn parse_field_part(part: &str) -> Result<(usize, usize)> {
        if let Some(paren_pos) = part.find('(') {
            let field_str = &part[..paren_pos];
            let rep_str = &part[paren_pos + 1..];

            let field_index = field_str
                .parse::<usize>()
                .map_err(|_| Error::terser_path("Invalid field index"))?;

            if let Some(close_paren) = rep_str.find(')') {
                let rep_index = rep_str[..close_paren]
                    .parse::<usize>()
                    .map_err(|_| Error::terser_path("Invalid repetition index"))?;
                Ok((field_index, rep_index))
            } else {
                Err(Error::terser_path("Missing closing parenthesis"))
            }
        } else {
            let field_index = part
                .parse::<usize>()
                .map_err(|_| Error::terser_path("Invalid field index"))?;
            Ok((field_index, 0))
        }
    }

    /// Parse a simple numeric index
    fn parse_index(part: &str) -> Result<usize> {
        part.parse::<usize>()
            .map_err(|_| Error::terser_path("Invalid index"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_field() {
        let path = TerserPath::parse("PID-5").unwrap();
        assert_eq!(path.segment_id, "PID");
        assert_eq!(path.segment_index, 0);
        assert_eq!(path.field_index, 5);
        assert_eq!(path.repetition_index, 0);
        assert_eq!(path.component_index, None);
        assert_eq!(path.subcomponent_index, None);
    }

    #[test]
    fn test_parse_with_component() {
        let path = TerserPath::parse("PID-5-1").unwrap();
        assert_eq!(path.segment_id, "PID");
        assert_eq!(path.field_index, 5);
        assert_eq!(path.component_index, Some(1));
    }

    #[test]
    fn test_parse_with_subcomponent() {
        let path = TerserPath::parse("PID-5-1-2").unwrap();
        assert_eq!(path.component_index, Some(1));
        assert_eq!(path.subcomponent_index, Some(2));
    }

    #[test]
    fn test_parse_segment_index() {
        let path = TerserPath::parse("OBX(2)-5").unwrap();
        assert_eq!(path.segment_id, "OBX");
        assert_eq!(path.segment_index, 2);
        assert_eq!(path.field_index, 5);
    }

    #[test]
    fn test_parse_repetition_index() {
        let path = TerserPath::parse("PID-11(1)-1").unwrap();
        assert_eq!(path.field_index, 11);
        assert_eq!(path.repetition_index, 1);
        assert_eq!(path.component_index, Some(1));
    }
}
