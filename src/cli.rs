use clap::{App, Arg};

pub fn build_cli() -> App<'static, 'static> {
    App::new("conda-leaves")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Lukasz Kielar, lukaszkielar@outlook.com")
        .about("Blazingly fast conda dependency management tool.")
        .arg(
            Arg::with_name("foo")
                .long("foo")
                .short("f")
                .help("Some help text describing the --foo arg"),
        )
        .after_help(
            "You can also run `conda-leaves SUBCOMMAND -h` to get more information about that subcommand.",
        )
        .help("
            conda-leaves v1.0\n\
            Blazingly fast conda dependency management tool\n\
            (C) lukaszkielar@outlook.com\n\n\

            USAGE: conda-leaves <opts> <command>\n\n\

            Options:\n\
            -h, --help       Display this message\n\
            -V, --version    Display version info\n\
            -s <stuff>       Do something with stuff\n\
            -v               Be verbose\n\n\

            Commmands:\n\
            help             Prints this message\n\
            work             Do some work",
        )
}
