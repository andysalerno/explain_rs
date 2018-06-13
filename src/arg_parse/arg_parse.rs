use man_parse::man_section::ManSection;
use std::env;

#[derive(Default)]
pub struct ExplainArgs {
    pub command_name: String,
    pub command_args: Vec<String>,

    pub debug: bool,
    pub help: bool,
    pub section: Option<ManSection>,
}

// Optional.  Which section should we parse through?
const SHORT_SECTION_ARG: &str = "-s=";
const LONG_SECTION_ARG: &str = "--section=";

pub fn argparse() -> ExplainArgs {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        ::std::process::exit(1);
    }

    let mut result = ExplainArgs::default();

    let mut args_iter = args.into_iter();

    // first arg is always our calling path, ignore it
    args_iter.next();

    // first loop is for optional meta-arguments
    while let Some(arg) = args_iter.next() {
        if !arg.starts_with("-") {
            // first non-flagged arg is always the command to explain
            result.command_name = arg.to_owned();
            break;
        }

        // there may be optional flags provided to inform how explain should execute
        match arg.as_str() {
            "-d" | "--debug" => result.debug = true,
            "-h" | "--help" => result.help = true,
            s if s.starts_with(SHORT_SECTION_ARG) | s.starts_with(LONG_SECTION_ARG) => {
                result.section = parse_section_arg(s)
            }
            _ => {}
        };
    }

    // rest of the iteration is for the arguments of the given command
    while let Some(arg) = args_iter.next() {
        result.command_args.push(arg.to_owned());
    }

    result
}

fn parse_section_arg(section_arg: &str) -> Option<ManSection> {
    if section_arg.starts_with(SHORT_SECTION_ARG) {
        Some(ManSection::from(&section_arg[SHORT_SECTION_ARG.len()..]))
    } else if section_arg.starts_with(LONG_SECTION_ARG) {
        Some(ManSection::from(&section_arg[LONG_SECTION_ARG.len()..]))
    } else {
        None
    }
}

fn print_usage() {
    println!("Usage: TODO");
}
