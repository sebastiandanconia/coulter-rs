use std::process::Command;
use std::fs;

use gtk4::gio;
use gio::prelude::*;
use gio::Settings;
use gtk4;
use gtk4::prelude::*;
use gtk4::HeaderBar;
use gtk4::{glib,
        Application,
        ApplicationWindow,
        Button,
        Box as GtkBox,
        TextBuffer,
        TextView,
        StringList,
        Entry,
        Label,
        ScrolledWindow,
        Orientation as GtkOrientation,
        FileDialog,
        ComboBoxText,
        CheckButton,
        Grid,
        SpinButton,
        Window,
    };

use code2pdf::{
    EnscriptContext,
    Media,
    Orientation,
    Syntax,
    pretty_print
};

const APP_ID: &str = "com.github.username.sebastiandanconia.code2pdf";

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    gtk4::init().expect("Error initializing GTK GUI library");

    app.connect_activate(build_window_main);
    // Start the main application event loop
    app.run()
}

fn build_window_main(app: &Application) {

    let settings = Settings::new(APP_ID);

    // Main window outputs that are needed by following code
    let buffer_code = TextBuffer::new(None);
    buffer_code.set_text("Paste your code here.");
    let view_code = TextView::with_buffer(&buffer_code);
    let entry_pdftitle = Entry::new();
    entry_pdftitle.set_text("Untitled");

    // Build the header bar
    let headerbar = HeaderBar::new();

    // Button to open an existing file.
    let button_open = Button::new();
    button_open.set_label("📂");
    button_open.set_tooltip_text(Some("Open a source file"));

    // Button to generate PDF pretty-printed output.
    let button_generate = Button::new();
    button_generate.set_label("📃"); // Alternative icon: ("🚀");
    button_generate.set_tooltip_text(Some("Generate a PDF"));

    // Button to set configuration settings/preferences.
    let button_prefs = Button::new();
    button_prefs.set_label("⚙️");
    button_prefs.set_tooltip_text(Some("Settings"));

    // Finish building the header bar
    headerbar.pack_start(&button_open);
    headerbar.pack_start(&button_generate);
    headerbar.pack_end(&button_prefs);

    // "Below the fold" = "everything below the title bar"
    let box_belowthefold = GtkBox::builder()
        .orientation(GtkOrientation::Vertical)
        .spacing(12)
        .margin_end(12)
        .build();

    let box_pdftitle = GtkBox::builder()
        .orientation(GtkOrientation::Horizontal)
        .margin_start(12)
        .spacing(12)
        .build();

    let label_pdftitle = Label::new(Some("Title:"));
    box_pdftitle.append(&label_pdftitle);
    // TODO: Make spacing adjustments if desired
    box_pdftitle.append(&entry_pdftitle);

    // **Syntax Setting**
    let label_syntax = Label::new(Some("Syntax highlighting:"));
    let syntax_combo = ComboBoxText::new();
    // Define syntax options with id and display name
    let syntax_options = [
        ("", "None"),
        ("ada", "Ada95"),
        ("asm", "Assembler"),
        ("awk", "AWK"),
        ("bash", "Bourne-Again Shell"),
        ("c", "C"),
        ("changelog", "ChangeLog"),
        ("cpp", "C++"),
        ("csh", "C-Shell"),
        ("delphi", "Delphi"),
        ("diff", "Normal Diff"),
        ("diffs", "Side Diff"),
        ("diffu", "Unified Diff"),
        ("dylan", "Dylan"),
        ("eiffel", "Eiffel"),
        ("elisp", "Emacs Lisp"),
        ("erlang", "Erlang"),
        ("f90", "Fortran90"),
        ("forth", "Forth"),
        ("fortran", "Fortran77"),
        ("fortran_pp", "Fortran77 with CPP"),
        ("haskell", "Haskell"),
        ("html", "HTML"),
        ("icon", "Icon"),
        ("idl", "IDL (CORBA)"),
        ("inf", "INF Script"),
        ("java", "Java"),
        ("javascript", "JavaScript"),
        ("ksh", "Korn Shell"),
        ("lua", "Lua"),
        ("m4", "M4 Macro"),
        ("mail", "Mail/News"),
        ("makefile", "Makefile"),
        ("matlab", "Matlab"),
        ("nroff", "Nroff"),
        ("oberon2", "Oberon 2"),
        ("objc", "Objective-C"),
        ("octave", "Octave"),
        ("outline", "Outline"),
        ("oz", "Mozart/Oz"),
        ("pascal", "Pascal"),
        ("perl", "Perl"),
        ("postscript", "PostScript"),
        ("pyrex", "Pyrex"),
        ("python", "Python"),
        ("rfc", "RFC/Internet Draft"),
        ("ruby", "Ruby"),
        ("scheme", "Scheme"),
        ("sh", "Bourne Shell"),
        ("skill", "Skill"),
        ("smalltalk", "Smalltalk"),
        ("sml", "Standard ML"),
        ("sql", "SQL (Sybase 11)"),
        ("states", "States"),
        ("synopsys", "Synopsys DC Shell"),
        ("tcl", "Tcl"),
        ("tcsh", "TC-Shell"),
        ("tex", "TeX/LaTeX"),
        ("vba", "Visual Basic"),
        ("verilog", "Verilog"),
        ("vhdl", "VHDL"),
        ("vrml", "VRML"),
        ("wmlscript", "WMLScript"),
        ("zsh", "Z-Shell"),
    ];
    for &(id, name) in &syntax_options {
        syntax_combo.append(Some(id), name);
    }

    // Persist "Syntax highlighting language" in Settings, but prompt for it here in the main window
    settings.bind("syntax", &syntax_combo, "active-id").build();

    box_pdftitle.append(&label_syntax);
    box_pdftitle.append(&syntax_combo);

    box_belowthefold.append(&box_pdftitle);

    // Main text area
    let box_code = GtkBox::builder()
        .orientation(GtkOrientation::Vertical)
        .build();
    let scrolled_code = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&view_code)
        .build();
    box_code.append(&scrolled_code);
    box_belowthefold.append(&box_code);

    // Create the main window.
    let window_main = ApplicationWindow::builder()
        .application(app)
        .default_width(750)
        .default_height(1000)
        .title("Code → PDF")
        .child(&box_belowthefold)
        .build();
    window_main.set_titlebar(Some(&headerbar));


    ////////////////////////////////////////////////////////////////////////////////
    // Connect signal handlers
    ////////////////////////////////////////////////////////////////////////////////

    button_open.connect_clicked(glib::clone!(
        #[weak] buffer_code,
        #[weak] window_main,
        #[weak] entry_pdftitle,
        move |_| {
        let file_dialog = FileDialog::builder()
            .title("Choose a source file")
            .modal(true)
            .build();

        file_dialog.open(Some(&window_main), gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if let Ok(file_content) = fs::read_to_string(&path) {
                        entry_pdftitle.set_text(path.file_name().unwrap().to_str().unwrap());
                        buffer_code.set_text(&file_content);
                    } else {
                        eprintln!("Failed to read source file");
                    }
                }
            } else if let Err(e) = result {
                println!("File selection canceled or failed: {}", e);
            }
        });
    }));

    button_generate.connect_clicked(glib::clone!(
        #[weak] buffer_code,
        #[weak] entry_pdftitle,
        #[strong] settings,
        move |_| {
        // Extra binding needed per the compiler
        let pdftitle_binding = entry_pdftitle.text();
        let pdftitle = match pdftitle_binding.as_str() {
            "" => None,
            "Untitled" => None,
            t => Some(t),
        };
        let result = pretty_print(
            prepare_context(&settings,
                pdftitle,
                &buffer_code.text(
                    &buffer_code.start_iter(),
                    &buffer_code.end_iter(),
                    true
                ).as_str()
        ));

        if let Ok(outfile) = result {
            let mut pdf_viewer = Command::new(settings.get::<String>("pdf-viewer"));
            pdf_viewer.args([outfile]);
            let _pid = pdf_viewer.spawn();
        }
    }));

    button_prefs.connect_clicked(glib::clone!(#[weak] window_main, #[strong] settings, move |_| {
        build_window_prefs(&settings, &window_main);
    }));

    window_main.present();
}

