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
        fn connect_adjusments(builder: &gtk::Builder, ds: &Rc<RefCell<DrumSynthStack>>) {
            $(
            let adjustment: gtk::Adjustment = builder.get_object(stringify!($adj)).unwrap();
            let ds_clone = ds.clone();
            adjustment.connect_value_changed(move |adj| {
                let mut ds = ds_clone.borrow_mut();
                let ds = ds.change();
                ds.$sec.$field = adj.get_value() as $ty;
            });
            )+
        }

        fn update_adjustments(builder: &gtk::Builder, ds: &Rc<RefCell<DrumSynthStack>>) {
            let ds = ds.borrow().current().clone();
            $(
            let adjustment: gtk::Adjustment = builder.get_object(stringify!($adj)).unwrap();
            adjustment.set_value(ds.$sec.$field as f64);
            )+
        }
    );
}

macro_rules! build_switches_fn {
    ($($switch:ident: $sec:ident, $field:ident,)+) => (
        fn connect_switches(builder: &gtk::Builder, ds: &Rc<RefCell<DrumSynthStack>>) {
            $(
            let switch: gtk::Switch = builder.get_object(stringify!($switch)).unwrap();
            let ds_clone = ds.clone();
            switch.connect_property_active_notify(move |switch| {
                let mut ds = ds_clone.borrow_mut();
                let ds = ds.change();
                ds.$sec.$field = switch.get_active();
            });
            )+
        }

        fn update_switches(builder: &gtk::Builder, ds: &Rc<RefCell<DrumSynthStack>>) {
            let ds = ds.borrow().current().clone();
            $(
            let switch: gtk::Switch = builder.get_object(stringify!($switch)).unwrap();
            switch.set_active(ds.$sec.$field);
            )+
        }
    );
}

build_adjustments_fn! {
    distortion_bits: Distortion, Bits, i32,
    distortion_clipping: Distortion, Clipping, i32,
    distortion_rate: Distortion, Rate, i32,
    general_filter_high_pass: General, HighPass, i32,
    general_filter_level: General, Filter, i32,
    general_level: General, Level, f32,
    general_resonance: General, Resonance, f32,
    general_semitones_transpose: General, Tuning, f32,
    general_time_stretch: General, Stretch, f32,
    noise_band_1_freq: NoiseBand, F, f32,
    noise_band_1_level: NoiseBand, Level, i32,
    noise_band_1_width: NoiseBand, dF, i32,
    noise_band_2_freq: NoiseBand2, F, f32,
    noise_band_2_level: NoiseBand2, Level, i32,
    noise_band_2_width: NoiseBand2, dF, i32,
    noise_level: Noise, Level, i32,
    noise_slope: Noise, Slope, i32,
    overtones_a_freq: Overtones, F1, f32,
    overtones_a_mode: Overtones, Wave1, i32,
    overtones_b_freq: Overtones, F2, f32,
    overtones_b_mode: Overtones, Wave2, i32,
    overtones_filter: Overtones, Filter, i32,
    overtones_level: Overtones, Level, i32,
    overtones_method: Overtones, Method, i32,
    overtones_param: Overtones, Param, i32,
    tone_droop: Tone, Droop, f32,
    tone_f1: Tone, F1, f32,
    tone_f2: Tone, F2, f32,
    tone_level: Tone, Level, i32,
    tone_phase: Tone, Phase, f32,
}

build_switches_fn! {
    noise_on: Noise, On,
    distortion_on: Distortion, On,
    noise_band_1_on: NoiseBand, On,
    noise_band_2_on: NoiseBand2, On,
    overtones_on: Overtones, On,
    tone_on: Tone, On,
}

struct DrumSynthStack {
    stack: Vec<rds::DrumSynth>,
}

impl DrumSynthStack {
    fn change(&mut self) -> &mut rds::DrumSynth {
        let new = self.stack.last().unwrap().clone();
        self.stack.push(new);
        self.stack.last_mut().unwrap()
    }

    fn undo(&mut self) {
        if self.stack.len() >= 2 {
            self.stack.pop();
        }
    }

    fn current(&self) -> &rds::DrumSynth {
        self.stack.last().unwrap()
    }

    fn new() -> Self {
        DrumSynthStack {
            stack: vec![rds::DrumSynth::new()],
        }
    }
}

fn main() {
    let application = gtk::Application::new("com.github.rds-studio", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

    let ds: Rc<RefCell<DrumSynthStack>> = Rc::new(RefCell::new(DrumSynthStack::new()));

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
