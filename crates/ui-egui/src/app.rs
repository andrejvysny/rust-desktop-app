use app_platform::{CommandHost, FileDialogs};
use serde_json::{Value, json};

pub struct AppUi {
    pub(crate) host: CommandHost,
    pub(crate) error: Option<String>,
    pub(crate) path: String,
    dialogs: Option<Box<dyn FileDialogs>>,
}

impl Default for AppUi {
    fn default() -> Self {
        Self::new(CommandHost::default())
    }
}

impl AppUi {
    pub fn new(host: CommandHost) -> Self {
        Self { host, error: None, path: "app-state.json".into(), dialogs: None }
    }

    pub fn with_dialogs(mut self, dialogs: Box<dyn FileDialogs>) -> Self {
        self.dialogs = Some(dialogs);
        self
    }

    pub fn host(&self) -> &CommandHost {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut CommandHost {
        &mut self.host
    }

    pub(crate) fn uses_path_field(&self) -> bool {
        self.dialogs.is_none()
    }

    pub fn run(&mut self, command: &str, params: Value) {
        self.error = self.host.execute(command, params).err().map(|error| error.to_string());
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        super::theme::apply(ui.ctx());
        super::menus::show(self, ui);
        super::panels::show(self, ui);
    }

    pub(crate) fn file_action(&mut self, command: &str) {
        let picked = match self.dialogs.as_mut() {
            Some(dialogs) if command == "app.open" => dialogs.open(),
            Some(dialogs) => dialogs.save(),
            None => Ok(Some(self.path.clone())),
        };
        match picked {
            Ok(Some(path)) => {
                self.path.clone_from(&path);
                self.run(command, json!({"path": path}));
            }
            Ok(None) => {}
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}
