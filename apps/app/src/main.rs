mod control;

use app_platform::{CommandHost, FileDialogs, NativeStorage};
use app_ui_egui::AppUi;
use std::error::Error;

struct NativeDialogs;

impl FileDialogs for NativeDialogs {
    fn open(&mut self) -> Result<Option<String>, app_core::AppError> {
        path_string(rfd::FileDialog::new().add_filter("Application state", &["json"]).pick_file())
    }

    fn save(&mut self) -> Result<Option<String>, app_core::AppError> {
        path_string(rfd::FileDialog::new().add_filter("Application state", &["json"]).set_file_name("app-state.json").save_file())
    }
}

fn path_string(path: Option<std::path::PathBuf>) -> Result<Option<String>, app_core::AppError> {
    path.map(|path| path.into_os_string().into_string().map_err(|_| app_core::AppError::Storage("selected path is not valid UTF-8".into())))
        .transpose()
}

struct DesktopApp {
    ui: AppUi,
    control: Option<control::ControlServer>,
    smoke_test: bool,
}

impl eframe::App for DesktopApp {
    fn logic(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(control) = &mut self.control {
            control.poll(self.ui.host_mut());
            context.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui.ui(ui);
        if self.smoke_test {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

struct Options {
    control: Option<control::ControlServer>,
    smoke_test: bool,
}

fn checked_arguments(args: impl Iterator<Item = std::ffi::OsString>) -> Result<Vec<String>, app_core::AppError> {
    let mut checked = Vec::new();
    let mut bytes = 0_usize;
    for argument in args {
        bytes = bytes.saturating_add(argument.as_encoded_bytes().len());
        if checked.len() >= 4096 || bytes > app_core::MAX_INPUT_BYTES {
            return Err(app_core::AppError::LimitExceeded("arguments exceed 4096 entries or 1048576 bytes".into()));
        }
        checked.push(argument.into_string().map_err(|_| app_core::AppError::InvalidParams("arguments must be valid UTF-8".into()))?);
    }
    Ok(checked)
}

fn arguments() -> Result<Option<Options>, Box<dyn Error>> {
    let mut args = checked_arguments(std::env::args_os().skip(1))?.into_iter();
    let mut options = Options { control: None, smoke_test: false };
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                println!("app [--control PORT] [--smoke-test]\nNative desktop application. Control binds 127.0.0.1 only.");
                return Ok(None);
            }
            "--version" => {
                println!("app {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "--control" if options.control.is_none() => {
                options.control = Some(control::ControlServer::bind(args.next().ok_or("missing control port")?.parse()?)?)
            }
            "--smoke-test" => options.smoke_test = true,
            _ => return Err("unknown or repeated argument; use --help".into()),
        }
    }
    Ok(Some(options))
}

fn main() -> Result<(), Box<dyn Error>> {
    let Some(arguments) = arguments()? else {
        return Ok(());
    };
    let options = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 560.0]), ..Default::default() };
    eframe::run_native(
        "Application Template",
        options,
        Box::new(move |_| {
            Ok(Box::new(DesktopApp {
                ui: AppUi::new(CommandHost::new(Box::new(NativeStorage))).with_dialogs(Box::new(NativeDialogs)),
                control: arguments.control,
                smoke_test: arguments.smoke_test,
            }))
        }),
    )
    .map_err(|error| error.to_string().into())
}

#[cfg(test)]
mod tests {
    use super::{checked_arguments, path_string};

    #[test]
    fn dialog_paths_preserve_names_and_cancellation() {
        assert_eq!(path_string(None).unwrap(), None);
        assert_eq!(path_string(Some("résumé.json".into())).unwrap(), Some("résumé.json".into()));
    }

    #[test]
    fn oversized_arguments_are_rejected() {
        assert!(checked_arguments(std::iter::repeat_n(std::ffi::OsString::from("x"), 4096)).is_ok());
        assert!(checked_arguments(std::iter::repeat_n(std::ffi::OsString::from("x"), 4097)).is_err());
        assert!(checked_arguments(std::iter::once("x".repeat(app_core::MAX_INPUT_BYTES + 1).into())).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_arguments_are_rejected() {
        use std::os::unix::ffi::OsStringExt as _;
        let argument = std::ffi::OsString::from_vec(vec![0xff]);
        assert!(checked_arguments(std::iter::once(argument)).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_dialog_path_is_rejected() {
        use std::os::unix::ffi::OsStringExt as _;
        let path = std::ffi::OsString::from_vec(vec![0xff]).into();
        assert!(path_string(Some(path)).is_err());
    }
}
