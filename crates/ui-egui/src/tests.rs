use crate::AppUi;
use egui_kittest::{Harness, kittest::Queryable};

fn harness() -> Harness<'static, AppUi> {
    Harness::builder().with_size(egui::vec2(900.0, 560.0)).build_ui_state(|ui, app: &mut AppUi| app.ui(ui), AppUi::default())
}

#[test]
fn buttons_change_state_and_undo_through_commands() {
    let mut harness = harness();
    harness.get_by_label("Increment").click();
    harness.run();
    assert_eq!(harness.state().host().session().model().counter, 1);
    harness.get_by_label("Decrement").click();
    harness.run();
    assert_eq!(harness.state().host().session().model().counter, 0);
    harness.get_by_label("Edit").click();
    harness.run();
    harness.get_by_label("Undo").click();
    harness.run();
    assert_eq!(harness.state().host().session().model().counter, 1);
}

#[test]
fn typing_updates_model_and_panel_button_updates_view() {
    let mut harness = harness();
    harness.get_by_label("Name:").click();
    harness.run();
    harness.get_by_label("Name:").type_text("Example");
    harness.run();
    assert_eq!(harness.state().host().session().model().name, "Example");
    let before = harness.state().host().session().inspect();
    harness.get_by_label("Toggle panel").click();
    harness.run();
    assert_ne!(harness.state().host().session().inspect().get("view"), before.get("view"));
}
