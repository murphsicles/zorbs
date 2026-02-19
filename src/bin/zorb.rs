// src/bin/zorb.rs
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::env;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use toml;
use reqwest::multipart;
use tar::Builder;
use flate2::write::GzEncoder;
use flate2::Compression;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New {
        name: String,
    },
    Add {
        package: String,
        #[arg(short, long)]
        version: Option<String>,
    },
    Publish,
    Install {
        package: String,
    },
}

#[derive(Serialize, Deserialize)]
struct ZorbToml {
    package: Package,
    dependencies: Option<toml::Table>,
}

#[derive(Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    description: Option<String>,
    license: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => new_project(&name),
        Commands::Add { package, version } => add_dependency(&package, version),
        Commands::Publish => publish().await,
        Commands::Install { package } => install(&package).await,
    }
}

fn new_project(name: &str) {
    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("Directory {} already exists", name);
        return;
    }

    fs::create_dir_all(dir.join("src")).unwrap();

    let zorb_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
description = "A new Zeta package"
license = "MIT"

[dependencies]
"#, name);

    fs::write(dir.join("zorb.toml"), zorb_toml).unwrap();
    fs::write(dir.join("src/main.zeta"), "// Welcome to Zeta!\nfn main() {\n    print(\"Hello, Zeta!\\n\")\n}\n").unwrap();

    println!("Created new zorb project '{}'", name);
    println!("cd {} && zorb publish", name);
}

fn add_dependency(package: &str, version: Option<String>) {
    let version = version.unwrap_or_else(|| "^0.1.0".to_string());
    let content = match fs::read_to_string("zorb.toml") {
        Ok(c) => c,
        Err(_) => {
            eprintln!("No zorb.toml found. Run `zorb new` first.");
            return;
        }
    };

    let mut zorb: ZorbToml = match toml::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid zorb.toml");
            return;
        }
    };

    if zorb.dependencies.is_none() {
        zorb.dependencies = Some(toml::Table::new());
    }

    if let Some(deps) = &mut zorb.dependencies {
        deps.insert(package.to_string(), toml::Value::String(version.clone()));
    }

    let new_content = toml::to_string_pretty(&zorb).unwrap();
    fs::write("zorb.toml", new_content).unwrap();

    println!("Added {} = \"{}\"", package, version);
}

async fn publish() {
    let current_dir = env::current_dir().unwrap();

    if !current_dir.join("zorb.toml").exists() {
        eprintln!("No zorb.toml found. Run `zorb new` first.");
        return;
    }

    let mut tar_buf = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_buf);
        let _ = builder.append_path_with_name("zorb.toml", "zorb.toml");
        let src_path = current_dir.join("src");
        if src_path.exists() {
            let _ = builder.append_dir_all("src", src_path);
        }
        let _ = builder.finish();
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(&tar_buf);
    let compressed = encoder.finish().unwrap();

    let client = reqwest::Client::new();
    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(compressed).file_name("package.zorb"));

    let url = "http://localhost:3000/api/zorbs/new";

    match client.post(url).multipart(form).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("Zorb published successfully!");
        }
        Ok(resp) => {
            eprintln!("Publish failed: {}", resp.status());
        }
        Err(e) => {
            eprintln!("Request error: {}", e);
        }
    }
}

async fn install(package: &str) {
    let url = format!("http://localhost:3000/{}/0.1.0/download", package.replace('@', "").replace('/', "-"));

    let client = reqwest::Client::new();

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let bytes = resp.bytes().await.unwrap();
            let filename = format!("{}.zorb", package.replace('/', "-"));
            fs::write(&filename, bytes).unwrap();
            println!("Installed {} -> {}", package, filename);
        }
        _ => {
            eprintln!("Failed to download {}", package);
        }
    }
}
