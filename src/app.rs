﻿use crate::args::build_cli;
use crate::processors::json::JSONProcessor;
use crate::processors::toml::TOMLProcessor;
use crate::processors::yaml::YAMLProcessor;
use crate::processors::Processor;
use crate::terminal::Messages;
use crate::values::VizValue;
use anyhow::{anyhow, bail, Result};
use clap::ArgMatches;
use std::env::var;
use std::fs;
use std::io::{stdin, Read};
use std::path::Path;
use std::process::exit;

pub fn run() -> Result<()> {
    let args = build_cli().get_matches();

    configure_colors(&args);

    let (contents, extension) = get_content_and_extension(&args)?;
    let indent = get_indent(&args)?;
    let data = get_parsed_data(&contents, &extension)?;

    print_parsed_data(data, indent);

    Ok(())
}

fn configure_colors(args: &ArgMatches) {
    let no_color = var("NO_COLOR");

    if no_color.is_ok() {
        colored::control::set_override(false);
    } else {
        colored::control::set_override(!args.get_flag("no-color"));
    }
}

fn get_content_and_extension(args: &ArgMatches) -> Result<(String, String)> {
    let file_path = args
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or("");

    if file_path.is_empty() {
        get_from_stdin(args)
    } else {
        get_file_content(file_path)
    }
}

fn get_from_stdin(args: &ArgMatches) -> Result<(String, String)> {
    let mut contents = String::new();
    stdin()
        .read_to_string(&mut contents)
        .map_err(|e| anyhow!("failed to read from stdin: {}", e.to_string()))?;

    if let Some(lang) = args.get_one::<String>("language") {
        Ok((contents, lang.clone()))
    } else {
        bail!("language is not specified for stdin")
    }
}

fn get_file_content(file_path: &str) -> Result<(String, String)> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(anyhow!("file not found"));
    }

    let contents = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("failed to read file: {}", e.to_string()))?;

    let ext = path
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        .to_lowercase();

    Ok((contents, ext))
}

fn get_indent(args: &ArgMatches) -> Result<usize> {
    let indent = *args.get_one::<usize>("indent").unwrap_or(&2);

    if indent > 10 {
        return Err(anyhow!(
            "indentation level must be less than or equal to 10."
        ));
    }

    Ok(indent)
}

fn get_parsed_data(contents: &str, extension: &str) -> Result<VizValue> {
    let parsed_data = match extension {
        "json" => JSONProcessor::process_data(&contents),
        "toml" => TOMLProcessor::process_data(&contents),
        "yaml" | "yml" => YAMLProcessor::process_data(&contents),
        _ => {
            return Err(anyhow!("unsupported file format."));
        }
    }?;

    Ok(parsed_data)
}

fn print_parsed_data(data: VizValue, indent: usize) {
    if let VizValue::Object(map) = data {
        println!("{{");
        let entries: Vec<_> = map.into_iter().collect();
        let total = entries.len();
        for (i, (key, val)) in entries.into_iter().enumerate() {
            let last = i + 1 == total;
            crate::prints::print_object_data(&key, val, indent, indent, last, true);
        }
        println!("}}");
    } else {
        Messages::error("internal error: parsed data is not a valid object.");
        exit(1);
    }
}
