#[macro_use]
extern crate clap;

#[macro_use]
extern crate slog;

use clap::Shell;
use cursive::views::{Dialog, TextView};

use slog::Drain;
use std::io;

fn main() {
    let args = app_from_crate!()
        .about("Automatically absorb staged changes into your current branch")
        .arg(
            clap::Arg::with_name("base")
                .help("Use this commit as the base of the hanoi stack")
                .short("b")
                .long("base")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("dry-run")
                .help("Don't make any actual changes")
                .short("n")
                .long("dry-run")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("force")
                .help("Skip safety checks")
                .short("f")
                .long("force")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("verbose")
                .help("Display more output")
                .short("v")
                .long("verbose")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("and-rebase")
                .help("Run rebase if successful")
                .short("r")
                .long("and-rebase")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("gen-completions")
                .help("Generate completions")
                .long("gen-completions")
                .takes_value(true)
                .possible_values(&["bash", "fish", "zsh", "powershell", "elvish"]),
        )
        .arg(
            clap::Arg::with_name("whole-file")
                .help("Match the change against the complete file   ")
                .short("w")
                .long("whole-file")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("one-fixup-per-commit")
                .help("Only generate one fixup per commit")
                .short("F")
                .long("one-fixup-per-commit")
                .takes_value(false),
        );
    let mut args_clone = args.clone();
    let args = args.get_matches();

    if let Some(shell) = args.value_of("gen-completions") {
        let app_name = "git-hanoi";
        match shell {
            "bash" => {
                args_clone.gen_completions_to(app_name, Shell::Bash, &mut io::stdout());
            }
            "fish" => {
                args_clone.gen_completions_to(app_name, Shell::Fish, &mut io::stdout());
            }
            "zsh" => {
                args_clone.gen_completions_to(app_name, Shell::Zsh, &mut io::stdout());
            }
            "powershell" => {
                args_clone.gen_completions_to(app_name, Shell::PowerShell, &mut io::stdout());
            }
            "elvish" => {
                args_clone.gen_completions_to(app_name, Shell::Elvish, &mut io::stdout());
            }
            _ => unreachable!(),
        }
        return;
    }

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter::new(
        drain,
        if args.is_present("verbose") {
            slog::Level::Debug
        } else {
            slog::Level::Info
        },
    )
    .fuse();
    let mut logger = slog::Logger::root(drain, o!());
    if args.is_present("verbose") {
        logger = logger.new(o!(
            "module" => slog::FnValue(|record| record.module()),
            "line" => slog::FnValue(|record| record.line()),
        ));
    }

    // Creates the cursive root - required for every application.
    let mut siv = cursive::default();

    // Creates a dialog with a single "Quit" button
    siv.add_layer(
        Dialog::around(TextView::new("Hello Dialog!"))
            .title("Cursive")
            .button("Quit", |s| s.quit()),
    );

    // Starts the event loop.
    siv.run();
    if let Err(e) = git_hanoi::run(&git_hanoi::Config {
        dry_run: args.is_present("dry-run"),
        force: args.is_present("force"),
        base: args.value_of("base"),
        and_rebase: args.is_present("and-rebase"),
        whole_file: args.is_present("whole-file"),
        one_fixup_per_commit: args.is_present("one-fixup-per-commit"),
        logger: &logger,
    }) {
        crit!(logger, "hanoi failed"; "err" => e.to_string());
        // wait for async logger to finish writing messages
        drop(logger);
        ::std::process::exit(1);
    }
}
