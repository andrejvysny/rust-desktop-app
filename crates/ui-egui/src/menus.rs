use crate::AppUi;
use serde_json::json;

pub fn show(app: &mut AppUi, ui: &mut egui::Ui) {
    egui::Panel::top("menu").show(ui, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    app.run("app.new", json!({}));
                    ui.close();
                }
                for (label, command) in [("Open…", "app.open"), ("Save…", "app.save")] {
                    if ui.button(label).clicked() {
                        app.file_action(command);
                        ui.close();
                    }
                }
            });
            ui.menu_button("Edit", |ui| {
                for (label, command) in [("Undo", "edit.undo"), ("Redo", "edit.redo")] {
                    if ui.add_enabled(app.host.session().is_enabled(command), egui::Button::new(label)).clicked() {
                        app.run(command, json!({}));
                        ui.close();
                    }
                }
            });
            if ui.button("Toggle panel").clicked() {
                app.run("view.togglePanel", json!({}));
            }
        });
    });
}
