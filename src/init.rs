use crate::config::Config;
use ontodev_valve::{valve, ValveCommand};
use std::error;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{prelude::*, BufReader};
use std::path::Path;

fn add_to_gitignore(input: &str) -> Result<String, String> {
    if Path::new(".gitignore").exists() {
        let file = match File::open(".gitignore") {
            Ok(f) => f,
            Err(e) => return Err(e.to_string()),
        };
        let reader = BufReader::new(file);

        let mut found = false; //if true, then input is already in .gitignore
        let mut last_line_empty = false;
        let mut modified = false; //if true, then nanobot has already modified .gitignore

        for line in reader.lines() {
            let string = match line {
                Ok(l) => l,
                Err(e) => return Err(e.to_string()),
            };
            if string.contains(input) {
                found = true;
            }
            if string.trim().eq("") {
                last_line_empty = true;
            } else {
                last_line_empty = false;
            }
            if string.contains("Generated by nanobot") {
                modified = true;
            }
        }

        if !found {
            if modified {
                //insert input in .gitignore
                //where nanobot has already modified the file previously
                let mut file = match OpenOptions::new().write(true).open(".gitignore") {
                    Ok(f) => f,
                    Err(e) => return Err(e.to_string()),
                };

                let file_string = match fs::read_to_string(".gitignore") {
                    Ok(f) => f,
                    Err(e) => return Err(e.to_string()),
                };
                let mut file_lines: Vec<&str> = file_string.split("\n").collect();
                for (pos, line) in file_lines.clone().iter().enumerate() {
                    if line.contains("Generated by nanobot") {
                        file_lines.insert(pos + 1, input);
                    }
                }
                let file_res = file_lines.join("\n");

                if let Err(e) = write!(file, "{}", file_res) {
                    eprintln!("Couldn't write to file: {}", e);
                }
                Ok(String::from("NotFound-Modified"))
            } else {
                let mut file = match OpenOptions::new().write(true).append(true).open(".gitignore")
                {
                    Ok(f) => f,
                    Err(e) => return Err(e.to_string()),
                };

                if !last_line_empty {
                    if let Err(e) = writeln!(file, "") {
                        eprintln!("Couldn't write to file: {}", e);
                    }
                }

                if let Err(e) = writeln!(file, "# Generated by nanobot") {
                    eprintln!("Couldn't write to file: {}", e);
                }

                if let Err(e) = writeln!(file, "{}", input) {
                    eprintln!("Couldn't write to file: {}", e);
                }
                Ok(String::from("NotFound-NotModified"))
            }
        } else {
            //input is already in .gitignore
            Ok(String::from("Found"))
        }
    } else {
        Ok(String::from("No .gitignore"))
    }
}

fn create_table_tsv() -> Result<(), Box<dyn error::Error>> {
    let data = include_str!("resources/table.tsv");
    fs::write("src/schema/table.tsv", data).expect("Unable to write file");
    Ok(())
}

fn create_column_tsv() -> Result<(), Box<dyn error::Error>> {
    let data = include_str!("resources/column.tsv");
    fs::write("src/schema/column.tsv", data).expect("Unable to write file");
    Ok(())
}

fn create_datatype_tsv() -> Result<(), Box<dyn error::Error>> {
    let data = include_str!("resources/datatype.tsv");
    fs::write("src/schema/datatype.tsv", data).expect("Unable to write file");
    Ok(())
}

pub async fn init(config: &Config) -> Result<String, String> {
    let database = config.connection.to_owned();
    // Fail if nanobot.toml or .nanobot.db exist
    let path = Path::new("nanobot.toml");
    if path.exists() {
        return Err(format!("Cannot init: '{}' file already exists.", path.display()));
    }
    let path = Path::new(&database);
    if path.exists() {
        return Err(format!("Cannot init: '{}' database already exists", path.display()));
    }

    // Create default config nanobot.toml
    let path = Path::new("nanobot.toml");
    let toml = include_str!("resources/default_config.toml");
    fs::write(path, toml).expect("Unable to write file");
    tracing::info!("Created config file '{}'", path.display());

    // Create the basic VALVE schema tables, if they don't exist
    let path = Path::new("src/schema");
    if !path.exists() {
        match fs::create_dir_all(path) {
            Err(_x) => return Err(format!("Could not create '{}'", path.display())),
            Ok(_x) => {}
        };
        tracing::info!("Created '{}' directory", path.display());
    }

    let path = Path::new("src/schema/table.tsv");
    if !path.exists() {
        match create_table_tsv() {
            Err(_x) => return Err(format!("Could not create '{}'", path.display())),
            Ok(_x) => {}
        };
        tracing::info!("Created '{}' file", path.display());
    }

    let path = Path::new("src/schema/column.tsv");
    if !path.exists() {
        match create_column_tsv() {
            Err(_x) => return Err(format!("Could not create '{}'", path.display())),
            Ok(_x) => {}
        };
        tracing::info!("Created '{}' file", path.display());
    }

    let path = Path::new("src/schema/datatype.tsv");
    if !path.exists() {
        match create_datatype_tsv() {
            Err(_x) => return Err(format!("Could not create '{}'", path.display())),
            Ok(_x) => {}
        };
        tracing::info!("Created '{}' file", path.display());
    }

    //create database
    match File::create(&database) {
        Err(_x) => return Err(String::from("Couldn't create database")),
        Ok(_x) => {}
    }

    //add database to .gitignore
    match add_to_gitignore(format!("{}*", &database).as_str()) {
        Err(x) => return Err(x),
        Ok(_x) => {}
    }

    // load tables into database
    let path = "src/schema/table.tsv";
    match valve(path, &database, &ValveCommand::Load, false, "table").await {
        Err(_x) => return Err(format!("Could not load from '{}'", path)),
        Ok(_x) => {}
    }

    tracing::info!("Loaded '{}' using '{}'", database, path);

    Ok(String::from("Initialized a Nanobot project"))
}
