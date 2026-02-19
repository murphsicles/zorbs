use clap::{Parser, Subcommand};
use flate2::write::GzEncoder;
use flate2::Compression;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use tar::Builder;
use toml;

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
        package: Option<String>,
    },
    Lock,
}

#[derive(Serialize, Deserialize, Debug)]
struct ZorbToml {
    package: Package,
    #[serde(default)]
    dependencies: Option<toml::Table>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    license: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Lockfile {
    package: Vec<LockedPackage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LockedPackage {
    name: String,
    version: String,
    download_url: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => new_project(&name),
        Commands::Add { package, version } => add_dependency(&package, version),
        Commands::Publish => publish().await,
        Commands::Install { package } => install(package).await,
        Commands::Lock => generate_lock().await,
    }
}

fn new_project(name: &str) {
    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("Directory {} already exists", name);
        std::process::exit(1);
    }

    fs::create_dir_all(dir.join("src")).expect("Failed to create directories");

    let zorb_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
description = "A new Zeta package"
license = "MIT"

[dependencies]
"#, name);

    fs::write(dir.join("zorb.toml"), zorb_toml).expect("Failed to write zorb.toml");
    fs::write(dir.join("src/main.zeta"), r#"// Welcome to Zeta!
fn main() {
    print("Hello, Zeta!\n")
}
"#).expect("Failed to write main.zeta");

    println!("Created new zorb project '{}'", name);
    println!("Next: cd {} && zorb publish", name);
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
    println!("Run `zorb lock` to update zorb.lock");
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
        builder.append_path_with_name(current_dir.join("zorb.toml"), "zorb.toml").unwrap();
        let src_path = current_dir.join("src");
        if src_path.exists() {
            builder.append_dir_all("src", src_path).unwrap();
        }
        builder.finish().unwrap();
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_buf).unwrap();
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

async fn generate_lock() {
    let content = match fs::read_to_string("zorb.toml") {
        Ok(c) => c,
        Err(_) => {
            eprintln!("No zorb.toml found. Run `zorb new` first.");
            return;
        }
    };

    let zorb: ZorbToml = match toml::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid zorb.toml");
            return;
        }
    };

    let mut packages = vec![];

    if let Some(deps) = zorb.dependencies {
        let client = reqwest::Client::new();
        for (name, _) in deps {
            let url = format!("http://localhost:3000/api/resolve?name={}", name);
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let data: serde_json::Value = match resp.json().await {
                        Ok(d) => d,
                        Err(_) => continue,
                    };
                    if let (Some(n), Some(v), Some(u)) = (
                        data.get("name").and_then(|v| v.as_str()),
                        data.get("version").and_then(|v| v.as_str()),
                        data.get("download_url").and_then(|v| v.as_str()),
                    ) {
                        packages.push(LockedPackage {
                            name: n.to_string(),
                            version: v.to_string(),
                            download_url: u.to_string(),
                        });
                    }
                }
                _ => {
                    eprintln!("Could not resolve {}", name);
                }
            }
        }
    }

    let lockfile = Lockfile { package: packages };
    let lock_content = toml::to_string_pretty(&lockfile).unwrap();
    fs::write("zorb.lock", lock_content).unwrap();

    println!("Generated zorb.lock with {} resolved packages", lockfile.package.len());
}

async fn install(package: Option<String>) {
    if let Some(pkg) = package {
        let url = format!("http://localhost:3000/{}/0.1.0/download", pkg.replace('@', "").replace('/', "-"));
        download_single(&url, &pkg).await;
        return;
    }

    if Path::new("zorb.lock").exists() {
        let content = fs::read_to_string("zorb.lock").unwrap();
        let lock: Lockfile = toml::from_str(&content).unwrap();
        for p in lock.package {
            download_single(&p.download_url, &p.name).await;
        }
    } else {
        println!("No zorb.lock found. Generating now...");
        generate_lock().await;
        install(None).await;
    }
}

async fn download_single(url: &str, name: &str) {
    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let bytes = resp.bytes().await.unwrap();
            let filename = format!("{}.zorb", name.replace('/', "-"));
            fs::write(&filename, bytes).unwrap();
            println!("Installed {} -> {}", name, filename);
        }
        _ => {
            eprintln!("Failed to download {}", name);
        }
    }
}
