use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use rs7_core::Version;
use rs7_parser::parse_message;
use rs7_terser::Terser;
use rs7_validator::Validator;
use serde_json::json;
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "rs7")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse an HL7 message and display its structure
    Parse {
        /// Input file path (use '-' for stdin)
        #[arg(value_name = "FILE")]
        input: String,

        /// Output format: text, json, pretty
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show segment details
        #[arg(short, long)]
        detailed: bool,
    },

    /// Validate an HL7 message against standards
    Validate {
        /// Input file path (use '-' for stdin)
        #[arg(value_name = "FILE")]
        input: String,

        /// HL7 version to validate against (e.g., 2.5, 2.7)
        #[arg(long, value_name = "VERSION")]
        hl7_version: Option<String>,

        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Extract field values using Terser paths
    Extract {
        /// Input file path (use '-' for stdin)
        #[arg(value_name = "FILE")]
        input: String,

        /// Terser paths to extract (e.g., PID-5, OBX(0)-5)
        #[arg(required = true)]
        paths: Vec<String>,

        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Convert message to different formats
    Convert {
        /// Input file path (use '-' for stdin)
        #[arg(value_name = "FILE")]
        input: String,

        /// Output format: json, fhir
        #[arg(short, long, required = true)]
        to: String,

        /// Pretty-print output
        #[arg(short, long)]
        pretty: bool,
    },

    /// Display message information
    Info {
        /// Input file path (use '-' for stdin)
        #[arg(value_name = "FILE")]
        input: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse {
            input,
            format,
            detailed,
        } => parse_command(&input, &format, detailed)?,
        Commands::Validate {
            input,
            hl7_version,
            format,
        } => validate_command(&input, hl7_version.as_deref(), &format)?,
        Commands::Extract {
            input,
            paths,
            format,
        } => extract_command(&input, &paths, &format)?,
        Commands::Convert { input, to, pretty } => convert_command(&input, &to, pretty)?,
        Commands::Info { input } => info_command(&input)?,
    }

    Ok(())
}

fn read_input(input: &str) -> Result<String> {
    if input == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        Ok(buffer)
    } else {
        fs::read_to_string(input).context(format!("Failed to read file: {}", input))
    }
}

