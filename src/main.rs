#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use device_query::{DeviceQuery, DeviceState, Keycode, Keycode::*};
use eframe::egui;
use egui::*;
pub mod toggle;
use midir::{Ignore, MidiInput, MidiInputPort};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Read,
    sync::{Arc, Mutex},
};
use phf::phf_map;
use toggle::toggle;
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let device_state = DeviceState::new();
    let options = eframe::NativeOptions::default();
    let mut midi_in = MidiInput::new("midir reading input").expect("failed to find midi input");
    let in_ports = midi_in.ports();
    let ip: Option<&MidiInputPort> = match in_ports.len() {
        0 => {
            println!("no inputs found");
            None
        }
        _ => Some(in_ports.get(0).expect("failed to index ports")),
    };
    let mut ctt = Content {
        binds: Arc::new(Mutex::new(Vec::<Binding>::new())),
        dev_state: device_state,
        // midi_in: midi_in,
        // midi_in_port: ip.expect("no midi ports available").clone(),
    };
    {
        let mut ctt = ctt.clone();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("cfg.txt")
            .expect("failed to open file");
        let mut sbuf = String::new();
        file.read_to_string(&mut sbuf)
            .expect("failed to read to string");
        let texts: Vec<String> = sbuf.split_terminator('\n').map(|x| x.to_string()).collect();
        for t in texts {
            ctt.binds.lock().unwrap().push(Binding::from_string(t));
        }
        let _conn_in = midi_in.connect(
            ip.expect("failed to find input port"),
            "midir-read-input",
            move |stamp, message, _| {
                if message[0] == 154 {
                    for i in ctt.binds.lock().unwrap().iter_mut() {
                        if i.selected {
                            i.note = message[1] as u32;
                        }
                    }
                }
                println!("{:?}", message)
            },
            (),
        );
    }
    eframe::run_native("Keyboard events", options, Box::new(|_cc| Box::new(ctt)))
}

#[derive(Clone)]
struct Content {
    binds: Arc<Mutex<Vec<Binding>>>,
    dev_state: DeviceState,
    // midi_in: MidiInput,
    // midi_in_port: MidiInputPort,
}
#[derive(Clone)]
struct Binding {
    pub note: u32,
    pub keys: Vec<Keycode>,
    pub label: String,
    pub selected: bool,
}
const KEYCODEMAP: phf::Map <&'static str, Keycode> = phf::phf_map! {
    "Key0" => Key0,
    "Key1" => Key1,
    "Key2" => Key2,
    "Key3" => Key3,
    "Key4" => Key4,
    "Key5" => Key5,
    "Key6" => Key6,
    "Key7" => Key7,
    "Key8" => Key8,
    "Key9" => Key9,
    "A" => A,
    "B" => B,
    "C" => C,
    "D" => D,
    "E" => E,
    "F" => F,
    "G" => G,
    "H" => H,
    "I" => I,
    "J" => J,
    "K" => K,
    "L" => L,
    "M" => M,
    "N" => N,
    "O" => O,
    "P" => P,
    "Q" => Q,
    "R" => R,
    "S" => S,
    "T" => T,
    "U" => U,
    "V" => V,
    "W" => W,
    "X" => X,
    "Y" => Y,
    "Z" => Z,
    "F1" => F1,
    "F2" => F2,
    "F3" => F3,
    "F4" => F4,
    "F5" => F5,
    "F6" => F6,
    "F7" => F7,
    "F8" => F8,
    "F9" => F9,
    "F10" => F10,
    "F11" => F11,
    "F12" => F12,
    "Escape" => Escape,
    "Space" => Space,
    "LControl" => LControl,
    "RControl" => RControl,
    "LShift" => LShift,
    "RShift" => RShift,
    "LAlt" => LAlt,
    "RAlt" => RAlt,
    "Meta" => Meta,
    "Enter" => Enter,
    "Up" => Up,
    "Down" => Down,
    "Left" => Left,
    "Right" => Right,
    "Backspace" => Backspace,
    "CapsLock" => CapsLock,
    "Tab" => Tab,
    "Home" => Home,
    "End" => End,
    "PageUp" => PageUp,
    "PageDown" => PageDown,
    "Insert" => Insert,
    "Delete" => Delete,
    "Numpad0" => Numpad0,
    "Numpad1" => Numpad1,
    "Numpad2" => Numpad2,
    "Numpad3" => Numpad3,
    "Numpad4" => Numpad4,
    "Numpad5" => Numpad5,
    "Numpad6" => Numpad6,
    "Numpad7" => Numpad7,
    "Numpad8" => Numpad8,
    "Numpad9" => Numpad9,
    "NumpadSubtract" => NumpadSubtract,
    "NumpadAdd" => NumpadAdd,
    "NumpadDivide" => NumpadDivide,
    "NumpadMultiply" => NumpadMultiply,
    "Grave" => Grave,
    "Minus" => Minus,
    "Equal" => Equal,
    "LeftBracket" => LeftBracket,
    "RightBracket" => RightBracket,
    "BackSlash" => BackSlash,
    "Semicolon" => Semicolon,
    "Apostrophe" => Apostrophe,
    "Comma" => Comma,
    "Dot" => Dot,
    "Slash" => Slash,
};
impl Binding {
    pub fn str_keycode(s: &str) -> Keycode {
        match KEYCODEMAP.get(s) {
            Some(k) => *k,
            None => Key0,
        }
    }
    pub fn keycode_str(k: Keycode) -> &'static str {
        match KEYCODEMAP.entries().find_map(|(st, ke)| if *ke == k {Some(st)} else {None}) {
            Some(key) => { *key}
            None => {"Key0"}
        }
    }

    pub fn from_string(input: String) -> Binding {
        let mut v: Vec<&str> = input.split_whitespace().collect();
        let mut o = Vec::<device_query::keymap::Keycode>::new();
        let l = *v.get(0).expect("failed to get first");
        let code = v
            .get(1)
            .expect("no second position")
            .parse::<u32>()
            .expect("failed to parse to number");
        v.remove(0);
        v.remove(1);
        for i in v {
            o.push(Self::str_keycode(i));
        }
        Binding {
            note: code,
            keys: o,
            label: l.to_owned(),
            selected: false,
        }
    }
    pub fn to_string(&self) -> String {
        let mut o = String::new();
        o.push_str((self.note.to_string() + " ").as_str());
        for b in &self.keys {
            o.push_str((Self::keycode_str(*b).to_owned() + " ").as_str());
        }
        o
    }
}
impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(&ctx, |ui| {
            if ui.button("add").clicked() {
                self.binds.lock().unwrap().push(Binding {
                    note: 0,
                    keys: self.dev_state.get_keys(),
                    label: "label".to_owned(),
                    selected: false,
                });
            }
            let mut i = 0;
            while i < self.binds.lock().unwrap().len() {
                ui.group(|ui| {
                    ui.text_edit_singleline(&mut (self.binds.lock().unwrap()[i].label));
                    ui.add(toggle(&mut self.binds.lock().unwrap()[i].selected));
                    ui.label(self.binds.lock().unwrap()[i].to_string());
                    if ui.button("remove").clicked() {
                        self.binds.lock().unwrap().remove(i);
                    } else {
                        i += 1;
                    }
                });
            }
        });
    }
}
