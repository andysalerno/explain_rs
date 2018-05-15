use std::env;

pub struct ExplainArgs {
    pub command_name: String,
    pub command_args: Vec<String>,

    pub explain_args: Vec<String>,
    pub debug: bool,
}

pub fn argparse() -> ExplainArgs {
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
        debug: false,
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

            match arg.as_str() {
                "-d" => {
                    result.debug = true;
                }
                _ => {}
            }
        }
    }

    result
}

fn print_usage() {
    println!("Usage: TODO");
}