fn parse_command(input: &str, format: &str, detailed: bool) -> Result<()> {
    let content = read_input(input)?;
    let message = parse_message(&content).context("Failed to parse HL7 message")?;

    match format {
        "json" => {
            let json = json!({
                "version": message.get_version().map(|v| format!("{:?}", v)),
                "message_type": message.get_message_type().map(|(m, e)| format!("{}^{}", m, e)),
                "control_id": message.get_control_id(),
                "segment_count": message.segments.len(),
                "segments": message.segments.iter().map(|s| {
                    json!({
                        "id": s.id,
                        "field_count": s.fields.len(),
                    })
                }).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        "pretty" => {
            println!("{}", "Message Structure:".bold().green());
            println!("  Version: {}", message.get_version().map(|v| format!("{:?}", v)).unwrap_or("Unknown".to_string()).cyan());
            println!("  Type: {}", message.get_message_type().map(|(m, e)| format!("{}^{}", m, e)).unwrap_or("Unknown".to_string()).cyan());
            println!("  Control ID: {}", message.get_control_id().unwrap_or("N/A").cyan());
            println!("  Segments: {}", message.segments.len().to_string().yellow());

            if detailed {
                println!();
                for (idx, segment) in message.segments.iter().enumerate() {
                    println!("  {}. {} ({} fields)", idx + 1, segment.id.bold(), segment.fields.len());
                    for (field_idx, field) in segment.fields.iter().enumerate() {
                        let encoded = field.encode(&message.delimiters);
                        if !encoded.is_empty() {
                            println!("      [{}]: {}", field_idx, encoded.bright_black());
                        }
                    }
                }
            }
        }
        _ => {
            // Default text format
            println!("HL7 Message Parsed Successfully");
            println!("Version: {}", message.get_version().map(|v| format!("{:?}", v)).unwrap_or("Unknown".to_string()));
            println!("Message Type: {}", message.get_message_type().map(|(m, e)| format!("{}^{}", m, e)).unwrap_or("Unknown".to_string()));
            println!("Control ID: {}", message.get_control_id().unwrap_or("N/A"));
            println!("Segments: {}", message.segments.len());

            if detailed {
                println!("\nSegment Details:");
                for segment in &message.segments {
                    println!("  {} ({} fields)", segment.id, segment.fields.len());
                }
            }
        }
    }

    Ok(())
}

fn validate_command(input: &str, version: Option<&str>, format: &str) -> Result<()> {
    let content = read_input(input)?;
    let message = parse_message(&content).context("Failed to parse HL7 message")?;

    // Get version from parameter or message
    let hl7_version = if let Some(v) = version {
        match v {
            "2.3" => Version::V2_3,
            "2.3.1" => Version::V2_3_1,
            "2.4" => Version::V2_4,
            "2.5" => Version::V2_5,
            "2.5.1" => Version::V2_5_1,
            "2.6" => Version::V2_6,
            "2.7" => Version::V2_7,
            "2.7.1" => Version::V2_7_1,
            _ => message.get_version().unwrap_or(Version::V2_5),
        }
    } else {
        message.get_version().unwrap_or(Version::V2_5)
    };

    let validator = Validator::new(hl7_version);
    let result = validator.validate(&message);

    match format {
        "json" => {
            let errors_json: Vec<_> = result.errors.iter().map(|e| {
                json!({
                    "location": e.location,
                    "message": e.message,
                    "type": format!("{:?}", e.error_type),
                })
            }).collect();

            let warnings_json: Vec<_> = result.warnings.iter().map(|w| {
                json!({
                    "location": w.location,
                    "message": w.message,
                })
            }).collect();

            let json = json!({
                "valid": result.is_valid(),
                "error_count": result.errors.len(),
                "warning_count": result.warnings.len(),
                "errors": errors_json,
                "warnings": warnings_json,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            if result.is_valid() {
                println!("{}", "✓ Message is valid".green().bold());
            } else {
                println!("{}", "✗ Message validation failed".red().bold());
            }

            if !result.errors.is_empty() {
                println!("\n{}", "Errors:".red().bold());
                for error in &result.errors {
                    println!("  • {} - {}", error.location.yellow(), error.message.red());
                }
            }

            if !result.warnings.is_empty() {
                println!("\n{}", "Warnings:".yellow().bold());
                for warning in &result.warnings {
                    println!("  • {} - {}", warning.location.yellow(), warning.message);
                }
            }

            if result.is_valid() && result.warnings.is_empty() {
                println!("\nNo issues found.");
            }
        }
    }

    Ok(())
}

fn extract_command(input: &str, paths: &[String], format: &str) -> Result<()> {
    let content = read_input(input)?;
    let message = parse_message(&content).context("Failed to parse HL7 message")?;
    let terser = Terser::new(&message);

    match format {
        "json" => {
            let mut results = serde_json::Map::new();
            for path in paths {
                let value = terser.get(path).ok().flatten().unwrap_or("").to_string();
                results.insert(path.clone(), json!(value));
            }
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        _ => {
            for path in paths {
                let value = terser.get(path).ok().flatten().unwrap_or("");
                if value.is_empty() {
                    println!("{}: {}", path.cyan(), "N/A".bright_black());
                } else {
                    println!("{}: {}", path.cyan(), value.green());
                }
            }
        }
    }

    Ok(())
}

fn convert_command(input: &str, to: &str, pretty: bool) -> Result<()> {
    let content = read_input(input)?;
    let message = parse_message(&content).context("Failed to parse HL7 message")?;

    match to {
        "json" => {
            let json = json!({
                "version": message.get_version().map(|v| format!("{:?}", v)),
                "message_type": message.get_message_type().map(|(m, e)| format!("{}^{}", m, e)),
                "control_id": message.get_control_id(),
                "sending_application": message.get_sending_application(),
                "sending_facility": message.get_sending_facility(),
                "receiving_application": message.get_receiving_application(),
                "receiving_facility": message.get_receiving_facility(),
                "segments": message.segments.iter().map(|seg| {
                    json!({
                        "id": seg.id,
                        "fields": seg.fields.iter().map(|f| f.encode(&message.delimiters)).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
            });

            if pretty {
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("{}", serde_json::to_string(&json)?);
            }
        }
        "fhir" => {
            #[cfg(feature = "fhir")]
            {
                use rs7_fhir::converter::MessageConverter;
                let converter = MessageConverter::new();
                match converter.convert(&message) {
                    Ok(bundle) => {
                        if pretty {
                            println!("{}", serde_json::to_string_pretty(&bundle)?);
                        } else {
                            println!("{}", serde_json::to_string(&bundle)?);
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", format!("FHIR conversion failed: {}", e).red());
                        std::process::exit(1);
                    }
                }
            }
            #[cfg(not(feature = "fhir"))]
            {
                eprintln!("{}", "FHIR conversion requires the 'fhir' feature. Rebuild with: cargo build --features fhir".red());
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("{}", format!("Unsupported output format: {}", to).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

fn info_command(input: &str) -> Result<()> {
    let content = read_input(input)?;
    let message = parse_message(&content).context("Failed to parse HL7 message")?;

    println!("{}", "HL7 Message Information".bold().green());
    println!("{}", "=".repeat(50).bright_black());

    println!("\n{}", "Header Information:".bold());
    println!("  Version:              {}", message.get_version().map(|v| format!("{:?}", v)).unwrap_or("Unknown".to_string()).cyan());
    println!("  Message Type:         {}", message.get_message_type().map(|(m, e)| format!("{}^{}", m, e)).unwrap_or("Unknown".to_string()).cyan());
    println!("  Control ID:           {}", message.get_control_id().unwrap_or("N/A").cyan());
    println!("  Sending Application:  {}", message.get_sending_application().unwrap_or("N/A").cyan());
    println!("  Sending Facility:     {}", message.get_sending_facility().unwrap_or("N/A").cyan());
    println!("  Receiving Application: {}", message.get_receiving_application().unwrap_or("N/A").cyan());
    println!("  Receiving Facility:   {}", message.get_receiving_facility().unwrap_or("N/A").cyan());

    println!("\n{}", "Message Structure:".bold());
    println!("  Total Segments:       {}", message.segments.len().to_string().yellow());

    // Count segment types
    let mut segment_counts = std::collections::HashMap::new();
    for segment in &message.segments {
        *segment_counts.entry(&segment.id).or_insert(0) += 1;
    }

    println!("  Segment Types:        {}", segment_counts.len().to_string().yellow());
    println!("\n  Segment Breakdown:");
    let mut sorted_segments: Vec<_> = segment_counts.iter().collect();
    sorted_segments.sort_by(|a, b| a.0.cmp(b.0));
    for (id, count) in sorted_segments {
        println!("    {} x {}", count.to_string().bright_white(), id.bright_cyan());
    }

    // Calculate message size
    let encoded = message.encode();
    println!("\n{}", "Size Information:".bold());
    println!("  Encoded Size:         {} bytes", encoded.len().to_string().yellow());
    println!("  Average Segment Size: {} bytes", (encoded.len() / message.segments.len()).to_string().yellow());

    Ok(())
}
