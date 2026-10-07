use app_core::{AppError, MAX_INPUT_BYTES};
use serde_json::{Value, json};

pub(crate) struct Options {
    pub action: String,
    pub load: Option<String>,
    pub save: Option<String>,
    pub commands: Vec<(String, Value)>,
}

impl Options {
    pub fn parse(args: impl Iterator<Item = String>) -> Result<Self, AppError> {
        let mut args = args.peekable();
        let action = args.next().unwrap_or_else(|| "help".into());
        let mut options = Self { action, load: None, save: None, commands: Vec::new() };
        if options.action == "--help" || options.action == "-h" {
            options.action = "help".into();
        }
        if !["commands", "state", "run", "mcp", "help"].contains(&options.action.as_str()) {
            return Err(invalid("expected commands, state, run, mcp, or help"));
        }
        if options.action == "run" && args.peek().is_some_and(|arg| !arg.starts_with('-')) {
            let command = next(&mut args, "command")?;
            let params = if args.peek().is_some_and(|arg| !arg.starts_with('-')) { parse_json(&next(&mut args, "JSON params")?)? } else { json!({}) };
            options.commands.push((command, params));
        }
        options.flags(args)?;
        if options.action == "run" && options.commands.is_empty() {
            return Err(invalid("run requires COMMAND or --cmd ID=JSON"));
        }
        Ok(options)
    }

    fn flags(&mut self, mut args: impl Iterator<Item = String>) -> Result<(), AppError> {
        while let Some(flag) = args.next() {
            if self.action == "mcp" || self.action == "help" {
                return Err(invalid("mcp and help accept no options"));
            }
            match flag.as_str() {
                "--load" if self.load.is_none() => self.load = Some(next(&mut args, "load path")?),
                "--save" if self.save.is_none() => self.save = Some(next(&mut args, "save path")?),
                "--cmd" if self.action == "run" => {
                    let item = next(&mut args, "ID=JSON")?;
                    let (id, params) = item.split_once('=').ok_or_else(|| invalid("--cmd requires ID=JSON"))?;
                    if id.is_empty() {
                        return Err(invalid("command ID cannot be empty"));
                    }
                    if self.commands.len() >= 1024 {
                        return Err(invalid("too many commands"));
                    }
                    self.commands.push((id.into(), parse_json(params)?));
                }
                _ => return Err(invalid(&format!("unknown or repeated option: {flag}"))),
            }
        }
        Ok(())
    }
}

fn next(args: &mut impl Iterator<Item = String>, label: &str) -> Result<String, AppError> {
    let value = args.next().ok_or_else(|| invalid(&format!("missing {label}")))?;
    if value.len() > MAX_INPUT_BYTES {
        return Err(AppError::LimitExceeded("argument exceeds byte limit".into()));
    }
    Ok(value)
}

fn parse_json(text: &str) -> Result<Value, AppError> {
    if text.len() > MAX_INPUT_BYTES {
        return Err(AppError::LimitExceeded("params exceed byte limit".into()));
    }
    let value: Value = serde_json::from_str(text).map_err(|error| invalid(&error.to_string()))?;
    if !value.is_object() {
        return Err(invalid("command params must be an object"));
    }
    Ok(value)
}

fn invalid(message: &str) -> AppError {
    AppError::InvalidParams(message.into())
}
