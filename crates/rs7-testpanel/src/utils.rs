//! Utility functions for the RS7 Test Panel

use rs7_core::Message;

/// Format a message as a tree structure for display
pub fn format_message_tree(message: &Message) -> Vec<TreeNode> {
    let mut nodes = Vec::new();

    for (seg_idx, segment) in message.segments.iter().enumerate() {
        let mut segment_node = TreeNode {
            label: format!("{} (Segment {})", segment.id, seg_idx + 1),
            children: Vec::new(),
            expanded: seg_idx == 0, // Expand first segment by default
        };

        for (field_idx, field) in segment.fields.iter().enumerate() {
            if field_idx == 0 && segment.id == "MSH" {
                // Skip MSH-1 (field separator)
                continue;
            }

            let field_value = field.value().unwrap_or("");
            if !field_value.is_empty() {
                let field_label = format!(
                    "{}-{}: {}",
                    segment.id,
                    if segment.id == "MSH" { field_idx } else { field_idx + 1 },
                    truncate_string(field_value, 60)
                );

                let mut field_node = TreeNode {
                    label: field_label,
                    children: Vec::new(),
                    expanded: false,
                };

                // Add repetitions if present
                for (rep_idx, repetition) in field.repetitions.iter().enumerate() {
                    if field.repetitions.len() > 1 {
                        let rep_value = repetition.value().unwrap_or("");
                        if !rep_value.is_empty() {
                            let rep_label = format!(
                                "Rep {}: {}",
                                rep_idx + 1,
                                truncate_string(rep_value, 50)
                            );

                            let mut rep_node = TreeNode {
                                label: rep_label,
                                children: Vec::new(),
                                expanded: false,
                            };

                            // Add components
                            add_component_nodes(&mut rep_node, repetition);
                            field_node.children.push(rep_node);
                        }
                    } else {
                        // Single repetition - add components directly
                        add_component_nodes(&mut field_node, repetition);
                    }
                }

                segment_node.children.push(field_node);
            }
        }

        nodes.push(segment_node);
    }

    nodes
}

fn add_component_nodes(parent: &mut TreeNode, repetition: &rs7_core::Repetition) {
    if repetition.components.len() > 1 {
        for (comp_idx, component) in repetition.components.iter().enumerate() {
            let comp_value = component.value().unwrap_or("");
            if !comp_value.is_empty() {
                let comp_label = format!(
                    "Comp {}: {}",
                    comp_idx + 1,
                    truncate_string(comp_value, 40)
                );

                let mut comp_node = TreeNode {
                    label: comp_label,
                    children: Vec::new(),
                    expanded: false,
                };

                // Add subcomponents if present
                if component.subcomponents.len() > 1 {
                    for (sub_idx, subcomp) in component.subcomponents.iter().enumerate() {
                        if !subcomp.value.is_empty() {
                            comp_node.children.push(TreeNode {
                                label: format!("Sub {}: {}", sub_idx + 1, &subcomp.value),
                                children: Vec::new(),
                                expanded: false,
                            });
                        }
                    }
                }

                parent.children.push(comp_node);
            }
        }
    }
}

/// A tree node for hierarchical display
#[derive(Clone)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

impl TreeNode {
    /// Render this tree node in the UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if self.children.is_empty() {
            ui.label(&self.label);
        } else {
            egui::CollapsingHeader::new(&self.label)
                .default_open(self.expanded)
                .show(ui, |ui| {
                    for child in &mut self.children {
                        child.ui(ui);
                    }
                });
        }
    }
}

/// Truncate a string with ellipsis if too long
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format bytes as human-readable size
pub fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Get message statistics
pub fn get_message_stats(message: &Message) -> MessageStats {
    let segment_count = message.segments.len();
    let mut field_count = 0;
    let mut segment_types: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for segment in &message.segments {
        *segment_types.entry(segment.id.clone()).or_insert(0) += 1;
        field_count += segment.fields.len();
    }

    let message_type = message.get_message_type()
        .map(|(t, e)| format!("{}^{}", t, e))
        .unwrap_or_else(|| "Unknown".to_string());

    let version = message.get_version()
        .map(|v| v.as_str().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    MessageStats {
        message_type,
        version,
        segment_count,
        field_count,
        segment_types,
    }
}

/// Statistics about a parsed message
pub struct MessageStats {
    pub message_type: String,
    pub version: String,
    pub segment_count: usize,
    pub field_count: usize,
    pub segment_types: std::collections::HashMap<String, usize>,
}

use egui;
