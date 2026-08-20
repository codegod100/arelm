//! The actual relm4 application: a small counter component.
//!
//! This module is deliberately plain gtk4 (no libadwaita) so the pixiewood
//! manifest only needs to declare `glib` + `fontconfig` + `gtk` as
//! dependencies (see `pixiewood.xml`), matching what gtk-android-builder
//! ships wraps for out of the box.
//!
//! It is compiled into both:
//!   - the desktop `arelm` binary (src/main.rs), and
//!   - the Android cdylib (src/android.rs), loaded as a JNI library by the
//!     GTK-on-Android glue that pixiewood packages into the APK.
//! Same UI code, two entrypoints.

use relm4::gtk;
use relm4::gtk::glib::clone;
use relm4::gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

pub struct AppModel {
    counter: i32,
}

#[derive(Debug)]
pub enum AppMsg {
    Increment,
    Decrement,
}

pub struct AppWidgets {
    label: gtk::Label,
}

impl SimpleComponent for AppModel {
    type Init = i32;
    type Input = AppMsg;
    type Output = ();
    type Root = gtk::Window;
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("arelm")
            .default_width(320)
            .default_height(200)
            .build()
    }

    fn init(
        counter: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel { counter };

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(16)
            .margin_bottom(16)
            .margin_start(16)
            .margin_end(16)
            .valign(gtk::Align::Center)
            .build();

        let label = gtk::Label::new(Some(&format!("Counter: {}", model.counter)));
        label.set_css_classes(&["title-2"]);

        let button_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk::Align::Center)
            .build();

        let inc_button = gtk::Button::with_label("Increment");
        let dec_button = gtk::Button::with_label("Decrement");

        button_box.append(&dec_button);
        button_box.append(&inc_button);
        vbox.append(&label);
        vbox.append(&button_box);
        window.set_child(Some(&vbox));

        inc_button.connect_clicked(clone!(
            #[strong]
            sender,
            move |_| sender.input(AppMsg::Increment)
        ));
        dec_button.connect_clicked(clone!(
            #[strong]
            sender,
            move |_| sender.input(AppMsg::Decrement)
        ));

        let widgets = AppWidgets { label };
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppMsg::Increment => self.counter = self.counter.saturating_add(1),
            AppMsg::Decrement => self.counter = self.counter.saturating_sub(1),
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.label.set_label(&format!("Counter: {}", self.counter));
    }
}
