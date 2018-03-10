mod simple_parser;
mod man_parse;

use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use man_parse::troff_parser::TroffParser;
use man_parse::troff_preprocessor::TroffPreprocessor;
use simple_parser::preprocessor::Preprocessor;

struct ExplainArgs {
    command_name: String,
    command_args: Vec<String>,

    explain_args: Vec<String>,
}

fn main() {
    let args = argparse();

    println!("executing for program named: {}", args.command_name);

    let section = if args.explain_args.len() >= 2 {
        // TODO: for now, I'm saying all explain args is -s...
        Some(&args.explain_args[1])
    } else {
        None
    };

    let man_path = get_manpage_path(&args.command_name);
    println!("found manpath: [{}]", &man_path);

    let man_text = if is_gzipped(&man_path) {
        unzip(&man_path)
    } else {
        read_file_content(&man_path)
    };

    // if !is_troff(&man_text) {
    //     println!("Man text does not appear to be troff.");
    // }

    let classifier = man_parse::troff_tokenize::TroffClassifier {};
    let tokenized = simple_parser::tokenizer::tokenize(&man_text, &classifier);

    for tok in &tokenized {
        println!("{:?}", tok);
    }

    let mut parser = match section {
        None => TroffParser::new(),
        // TODO: for now, I'm saying every -s argument is for Synopsis...
        Some(s) => TroffParser::for_section(man_parse::troff_parser::ManSection::Synopsis),
    };

    parser.parse(tokenized.iter());

    if section.is_some() {
        println!("\x1B[1msection text: {}", parser.section_text());
    }
}

fn argparse() -> ExplainArgs {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        panic!("improperly invoked");
    }

    // '--' is used to delim between args for this program,
    // and the args to explain
    let mut before_delim = true;

    let mut result = ExplainArgs {
        command_name: String::new(),
        command_args: Vec::new(),
        explain_args: Vec::new(),
    };

    for (i, arg) in args.iter().enumerate() {
        // first arg is always the os-provided bin path
        if i == 0 {
            continue;
        }

        // second arg is always the requested bin to explain
        if i == 1 {
            result.command_name = arg.to_owned();
        } else if before_delim {
            if arg == "--" {
                before_delim = false;
                continue;
            }

            // everything before '--' is an arg to explain
            result.command_args.push(arg.to_owned());
        } else {
            // everything after '--' is an arg to the 'explain' bin itself
            result.explain_args.push(arg.to_owned());
        }
    }

    result
}

fn get_manpage_path(program_name: &str) -> String {
    let output = Command::new("man")
        .arg("-w")
        .arg(program_name)
        .output()
        .expect("failed to invoke man");

    let formatted = format!("{}", String::from_utf8_lossy(&output.stdout));

    formatted.trim().into()
}

fn is_gzipped(path: &str) -> bool {
    path.ends_with(".gz")
}

fn unzip(zip_path: &str) -> String {
    let output = Command::new("gunzip")
        .arg("-c")
        .arg(zip_path)
        .output()
        .expect("failed to invoke gunzip");

    format!("{}", String::from_utf8_lossy(&output.stdout))
}

fn is_troff(text: &str) -> bool {
    text.starts_with(".TH")
}

fn print_usage() {
    println!("Usage: TODO");
}

fn read_file_content(file_path: &str) -> String {
    let mut file = File::open(file_path).expect(&format!("path not found: {}", &file_path));

    let mut content = String::new();

    if let Err(e) = file.read_to_string(&mut content) {
        panic!("Could not read file: {} with error: {}", file_path, e)
    }

    content
}
