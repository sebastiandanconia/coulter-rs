
//use crate::Config;

use code2pdf::Config;
use code2pdf::pretty_print;
use gtk::ScrolledWindow;

use std::process::Command;
use std::sync::Arc;

use gtk;
use gtk::prelude::*;
use gtk::HeaderBar;
use gtk::{glib, Application, ApplicationWindow};
use gtk::{Button, TextView, TextBuffer};

const APP_ID: &str = "com.github.username.sebastiandanconia.code2pdf";


fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    gtk::init().expect("Error initializing GTK GUI library");

    let header_bar = HeaderBar::new();

    let buffer = Arc::new(TextBuffer::new(None));
    buffer.set_text("Paste your code here.");

    let view = TextView::with_buffer(buffer.as_ref());

    let buffer_arc = Arc::clone(&buffer);

    // Button to open an existing file. Not currently implemented.
    let button_open = Button::new();
    button_open.set_label("📂");

    // Button to generate PDF pretty-printed output.
    let button_generate = Button::new();
    //button_generate.set_icon_name("document-send");
    button_generate.set_label("📃"); // Alternative icon: ("🚀");
    button_generate.connect_clicked(move |_| {
        //eprintln!("Button clicked [OK]");
        let generate_result = pretty_print(
            Config::default(),
        &buffer_arc.as_ref().text(
            &buffer_arc.as_ref().start_iter(),
            &buffer_arc.as_ref().end_iter(),
            true)
            .to_string());

        if let Ok(outfile) = generate_result {
            let mut pdf_viewer = Command::new("evince");
            pdf_viewer.args([outfile]);
            let _pid = pdf_viewer.spawn();
        }

    });

    // Button to set configuration settings/preferences.
    let button_prefs = Button::new();
    button_prefs.set_label("⚙️"); // .set_icon_name("applications-multimedia");

    header_bar.pack_start(&button_open);
    header_bar.pack_start(&button_generate);
    header_bar.pack_end(&button_prefs);


    let scrolled = ScrolledWindow::builder()
        .child(&view)
        .build();

    app.connect_activate(move |app| {
        // We create the main window.
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(750)
            .default_height(1000)
            .title("Code → PDF")
            .child(&scrolled)
            .build();

        window.set_titlebar(Some(&header_bar));

        // Show the window.
        window.present();
    });

    app.run()
}