use std::fs;
use std::io::Read;

use chibicc_for_rust::codegen::abi::{win64::*};
use chibicc_for_rust::codegen::*;
use chibicc_for_rust::frame_layout::*;
use chibicc_for_rust::parser::*;
use chibicc_for_rust::resolver::*;
use chibicc_for_rust::span::*;
use chibicc_for_rust::{span::source_map::SourceFile, tokenizer::*};
use clap::Parser as ClapParser;

#[derive(clap::Parser)]
struct Cli {
    #[arg(value_name = "INPUT")]
    input: std::path::PathBuf,
}

fn main() {
    let arg = Cli::parse();
    let mut f = match fs::File::open(&arg.input) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("error: couldn't read `{}`: {}", arg.input.display(), err);
            std::process::exit(1);
        }
    };
    let mut code = String::new();
    match f.read_to_string(&mut code) {
        Ok(_) => (),
        Err(err) => {
            eprintln!(
                "error: couldn't read `{}` after open: {}",
                arg.input.display(),
                err
            );
            std::process::exit(1);
        }
    }
    let source_file = SourceFile::new(source_map::FileName::Real(arg.input), code);
    compile(source_file);
}

fn compile(file: SourceFile) {
    let tokens = tokenize(file.src.as_bytes());

    let mut parser = Parser {
        tokens,
        index: 0,
        errors: vec![],
        node_cnt: 0,
    };

    let ast = parser.parse_crate();
    if parser.errors.is_empty() {
        let mut resolver = Resolver::new();
        resolver.resolve(&ast);
        let frame_builder = FrameBuilder::new();
        let layouts = frame_builder.build(&resolver.resolved);
        let _ = gen_asm::<Win64Abi>(ast, resolver.resolved, layouts);
    }
    for e in parser.errors {
        e.error_print(&file);
    }
}
