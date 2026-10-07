use app_ui_egui::AppUi;
use wasm_bindgen::JsCast as _;

struct WebApp(AppUi);

impl eframe::App for WebApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
}

pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(error) = launch().await {
            web_sys::console::error_1(&error);
            if let Some(element) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("loading")) {
                element.set_text_content(Some(&format!("Application failed to start: {error:?}")));
            }
        }
    });
}

async fn launch() -> Result<(), wasm_bindgen::JsValue> {
    let document = web_sys::window().and_then(|window| window.document()).ok_or("missing browser document")?;
    let canvas = document.get_element_by_id("app_canvas").ok_or("missing application canvas")?.dyn_into::<web_sys::HtmlCanvasElement>()?;
    eframe::WebRunner::new().start(canvas, eframe::WebOptions::default(), Box::new(|_| Ok(Box::new(WebApp(AppUi::default()))))).await?;
    if let Some(loading) = document.get_element_by_id("loading") {
        loading.remove();
    }
    Ok(())
}
