mod arg_parse;
mod man_parse;
mod simple_parser;
mod text_format;

use arg_parse::arg_parse::argparse;
use man_parse::man_section::ManSection;
use man_parse::troff_parser::TroffParser;
use std::fs::File;
use std::io::Read;
use std::process::Command;

fn main() {
    let args = argparse();

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

    let classifier = man_parse::troff_tokenize::TroffClassifier {};
    let tokenized = simple_parser::tokenizer::tokenize(&man_text, &classifier);

    if args.debug {
        for tok in &tokenized {
            println!("{:?}", tok);
        }
    }

    let mut parser = match section {
        None => TroffParser::new(),
        Some(s) => TroffParser::for_section(ManSection::from(s.as_str())),
    };

    parser.parse(tokenized.iter());

    if section.is_some() {
        println!("before:\n{}", parser.before_section_text());
        println!("-----------------");
        println!("after:\n{}", parser.section_text());
    }
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

// fn is_troff(text: &str) -> bool {
//     text.starts_with(".TH")
// }

fn read_file_content(file_path: &str) -> String {
    let mut file = File::open(file_path).expect(&format!("path not found: {}", &file_path));

    let mut content = String::new();

    if let Err(e) = file.read_to_string(&mut content) {
        panic!("Could not read file: {} with error: {}", file_path, e)
    }

    content
}