fn prepare_context<'a>(settings: &gio::Settings, pdftitle: Option<&str>, code: &'a str) -> EnscriptContext<'a> {
    EnscriptContext {
        title: pdftitle.map(|s| s.to_owned()),
        media: settings.get::<String>("media").parse::<Media>().unwrap(),
        syntax: settings.get::<String>("syntax").parse::<Syntax>().ok(),
        font: settings.get::<String>("font"),
        color: settings.get::<bool>("color"),
        tabsize: settings.get::<u32>("tabsize"),
        columns: settings.get::<u32>("columns"),
        orientation: settings.get::<String>("orientation").parse::<Orientation>().unwrap(),
        code: code,
    }
}

fn build_window_prefs(settings: &Settings, parent: &ApplicationWindow) /*-> Window*/ {
    // Create a new GTK window for preferences
    let window = Window::new();
    window.set_title(Some("Preferences"));
    window.set_transient_for(Some(parent));
    window.set_modal(true);

    // Create a grid to organize the settings widgets
    let grid = Grid::new();
    grid.set_row_spacing(10);
    grid.set_column_spacing(10);
    grid.set_margin_start(10);
    grid.set_margin_end(10);
    grid.set_margin_top(10);
    grid.set_margin_bottom(10);

    let mut row = 0;

    // Media Setting
    let media_label = Label::new(Some("Paper size:"));
    grid.attach(&media_label, 0, row, 1, 1);
    let media_combo = ComboBoxText::new();
    let media_options = ["Letter", "Legal", "Tabloid", "A4", "B4"];
    for &media in &media_options {
        media_combo.append(Some(media), media);
    }

    grid.attach(&media_combo, 1, row, 1, 1);
    settings.bind("media", &media_combo, "active-id").build();
    row += 1;

    // Font Setting
    let font_label = Label::new(Some("Font:"));
    grid.attach(&font_label, 0, row, 1, 1);
    /* let font_button = FontButton::new(); */
    let font_combo = ComboBoxText::new();
    let font_list = ["Courier7", "Courier8", "Courier9", "Courier10", "Courier11", "Courier12"];
    for &font in &font_list {
        font_combo.append(Some(font), font);
    }
    settings.bind("font", &font_combo, "active-id").build();
    grid.attach(&font_combo, 1, row, 1, 1);
    row += 1;

    // Color Setting
    let color_label = Label::new(Some("Color:"));
    grid.attach(&color_label, 0, row, 1, 1);
    let color_check = CheckButton::new();
    grid.attach(&color_check, 1, row, 1, 1);
    settings.bind("color", &color_check, "active").build();
    row += 1;

    // Tabsize Setting
    let tabsize_label = Label::new(Some("Tab size:"));
    grid.attach(&tabsize_label, 0, row, 1, 1);
    let tabsize_spin = SpinButton::with_range(1.0, 16.0, 1.0);
    tabsize_spin.set_digits(0); // Display as integer
    grid.attach(&tabsize_spin, 1, row, 1, 1);
    settings.bind("tabsize", &tabsize_spin, "value").build();
    row += 1;

    // Columns Setting
    let columns_label = Label::new(Some("Columns:"));
    grid.attach(&columns_label, 0, row, 1, 1);
    let columns_spin = SpinButton::with_range(1.0, 4.0, 1.0);
    columns_spin.set_digits(0); // Display as integer
    grid.attach(&columns_spin, 1, row, 1, 1);
    settings.bind("columns", &columns_spin, "value").build();
    row += 1;

    // Orientation Setting
    let orientation_label = Label::new(Some("Orientation:"));
    grid.attach(&orientation_label, 0, row, 1, 1);
    let orientation_combo = ComboBoxText::new();
    let orientation_options = ["Portrait", "Landscape"];
    for &orientation in &orientation_options {
        orientation_combo.append(Some(orientation), orientation);
    }
    grid.attach(&orientation_combo, 1, row, 1, 1);
    settings.bind("orientation", &orientation_combo, "active-id").build();
    row += 1;

    // PDF Viewer Setting
    let pdfviewer_label = Label::new(Some("PDF viewer:"));
    grid.attach(&pdfviewer_label, 0, row, 1, 1);
    // TODO: FileDialog() .application(...)
    let pdfviewer_entry = Entry::new();
    grid.attach(&pdfviewer_entry, 1, row, 1, 1);
    settings.bind("pdf-viewer", &pdfviewer_entry, "text").build();
    row += 1;

    // **Close Button**
    let close_button = Button::with_label("Close");
    close_button.connect_clicked(glib::clone!(#[weak] window, move |_| {
        window.close();
    }));
    grid.attach(&close_button, 0, row, 2, 1); // Span both columns

    // Set the grid as the window's child
    window.set_child(Some(&grid));

    window.present();
}
