extern crate gtk;

use gtk::prelude::*;
use gtk::{Button, Window, WindowType};

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let builder = gtk::Builder::new_from_file("ui/main.glade");

    let window: gtk::Window = builder.get_object("main").unwrap();

    window.show_all();

    gtk::main();
}
