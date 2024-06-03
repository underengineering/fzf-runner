pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub no_display: bool,
}

impl DesktopEntry {
    pub fn new(content: &str) -> Result<Self, &'static str> {
        let mut name = None;
        let mut exec = None;
        let mut no_display = false;

        let mut in_desktop_entry = false;
        for line in content.lines() {
            if !in_desktop_entry && line.starts_with("[Desktop Entry]") {
                // Find first desktop entry
                in_desktop_entry = true;
            } else if in_desktop_entry && line.starts_with('[') {
                // Next entry started, break
                break;
            } else if in_desktop_entry && !line.starts_with('#') {
                if let Some((key, value)) = line.split_once('=') {
                    if key == "Name" {
                        name = Some(value);
                    } else if key == "Exec" {
                        exec = Some(value);
                    } else if key == "NoDisplay" {
                        no_display = value == "true";
                    }
                }
            }
        }

        Ok(Self {
            name: name.ok_or("No name set")?.to_string(),
            exec: exec.ok_or("No exec set")?.to_string(),
            no_display,
        })
    }
}
