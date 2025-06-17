use clap::Parser;
use last_rs::{get_logins, Enter, Exit, LastError};
use std::path::Path;
use thiserror::Error;
use time::error::Format as TimeFormatError;
use time::error::IndeterminateOffset as TimeIndeterminateOffsetError;
use time::macros::format_description;
use time::UtcOffset;

#[derive(Error, Debug)]
pub enum PrintError {
    #[error(transparent)]
    Api(#[from] LastError),

    #[error(transparent)]
    TimeFormat(#[from] TimeFormatError),

    #[error(transparent)]
    TimeIndeterminateOffset(#[from] TimeIndeterminateOffsetError),
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    Print(#[from] PrintError),

    #[error("Could not parse option `{option}'. Expected `{expected}', found `{found}'.")]
    Argument {
        option: String,
        expected: String,
        found: String,
    },
}

// TODO: This is using the last login, which does not fully match up with the first entry.
// Looks like that's what last.c is using
fn prepare_footer(entries: &[Enter], file: &str, local_offset: UtcOffset) -> Option<String> {
    let last = entries.last()?;
    let name = Path::new(file).file_name()?.to_str()?;
    Some(format!(
        "{} begins {}",
        name,
        last.login_time.to_offset(local_offset).format(format_description!("[weekday repr:short] [month repr:short] [day padding:space] [hour]:[minute]:[second] [year]")).ok()?,
    ))
}

fn print(file: &str, number: Option<usize>) -> Result<(), PrintError> {
    let local_offset = UtcOffset::current_local_offset()?;

    let entries = get_logins(file)?;

    let footer = prepare_footer(&entries, file, local_offset);

    for entry in entries.iter().take(number.unwrap_or(entries.len())) {
        let exit_text = match entry.exit {
            Exit::StillLoggedIn => "still logged in".to_string(),
            Exit::Logout(time) | Exit::Crash(time) | Exit::Reboot(time) => {
                let delta_time = time - entry.login_time;
                let delta_time = if delta_time.whole_days() > 0 {
                    format!(
                        "({}+{:0>2}:{:0>2})",
                        delta_time.whole_days(),
                        delta_time.whole_hours() % 24,
                        delta_time.whole_minutes() % 60,
                    )
                } else {
                    format!(
                        " ({:0>2}:{:0>2})",
                        delta_time.whole_hours() % 24,
                        delta_time.whole_minutes() % 60,
                    )
                };
                format!(
                    "{:<6}{}",
                    match entry.exit {
                        Exit::Logout(time) => time
                            .to_offset(local_offset)
                            .format(format_description!("[hour]:[minute]"))?,
                        Exit::Crash(_) => "crash".to_string(),
                        Exit::Reboot(_) => "down".to_string(),
                        _ => unreachable!(),
                    },
                    delta_time
                )
            }
        };
        println!(
            "{:<9}{:<13}{:<17}{} - {}",
            entry.user,
            entry.line,
            entry.host,
            entry
                .login_time
                .to_offset(local_offset)
                .format(format_description!(
                    "[weekday repr:short] [month repr:short] [day padding:space] [hour]:[minute]"
                ))?,
            exit_text,
        );
    }

    if let Some(footer) = footer {
        println!();
        println!("{}", footer);
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(
    author = "Marcel Robitaille",
    version,
    about = "A Rust reimplementation of the util-linux last command."
)]
struct Arguments {
    #[arg(short, long, num_args = 1.., help=
        "Tell last to use a specific file instead of /var/log/wtmp.
        The --file option can be given multiple times,
        and all of the specified files will be processed.")]
    file: Vec<String>,

    #[arg(short, long = "limit", help = "Tell last how many lines to show.")]
    number: Option<usize>,
}

fn cli() -> Result<(), CliError> {
    let args = Arguments::parse();

    let files = if args.file.is_empty() {
        vec!["/var/log/wtmp".to_string()]
    } else {
        args.file
    };

    for file in files {
        print(&file, args.number)?;
    }

    Ok(())
}

// Small wrapper to handle errors in `cli()`
fn main() {
    cli().unwrap_or_else(|err| println!("last: {}", err));
}
