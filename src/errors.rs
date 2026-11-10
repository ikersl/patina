pub struct PatinaError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub help: Option<String>,
}

impl PatinaError {
    pub fn new(line: usize, col: usize, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line,
            col,
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

pub fn format_error(err: &PatinaError, source: &str, filename: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::new();

    // error: message
    out.push_str(&format!("\x1b[1;31merror\x1b[0m\x1b[1m: {}\x1b[0m\n", err.message));

    // --> file:line:col
    out.push_str(&format!(
        "  \x1b[1;34m-->\x1b[0m {}:{}:{}\n",
        filename, err.line, err.col
    ));

    if err.line > 0 && err.line <= lines.len() {
        let line_str = lines[err.line - 1];
        let gutter_width = err.line.to_string().len();

        // blank gutter
        out.push_str(&format!("{:>width$} \x1b[1;34m|\x1b[0m\n", "", width = gutter_width));

        // source line
        out.push_str(&format!(
            "\x1b[1;34m{:>width$} |\x1b[0m {}\n",
            err.line,
            line_str,
            width = gutter_width
        ));

        // caret
        let caret_offset = if err.col > 0 { err.col - 1 } else { 0 };
        out.push_str(&format!(
            "{:>width$} \x1b[1;34m|\x1b[0m {}\x1b[1;31m^\x1b[0m\n",
            "",
            " ".repeat(caret_offset),
            width = gutter_width
        ));
    }

    if let Some(help) = &err.help {
        out.push_str(&format!("  \x1b[1;32mhelp\x1b[0m: {help}\n"));
    }

    out
}

pub fn parse_legacy_error(msg: &str) -> PatinaError {
    // Parse "line X:Y: message" format from lexer/parser
    if let Some(rest) = msg.strip_prefix("line ") {
        if let Some(colon1) = rest.find(':') {
            let line: usize = rest[..colon1].parse().unwrap_or(0);
            let after = &rest[colon1 + 1..];
            if let Some(colon2) = after.find(':') {
                let col: usize = after[..colon2].parse().unwrap_or(0);
                let message = after[colon2 + 1..].trim().to_string();
                return PatinaError::new(line, col, message);
            }
        }
    }
    PatinaError::new(0, 0, msg)
}
