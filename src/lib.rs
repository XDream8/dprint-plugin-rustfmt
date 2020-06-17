use rustfmt_nightly::{Input, Session, Config, NewlineStyle, EmitMode, Edition};

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use dprint_core::generate_plugin_code;
use dprint_core::configuration::{GlobalConfiguration, ResolveConfigurationResult, NewLineKind, ConfigurationDiagnostic};

#[derive(Clone, Serialize, Deserialize)]
struct Configuration {
    // Unfortunately no resolved configuration at the moment because serializing
    // rustfmt's PartialConfig configuration kept causing a panic
    #[serde(flatten)]
    config: HashMap<String, String>,
    #[serde(skip_serializing, skip_deserializing)]
    rustfmt_config: Config,
}

fn resolve_config(
    config: HashMap<String, String>,
    global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<Configuration> {
    let mut rustfmt_config = Config::default();
    let mut diagnostics = Vec::new();

    rustfmt_config.set().edition(Edition::Edition2018);

    // set dprint global configuration
    if let Some(line_width) = global_config.line_width {
        rustfmt_config.set().max_width(line_width as usize);
    }
    if let Some(use_tabs) = global_config.use_tabs {
        rustfmt_config.set().hard_tabs(use_tabs);
    }
    if let Some(indent_width) = global_config.indent_width {
        rustfmt_config.set().tab_spaces(indent_width as usize);
    }
    if let Some(new_line_kind) = global_config.new_line_kind {
        rustfmt_config.set().newline_style(match new_line_kind {
            NewLineKind::Auto => NewlineStyle::Auto,
            NewLineKind::LineFeed => NewlineStyle::Unix,
            NewLineKind::CarriageReturnLineFeed => NewlineStyle::Windows,
            NewLineKind::System => NewlineStyle::Native,
        });
    }

    for (key, value) in config.iter() {
        if key == "newLineKind" {
            match value.as_str() {
                "auto" => rustfmt_config.set().newline_style(NewlineStyle::Auto),
                "lf" => rustfmt_config.set().newline_style(NewlineStyle::Unix),
                "crlf" => rustfmt_config.set().newline_style(NewlineStyle::Windows),
                "system" => rustfmt_config.set().newline_style(NewlineStyle::Native),
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: String::from(key),
                        message: format!("Invalid newline kind: {}", value),
                    });
                }
            }
            continue;
        }

        let key = match key.as_str() {
            "lineWidth" => "max_width",
            "useTabs" => "hard_tabs",
            "indentWidth" => "tab_spaces",
            _ => key,
        };
        if Config::is_valid_key_val(key, value) {
            rustfmt_config.override_value(key, value);
        } else {
            let message = format!("Invalid key or value in configuration. Key: {}, Value: {}", key, value);
            diagnostics.push(ConfigurationDiagnostic {
                property_name: String::from(key),
                message,
            });
        }
    }

    rustfmt_config.set().emit_mode(EmitMode::Stdout);

    ResolveConfigurationResult {
        diagnostics,
        config: Configuration { config, rustfmt_config },
    }
}

fn get_plugin_config_key() -> String {
    // return the JSON object key name used in the configuration file
    String::from("rustfmt")
}

fn get_plugin_file_extensions() -> Vec<String> {
    vec![String::from("rs")]
}

fn get_plugin_help_url() -> String {
    String::from("https://dprint.dev/plugins/rustfmt")
}

fn get_plugin_config_schema_url() -> String {
    // for now, return an empty string. Return a schema url once VSCode
    // supports $schema properties in descendant objects:
    // https://github.com/microsoft/vscode/issues/98443
    String::new()
}

fn get_plugin_license_text() -> String {
    std::str::from_utf8(include_bytes!("../LICENSE")).unwrap().into()
}

fn format_text(
    _: &PathBuf,
    file_text: &str,
    config: &Configuration,
) -> Result<String, String> {
    let mut out = Vec::new();
    {
        let input = Input::Text(String::from(file_text));
        let mut session = Session::new(config.rustfmt_config.clone(), Some(&mut out));
        match session.format(input) {
            Err(err) => {
                return Err(err.to_string());
            },
            _ => {
                // do nothing
            }
        }
    }

    // rustfmt adds this prefix, so just ignore it
    let prefix = "stdin:\n\n";
    Ok(String::from(std::str::from_utf8(&out[prefix.len()..]).unwrap()))
}

generate_plugin_code!();
