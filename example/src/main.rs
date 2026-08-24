//! Example consumer of the `root` (arelm) cell. The consumer owns its
//! application ID, UI, and entrypoint; Arelm only supplies the coherent
//! Relm4/GTK dependency graph.
//!
//! From the repo root: `buck2 run example//:app`.
use arelm::relm4::gtk;
use gtk::prelude::*;

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.example.ArelmCellConsumer")
        .build();

    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Arelm cell consumer")
            .default_width(360)
            .default_height(200)
            .child(&gtk::Label::new(Some(
                "This UI and application ID belong to the consuming cell.",
            )))
            .build();
        window.present();
    });

    app.run();
}
