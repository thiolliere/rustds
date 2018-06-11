extern crate gtk;
extern crate gio;
extern crate rds;

use gtk::prelude::*;
use gio::prelude::*;
use std::env::args;
use std::rc::Rc;
use std::cell::RefCell;

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

macro_rules! build_adjustments_fn {
    ($($adj:ident: $sec:ident, $field:ident, $ty:ty,)+) => (
        fn connect_adjusments(builder: &gtk::Builder, ds: &Rc<RefCell<rds::DrumSynth>>) {
            $(
            let adjustment: gtk::Adjustment = builder.get_object(stringify!($adj)).unwrap();
            let ds_clone = ds.clone();
            adjustment.connect_value_changed(move |adj| {
                if let Ok(mut ds) = ds_clone.try_borrow_mut() {
                    ds.$sec.$field = adj.get_value() as $ty;
                }
            });
            )+
        }

        fn update_adjustments(builder: &gtk::Builder, ds: &Rc<RefCell<rds::DrumSynth>>) {
            let ds = ds.borrow();
            $(
            let adjustment: gtk::Adjustment = builder.get_object(stringify!($adj)).unwrap();
            adjustment.set_value(ds.$sec.$field as f64);
            )+
        }
    );
}

macro_rules! build_switches_fn {
    ($($switch:ident: $sec:ident, $field:ident,)+) => (
        fn connect_switches(builder: &gtk::Builder, ds: &Rc<RefCell<rds::DrumSynth>>) {
            $(
            let switch: gtk::Switch = builder.get_object(stringify!($switch)).unwrap();
            let ds_clone = ds.clone();
            switch.connect_activate(move |switch| {
                if let Ok(mut ds) = ds_clone.try_borrow_mut() {
                    ds.$sec.$field = switch.get_active();
                }
            });
            )+
        }

        fn update_switches(builder: &gtk::Builder, ds: &Rc<RefCell<rds::DrumSynth>>) {
            let ds = ds.borrow();
            $(
            let switch: gtk::Switch = builder.get_object(stringify!($switch)).unwrap();
            switch.set_active(ds.$sec.$field);
            )+
        }
    );
}

build_adjustments_fn! {
    tone_f1: Tone, F1, f32,
    tone_f2: Tone, F2, f32,
}

build_switches_fn! {
    noise_on: Noise, On,
}

fn main() {
    let application = gtk::Application::new("com.github.rds-studio", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

    let ds: Rc<RefCell<rds::DrumSynth>> = Rc::new(RefCell::new(rds::DrumSynth::new()));

    application.connect_startup(move |app| {
        let builder = gtk::Builder::new_from_file("ui/main.glade");

        let window: gtk::Window = builder.get_object("window").unwrap();
        let mutate: gtk::Button = builder.get_object("mutate").unwrap();

        connect_adjusments(&builder, &ds);
        connect_switches(&builder, &ds);

        mutate.connect_clicked(clone!(builder, ds => move |_| {
            // TODO: mutate
            update_switches(&builder, &ds);
            update_adjustments(&builder, &ds);
            // TODO: play sound
        }));

        window.set_application(app);
        window.connect_delete_event(clone!(window => move |_, _| {
            window.destroy();
            Inhibit(false)
        }));

        update_switches(&builder, &ds);
        update_adjustments(&builder, &ds);

        window.show_all();
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}
