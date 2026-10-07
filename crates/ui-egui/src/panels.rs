use crate::AppUi;
use serde_json::json;

pub fn show(app: &mut AppUi, ui: &mut egui::Ui) {
    let state = app.host.session().inspect();
    if state.pointer("/view/panel_visible").and_then(serde_json::Value::as_bool) == Some(true) {
        egui::Panel::right("state").default_size(250.0).show(ui, |ui| {
            ui.heading("Application state");
            ui.monospace(serde_json::to_string_pretty(&state).unwrap_or_default());
            ui.separator();
            ui.heading("Commands");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for command in app.host.session().commands() {
                    ui.label(command.id).on_hover_text(command.description);
                }
            });
        });
    }
    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Application Template");
        ui.label("One model. One command path. Any frontend.");
        ui.add_space(16.0);
        let mut name = app.host.session().model().name.clone();
        ui.horizontal(|ui| {
            let label = ui.label("Name:");
            if ui.add(egui::TextEdit::singleline(&mut name).id_salt("name")).labelled_by(label.id).changed() {
                app.run("demo.setName", json!({"name": name}));
            }
        });
        ui.label(format!("Counter: {}", app.host.session().model().counter));
        ui.horizontal(|ui| {
            for (label, amount) in [("Increment", 1), ("Decrement", -1)] {
                if ui.button(label).clicked() {
                    app.run("demo.incrementCounter", json!({"amount": amount}));
                }
            }
        });
        if app.uses_path_field() {
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.label("Storage path:");
                ui.text_edit_singleline(&mut app.path);
            });
            ui.label("Open and Save use this path.");
        }
        if let Some(error) = &app.error {
            ui.colored_label(egui::Color32::from_rgb(180, 40, 40), error);
        }
    });
}
