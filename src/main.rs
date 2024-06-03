use fork::Fork;
use std::{
    env,
    error::Error,
    fs::ReadDir,
    io::Write,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

mod args;
use args::Arg;

mod desktop_entry;
use desktop_entry::DesktopEntry;

fn parse_desktop_file(path: &Path) -> Result<DesktopEntry, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;
    Ok(DesktopEntry::new(&content)?)
}

fn parse_applications(entries: ReadDir, applications: &mut Vec<DesktopEntry>) {
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        match parse_desktop_file(&path) {
            Ok(desktop_file) => applications.push(desktop_file),
            Err(e) => eprintln!("Failed to parse '{}': {}", path.display(), e),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let xdg_data_dirs = env::var("XDG_DATA_DIRS").expect("XDG_DATA_DIRS is not set");
    let mut applications = Vec::new();
    for data_dir in xdg_data_dirs.split(':') {
        let applications_path = PathBuf::from(data_dir).join("applications");

        let entries = std::fs::read_dir(&applications_path);
        if let Ok(entries) = entries {
            parse_applications(entries, &mut applications);
        }
    }

    let output = {
        let mut fzf = Command::new("fzf")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(["--delimiter", ";", "--with-nth", "2.."])
            .spawn()?;

        {
            let mut stdin = fzf.stdin.take().expect("Failed to open fzf stdin");
            for (idx, app) in applications.iter().enumerate() {
                if app.no_display {
                    continue;
                }

                stdin
                    .write_all(format!("{};{}\n", idx, app.name).as_bytes())
                    .expect("Failed to write to fzf stdin");
            }
        }

        fzf.wait_with_output()?
    };

    let output = String::from_utf8_lossy(&output.stdout);
    let (choice_index, _) = output.split_once(';').ok_or("Delimiter not found")?;
    let choice_index = choice_index.parse::<usize>()?;
    let choice = &applications[choice_index];

    let mut args = args::parse_arguments(&choice.exec);
    let (arg0, args) = args.split_first_mut().unwrap();
    if let Arg::Text(arg0) = arg0 {
        // Clean args from special fields
        let clean_args = args.iter().filter_map(|arg| match arg {
            Arg::Text(text) => Some(text.as_str()),
            Arg::Field(_) => None,
        });
        if let Ok(Fork::Child) = fork::daemon(false, false) {
            Command::new(arg0)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .args(clean_args)
                .exec();
        }
    } else {
        eprintln!("Invalid first argument: {arg0:?}");
        std::process::exit(1);
    }

    Ok(())
}
