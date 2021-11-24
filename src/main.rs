use clap::{App, Arg};
use last_rs::{get_logins, Exit, LastError};

fn print(file: &str) -> Result<(), LastError> {
    for entry in get_logins(file)? {
        // println!("{:?}", entry);
        // let logout_time = match entry.exit {
        //     Logout::Message(message) => message,
        //     Logout::Time(time) => time.format("%H:%M").to_string(),
        // };
        let exit_text = match entry.exit {
            Exit::StillLoggedIn => "still logged in".to_string(),
            Exit::Logout(time) | Exit::Crash(time) | Exit::Reboot(time) => {
                let delta_time = time - entry.login_time;
                let delta_time = if delta_time.num_days() > 0 {
                    format!(
                        "({}+{:0>2}:{:0>2})",
                        delta_time.num_days(),
                        delta_time.num_hours() % 24,
                        delta_time.num_minutes() % 60,
                    )
                } else {
                    format!(
                        " ({:0>2}:{:0>2})",
                        delta_time.num_hours() % 24,
                        delta_time.num_minutes() % 60,
                    )
                };
                format!(
                    "{:<6}{}",
                    match entry.exit {
                        Exit::Logout(time) => time.format("%H:%M").to_string(),
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
            entry.login_time.format("%a %b %e %H:%M"),
            exit_text,
        );
    }

    Ok(())
}

fn main() {
    let matches = App::new("last")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .help(
                    "Tell last to use a specific file instead of /var/log/wtmp. \
                    The --file option can be given multiple times, \
                    and all of the specified files will be processed.",
                )
                .multiple(true)
                .number_of_values(1)
                .takes_value(true),
        )
        .get_matches();

    let files: Vec<_> = matches
        .values_of("file")
        .map_or_else(|| vec!["/var/log/wtmp"], Iterator::collect);

    for file in files {
        print(file).unwrap_or_else(|err| println!("{}", err));
    }
}
