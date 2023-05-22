use std::{
    fs::File,
    io::{BufReader, Read},
    process::exit,
    str::Split,
};

use handybars::Context;

fn resolve_input(path: Option<&String>) -> Box<dyn Read> {
    match path.map(|s| s.as_str()) {
        None | Some("-") => Box::new(std::io::stdin().lock()),
        Some(v) => Box::new(BufReader::new(File::open(v).expect("failed to open input"))),
    }
}
fn parse_defines(args: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut expecting_define = false;
    let define_prefixs = ["--define", "-D"];
    let mut push_define = |mut input: Split<char>| {
        out.push((
            input.next().expect("expected value after =").to_owned(),
            input
                .next()
                .expect("expected define of the form X=Y")
                .to_owned(),
        ));
    };
    for arg in args {
        if expecting_define {
            expecting_define = false;
            let input = arg.split('=');
            push_define(input);
        }
        if let Some(has_equals) = define_prefixs.iter().find_map(|p| {
            if arg.starts_with(p) {
                Some(arg.len() > p.len() && arg.as_bytes()[p.len()] as char == '=')
            } else {
                None
            }
        }) {
            expecting_define = !has_equals;
            if has_equals {
                let mut input = arg.split('=');
                input.next();
                push_define(input);
            }
        }
    }
    out
}
fn print_usage(path: &str) {
    print!(
        r"handybars - simple template expansion

Usage: {path} [INPUT|-] {{(--define|-D)=varname=value}}*

    '-' for INPUT is stdin, if INPUT is not provided it defaults to '-'

e.g.
> echo '{{ hello.world }}' | {path} - --define hello.world='hello world'
> hello world
"
    )
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.contains(&"--help".to_owned()) {
        print_usage(&args[0]);
        exit(1);
    }
    let defines = if args.len() > 2 {
        parse_defines(&args[2..])
    } else {
        Vec::new()
    };
    let mut ctx = Context::new();
    for (var, val) in defines {
        ctx.define(var.parse().expect("failed to parse define variable"), val);
    }

    let mut input = String::new();
    resolve_input(args.get(1))
        .read_to_string(&mut input)
        .expect("failed to read input");
    let output = ctx.render(&input).expect("failed to render template");
    print!("{}", output);
}
