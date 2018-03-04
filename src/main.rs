mod man_parse;

use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let command_name = &args[1];

    let man_text = read_man_page(command_name);

    println!("{}", man_text);
}

fn print_usage() {
    println!("Usage: TODO");
}

fn read_man_page(program_name: &str) -> String {
    let proto_path = format!("/usr/share/man/man1/{}.1", program_name);

    let mut file = File::open(&proto_path).expect(&format!("man page not found: {}", &proto_path));

    let mut content = String::new();

    if let Err(e) = file.read_to_string(&mut content) {
        panic!(
            "Could not open man page: {} with error: {}",
            program_name, e
        )
    }

    content
}
