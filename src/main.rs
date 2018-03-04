mod simple_parser;
mod man_parse;

use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use man_parse::troff_parser::TroffParser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let command_name = &args[1];
    println!("executing for program named: {}", &command_name);

    let man_path = get_manpage_path(command_name);
    println!("found manpath: [{}]", &man_path);

    let man_text = if is_gzipped(&man_path) {
        unzip(&man_path)
    } else {
        read_file_content(&man_path)
    };

    if !is_troff(&man_text) {
        println!("Man text does not appear to be troff.\nQuitting.");
        return;
    }

    let classifier = man_parse::troff_tokenize::TroffClassifier {};
    let tokenized = simple_parser::tokenizer::tokenize(&man_text, &classifier);

    let mut parser = TroffParser::new();
    parser.parse(tokenized.iter());

    println!("{:?}", &tokenized);
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
