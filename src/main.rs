use vellum::{generate_html, validate_inputs, GeneratorConfig, assets::Assets};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

const CONFIG_DIR: &str = "vellum";
const CONFIG_FILE: &str = "config.toml";
const LOCAL_CONFIG_DIR: &str = "config";
const MAX_PARENT_SEARCH_DEPTH: usize = 4;
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Args {
    config_dir: Option<PathBuf>,
    show_help: bool,
    show_version: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        config_dir: None,
        show_help: false,
        show_version: false,
    };

    let mut argv: Vec<String> = env::args().skip(1).collect();

    while !argv.is_empty() {
        let arg = argv.remove(0);
        match arg.as_str() {
            "-h" | "--help" => args.show_help = true,
            "-V" | "--version" => args.show_version = true,
            "-c" | "--config" => {
                if argv.is_empty() {
                    return Err(format!("{} requires a path argument", arg));
                }
                args.config_dir = Some(PathBuf::from(argv.remove(0)));
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown option: {}", arg));
            }
            _ => {
                return Err(format!("Unexpected argument: {}", arg));
            }
        }
    }

    Ok(args)
}

fn print_help() {
    println!("Vellum - Static HTML Generator");
    println!();
    println!("USAGE:");
    println!("    vellum [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -c, --config <PATH> Use config from specified directory or file");
    println!("    -h, --help          Print help information");
    println!("    -V, --version       Print version information");
    println!();
    println!("CONFIG SEARCH ORDER (when -c not specified):");
    println!("    1. ~/.config/vellum/config.toml");
    println!("    2. ./config/config.toml");
    println!("    3. ../config/config.toml (up to {} levels)", MAX_PARENT_SEARCH_DEPTH);
}

fn get_global_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join(CONFIG_DIR))
}

fn find_config_dir() -> Option<PathBuf> {
    if let Some(global_dir) = get_global_config_dir() {
        if global_dir.join(CONFIG_FILE).exists() {
            return Some(global_dir);
        }
    }

    let current_dir = std::env::current_dir().ok()?;
    let mut search_dir = current_dir.as_path();

    for _ in 0..=MAX_PARENT_SEARCH_DEPTH {
        let config_dir = search_dir.join(LOCAL_CONFIG_DIR);
        if config_dir.join(CONFIG_FILE).exists() {
            return Some(config_dir);
        }

        match search_dir.parent() {
            Some(parent) => search_dir = parent,
            None => break,
        }
    }

    None
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Try 'vellum --help' for more information.");
            return ExitCode::FAILURE;
        }
    };

    if args.show_help {
        print_help();
        return ExitCode::SUCCESS;
    }

    if args.show_version {
        println!("vellum {}", VERSION);
        return ExitCode::SUCCESS;
    }

    let start_time = Instant::now();

    println!("Vellum - Static HTML Generator");
    println!("===============================");

    // Determine config directory and path
    // Accept either a directory (will append config.toml) or a direct file path
    let (config_dir, config_path) = if let Some(path) = args.config_dir {
        if path.is_file() || path.extension().map_or(false, |ext| ext == "toml") {
            // Direct file path provided
            let dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
            (dir, path)
        } else {
            // Directory provided
            let file_path = path.join(CONFIG_FILE);
            if !file_path.exists() {
                eprintln!("Error: Config file not found: {}", file_path.display());
                return ExitCode::FAILURE;
            }
            (path, file_path)
        }
    } else {
        match find_config_dir() {
            Some(dir) => {
                let file_path = dir.join(CONFIG_FILE);
                (dir, file_path)
            }
            None => {
                eprintln!("Error: Could not find config.toml");
                eprintln!("Searched locations:");
                if let Some(global_dir) = get_global_config_dir() {
                    eprintln!("  - {}", global_dir.join(CONFIG_FILE).display());
                }
                eprintln!("  - ./config/config.toml (and {} parent directories)", MAX_PARENT_SEARCH_DEPTH);
                eprintln!("Hint: Run `just install` to set up the global config directory,");
                eprintln!("      or create a 'config' folder with config.toml in your project");
                return ExitCode::FAILURE;
            }
        }
    };
    let config = match GeneratorConfig::from_file(&config_path) {
        Ok(config) => {
            println!("Config: {}", config_path.display());
            config
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            eprintln!("Hint: Run `just install` to set up the config directory");
            return ExitCode::FAILURE;
        }
    };

    let assets = match Assets::load(&config_dir) {
        Ok(assets) => assets,
        Err(e) => {
            eprintln!("Error loading assets: {}", e);
            eprintln!("Hint: Run `just install` to set up the config directory");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = validate_inputs(&config) {
        eprintln!("Error: {}", e);
        return ExitCode::FAILURE;
    }

    let markdown_size = fs::metadata(&config.markdown_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let logo_size = fs::metadata(&config.logo_path)
        .map(|m| m.len())
        .unwrap_or(0);

    println!("Input: {} ({} bytes)", config.markdown_path, markdown_size);
    println!("Logo: {} ({} bytes)", config.logo_path, logo_size);
    println!();

    match generate_html(&config, &assets) {
        Ok((html, stats)) => {
            println!("Loaded {} lines from source", stats.source_lines);
            println!("Processing file includes...");
            if stats.expanded_lines > stats.source_lines {
                println!(
                    "Expanded to {} lines (+{} from includes)",
                    stats.expanded_lines,
                    stats.expanded_lines - stats.source_lines
                );
            }
            println!("Substituting template variables...");
            println!("Transforming achievement markers...");
            if stats.achievement_markers > 0 {
                println!("Found {} achievement marker(s)", stats.achievement_markers);
            }
            println!("Parsing markdown to HTML...");
            println!("Generated {} bytes of HTML content", stats.html_content_size);
            println!("Extracting navigation sections...");
            println!("Found {} section(s)", stats.section_count);
            println!("Rendering final document...");
            println!("Embedded assets and styles");

            if let Some(parent) = std::path::Path::new(&config.output_path).parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!("Error creating output directory: {}", e);
                        return ExitCode::FAILURE;
                    }
                }
            }

            match fs::write(&config.output_path, &html) {
                Ok(_) => {
                    let elapsed = start_time.elapsed();

                    println!();
                    println!("===============================");
                    println!("Output: {} ({} bytes)", config.output_path, html.len());
                    println!("Processed in {:.2?}", elapsed);
                    println!("Done!");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error writing output: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}
