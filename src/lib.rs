use thiserror::Error;
use time::OffsetDateTime;
use utmp_rs::UtmpEntry;

// An exit event (logout, system crash, powered off)
#[derive(Debug)]
pub enum Exit {
    Logout(OffsetDateTime),
    Crash(OffsetDateTime),
    Reboot(OffsetDateTime),
    StillLoggedIn,
}

// An enter event (login, system boot, etc.)
#[derive(Debug)]
pub struct Enter {
    pub user: String,
    pub host: String,
    pub line: String,
    pub login_time: OffsetDateTime,
    pub exit: Exit,
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
            Some(Exit::Logout(*time))
        }
        UtmpEntry::ShutdownTime { time, .. } => Some(Exit::Reboot(*time)),
        UtmpEntry::BootTime { time, .. } => Some(Exit::Crash(*time)),
        _ => None,
    })
}

pub fn get_logins(file: &str) -> Result<Vec<Enter>, LastError> {
    // let entries = utmp_rs::parse_from_path("/var/run/utmp")?;
    let mut entries = utmp_rs::parse_from_path(file)?;
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
                    login_time: *time,
                    exit,
                })
            }
            _ => None,
        })
        .collect())
}
