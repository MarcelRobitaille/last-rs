use chrono::{DateTime, Local};
use thiserror::Error;
use utmp_rs::UtmpEntry;

// An exit event (logout, system crash, powered off)
#[derive(Debug)]
enum Exit {
    Logout(DateTime<Local>),
    Crash(DateTime<Local>),
    Reboot(DateTime<Local>),
    StillLoggedIn,
}

// An enter event (login, system boot, etc.)
#[derive(Debug)]
pub struct Enter {
    user: String,
    host: String,
    line: String,
    login_time: DateTime<Local>,
    exit: Exit,
}

#[derive(Error, Debug)]
pub enum LastError {
    #[error(transparent)]
    UtmpParse(#[from] utmp_rs::ParseError),
}

// We found a login
// Now iterate through the next enrties to find the accomanying logout
// It will be the next DEAD_PROCESS with the same ut_line
// Source: `last` source code
fn find_accompanying_logout(entries: &[UtmpEntry], target_line: &str) -> Option<Exit> {
    entries.iter().rev().find_map(|x| match x {
        // If we see a DEAD_PROCESS with the same line as the login, then it's a logout event
        UtmpEntry::DeadProcess { line, time, .. } if line == target_line => {
            Some(Exit::Logout(DateTime::from(*time)))
        }
        // Kind of hacky, but a RUN_LVL with user "shutdown" is a shutdown event
        // Source: last.c from util-linux
        UtmpEntry::RunLevel { user, time, .. } if user == "shutdown" => {
            Some(Exit::Reboot(DateTime::from(*time)))
        }
        // Boot event
        UtmpEntry::BootTime(time) => Some(Exit::Crash(DateTime::from(*time))),
        // Not sure what magic is this
        // Taken from last.c in util-linux
        UtmpEntry::RunLevel {
            pid, user, time, ..
        } if user == "runlevel" && ['0', '6'].contains(&(pid.to_be_bytes()[1] as char)) => {
            Some(Exit::Reboot(DateTime::from(*time)))
        }
        _ => None,
    })
}

pub fn iter_logins() -> Result<Vec<Enter>, LastError> {
    // let entries = utmp_rs::parse_from_path("/var/run/utmp")?;
    let mut entries = utmp_rs::parse_from_path("/var/log/wtmp")?;
    entries.reverse();
    Ok(entries
        .iter()
        .enumerate()
        .filter_map(|(i, x)| match x {
            UtmpEntry::UserProcess {
                user,
                host,
                time,
                line,
                ..
            } => {
                let exit = find_accompanying_logout(&entries[..i], &line[..])
                    .unwrap_or(Exit::StillLoggedIn);
                Some(Enter {
                    user: user.to_owned(),
                    host: host.to_owned(),
                    line: line.to_owned(),
                    login_time: DateTime::from(*time),
                    exit,
                })
            }
            _ => None,
        })
        .collect())
    // .take(num_logins);
}

fn print() -> Result<(), LastError> {
    for entry in iter_logins()? {
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
    print().unwrap_or_else(|err| println!("{}", err));
}
