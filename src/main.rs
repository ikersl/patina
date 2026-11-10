mod ast;
mod codegen;
mod errors;
mod lexer;
mod parser;

use codegen::Codegen;
use errors::{format_error, parse_legacy_error};
use lexer::Lexer;
use parser::Parser;
use std::process::Command;

const VERSION: &str = "0.2.0";

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
    let rust_code = Codegen::new().generate(&program);

    if emit_rust {
        println!("{rust_code}");
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

    let tmp_dir = std::env::temp_dir().join("patina_build");
    std::fs::create_dir_all(&tmp_dir).ok();

    let rust_file = tmp_dir.join(format!("{out_name}.rs"));
    if let Err(e) = std::fs::write(&rust_file, &rust_code) {
        eprintln!("\x1b[1;31merror\x1b[0m: could not write temp file: {e}");
        std::process::exit(1);
    }

    let output_path = std::env::current_dir()
        .unwrap_or_default()
        .join(&out_name);

    let mut cmd = Command::new("rustc");
    cmd.arg(&rust_file)
        .arg("-o")
        .arg(&output_path)
        .arg("--edition")
        .arg("2021");

    if release {
        cmd.arg("-O");
    }

    let status = cmd.status();

    match status {
        Ok(s) if s.success() => {
            if release {
                eprintln!(
                    "\x1b[1;32mcompiled\x1b[0m (release): {}",
                    output_path.display()
                );
            } else {
                eprintln!("\x1b[1;32mcompiled\x1b[0m: {}", output_path.display());
            }
            if run_after {
                let run_status = Command::new(&output_path).status();
                match run_status {
                    Ok(s) => std::process::exit(s.code().unwrap_or(0)),
                    Err(e) => {
                        eprintln!("\x1b[1;31merror\x1b[0m running program: {e}");
                        std::process::exit(1);
                    }
                }
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
