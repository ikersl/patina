mod ast;
mod codegen;
mod errors;
mod lexer;
mod parser;

use codegen::{Codegen, GenerateResult};
use errors::{format_error, parse_legacy_error};
use lexer::Lexer;
use parser::Parser;
use std::process::Command;

const VERSION: &str = "0.3.0";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("\x1b[1;35mPatina\x1b[0m v{VERSION} — a language that feels like Python, runs like Rust\n");
        eprintln!("Usage: patina <file.pa> [options]\n");
        eprintln!("Options:");
        eprintln!("  --emit-rust    Print generated Rust code instead of compiling");
        eprintln!("  --run          Compile and run immediately");
        eprintln!("  -o <name>      Output binary name (default: based on input filename)");
        eprintln!("  --release      Compile with optimizations");
        eprintln!("  --version      Print version");
        std::process::exit(1);
    }

    let input_file = &args[1];

    if input_file == "--version" {
        println!("Patina v{VERSION}");
        return;
    }

    let mut emit_rust = false;
    let mut run_after = false;
    let mut release = false;
    let mut output_name: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--emit-rust" => emit_rust = true,
            "--run" => run_after = true,
            "--release" => release = true,
            "--version" => {
                println!("Patina v{VERSION}");
                return;
            }
            "-o" => {
                i += 1;
                if i < args.len() {
                    output_name = Some(args[i].clone());
                } else {
                    eprintln!("\x1b[1;31merror\x1b[0m: -o requires a name");
                    std::process::exit(1);
                }
            }
            other => {
                eprintln!("\x1b[1;31merror\x1b[0m: unknown option '{other}'");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let source = match std::fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\x1b[1;31merror\x1b[0m: could not read '{input_file}': {e}");
            std::process::exit(1);
        }
    };

    // Lex
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            let err = parse_legacy_error(&e);
            eprint!("{}", format_error(&err, &source, input_file));
            std::process::exit(1);
        }
    };

    // Parse
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            let err = parse_legacy_error(&e);
            eprint!("{}", format_error(&err, &source, input_file));
            std::process::exit(1);
        }
    };

    // Generate Rust
    let result = Codegen::new().generate(&program);

    if emit_rust {
        println!("{}", result.code);
        return;
    }

    // Compile
    let out_name = output_name.unwrap_or_else(|| {
        std::path::Path::new(input_file)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });

    let output_path = std::env::current_dir()
        .unwrap_or_default()
        .join(&out_name);

    if result.crates.is_empty() {
        compile_rustc(&result.code, &out_name, &output_path, release, run_after);
    } else {
        compile_cargo(&result.code, &out_name, &output_path, release, run_after, &result.crates);
    }
}

fn compile_rustc(
    rust_code: &str,
    out_name: &str,
    output_path: &std::path::Path,
    release: bool,
    run_after: bool,
) {
    let tmp_dir = std::env::temp_dir().join("patina_build");
    std::fs::create_dir_all(&tmp_dir).ok();

    let rust_file = tmp_dir.join(format!("{out_name}.rs"));
    if let Err(e) = std::fs::write(&rust_file, rust_code) {
        eprintln!("\x1b[1;31merror\x1b[0m: could not write temp file: {e}");
        std::process::exit(1);
    }

    let mut cmd = Command::new("rustc");
    cmd.arg(&rust_file)
        .arg("-o")
        .arg(output_path)
        .arg("--edition")
        .arg("2021");
    if release {
        cmd.arg("-O");
    }

    match cmd.status() {
        Ok(s) if s.success() => {
            let mode = if release { " (release)" } else { "" };
            eprintln!("\x1b[1;32mcompiled\x1b[0m{mode}: {}", output_path.display());
            if run_after {
                run_binary(output_path);
            }
        }
        Ok(_) => {
            eprintln!("\x1b[1;31merror\x1b[0m: compilation failed (see rustc output above)");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("\x1b[1;31merror\x1b[0m: could not run rustc: {e}");
            eprintln!("  \x1b[1;32mhelp\x1b[0m: make sure rustc is installed (rustup.rs)");
            std::process::exit(1);
        }
    }
}

fn compile_cargo(
    rust_code: &str,
    out_name: &str,
    output_path: &std::path::Path,
    release: bool,
    run_after: bool,
    crates: &[(String, String)],
) {
    let proj_dir = std::env::temp_dir().join("patina_build").join(format!("{out_name}_cargo"));
    let src_dir = proj_dir.join("src");
    std::fs::create_dir_all(&src_dir).ok();

    let mut cargo_toml = format!(
        "[package]\nname = \"{out_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n"
    );
    for (name, version) in crates {
        cargo_toml.push_str(&format!("{name} = \"{version}\"\n"));
    }

    if let Err(e) = std::fs::write(proj_dir.join("Cargo.toml"), &cargo_toml) {
        eprintln!("\x1b[1;31merror\x1b[0m: could not write Cargo.toml: {e}");
        std::process::exit(1);
    }
    if let Err(e) = std::fs::write(src_dir.join("main.rs"), rust_code) {
        eprintln!("\x1b[1;31merror\x1b[0m: could not write src/main.rs: {e}");
        std::process::exit(1);
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("build").current_dir(&proj_dir);
    if release {
        cmd.arg("--release");
    }

    match cmd.status() {
        Ok(s) if s.success() => {
            let profile = if release { "release" } else { "debug" };
            let built = proj_dir.join("target").join(profile).join(out_name);
            if let Err(e) = std::fs::copy(&built, output_path) {
                eprintln!("\x1b[1;31merror\x1b[0m: could not copy binary: {e}");
                std::process::exit(1);
            }
            let mode = if release { " (release)" } else { "" };
            eprintln!("\x1b[1;32mcompiled\x1b[0m{mode}: {}", output_path.display());
            if run_after {
                run_binary(output_path);
            }
        }
        Ok(_) => {
            eprintln!("\x1b[1;31merror\x1b[0m: compilation failed (see cargo output above)");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("\x1b[1;31merror\x1b[0m: could not run cargo: {e}");
            eprintln!("  \x1b[1;32mhelp\x1b[0m: make sure cargo is installed (rustup.rs)");
            std::process::exit(1);
        }
    }
}

fn run_binary(path: &std::path::Path) {
    match Command::new(path).status() {
        Ok(s) => std::process::exit(s.code().unwrap_or(0)),
        Err(e) => {
            eprintln!("\x1b[1;31merror\x1b[0m running program: {e}");
            std::process::exit(1);
        }
    }
}
