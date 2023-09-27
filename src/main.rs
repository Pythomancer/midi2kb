#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use device_query::{DeviceQuery, DeviceState, Keycode, Keycode::*};
use eframe::egui;
pub mod toggle;
use active_win_pos_rs::get_active_window;
use enigo::{Enigo, Key, KeyboardControllable};
use midir::{MidiInput, MidiInputPort, MidiOutput};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    sync::{Arc, Mutex, OnceLock},
    thread::{self, JoinHandle},
};
use toggle::toggle;
const ICON: &[u8] = include_bytes!("../icon.png");
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        icon_data: Some(eframe::IconData::try_from_png_bytes(ICON).expect("failed to find image")),
        ..Default::default()
    };
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let device_state = DeviceState::new();

    let ctt = Content {
        binds: Arc::new(Mutex::new(Vec::<Binding>::new())),
        dev_state: device_state,
    };
    {
        let binds = Arc::clone(&ctt.binds);
        CELL.set(std::thread::spawn(move || {
            let midi_in = MidiInput::new("midir reading input").expect("failed to find midi input");
            let midi_out =
                MidiOutput::new("midir writing output").expect("failed to create midi output");
            // let _mioc = midi_out
            //     .create_virtual("midi2kb_o")
            //     .expect("failed to create virtual out");
            let ports = midi_out.ports();
            let midi_out_port = ports
                .iter()
                .filter(|mio| {
                    midi_out
                        .port_name(mio)
                        .expect("no output portname found")
                        .contains("midi2")
                })
                .next()
                .expect("no found named port");
            let mut mioc = midi_out
                .connect(midi_out_port, "midi2lr")
                .expect("failed to create output");
            let in_ports = midi_in.ports();
            let ip: Option<&MidiInputPort> = match in_ports.len() {
                0 => {
                    println!("no inputs found");
                    None
                }
                _ => Some(
                    in_ports
                        .iter()
                        .filter(|mi| {
                            midi_in
                                .port_name(mi)
                                .expect("failed to get in name")
                                .contains("TOUCH")
                        })
                        .next()
                        .expect("no kb found"),
                ),
            };
            let mut enigo = Enigo::new();
            let _conn_in = {
                midi_in
                    .connect(
                        ip.expect("failed to find input port"),
                        "midir-read-input",
                        move |_stamp, message, _| {
                            // println!("{:?}", message);
                            let appname =
                                get_active_window().expect("failed to get window").app_name;
                            let steal =
                                appname.contains("Photoshop") && !appname.contains("Lightroom");
                            let matching = appname.contains("midi2kb");
                            if message[0] == 154 {
                                for i in binds.lock().unwrap().iter_mut() {
                                    if i.note == message[1] && steal {
                                        for k in &i.keys {
                                            enigo.key_down(enigo_map(*k));
                                        }
                                        for k in &i.keys {
                                            enigo.key_up(enigo_map(*k));
                                        }
                                    }
                                    if matching && i.selected {
                                        i.note = message[1];
                                        i.selected = false;
                                    }
                                }
                                if !steal && !matching {
                                    println!("writing, {}", appname);
                                    let _ = mioc.send(message);
                                }
                            }
                        },
                        (),
                    )
                    .unwrap()
            };
            thread::park();
            // std::thread::sleep(std::time::Duration::from_secs(1000));
        }))
        .expect("failed to write join handle to cell");
    }
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
    for t in texts.iter().filter(|a| a.len() > 1) {
        ctt.binds
            .lock()
            .unwrap()
            .push(Binding::from_string(t.to_string()));
    }

    eframe::run_native("MIDI2KB", options, Box::new(|_cc| Box::new(ctt)))
}

#[derive(Clone)]
struct Content {
    binds: Arc<Mutex<Vec<Binding>>>,
    dev_state: DeviceState,
}
impl Content {
    pub fn save_to_file(&self, filename: String) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filename)
            .expect("failed to open file");
        let mut sbuf = String::new();

        for b in self.binds.lock().unwrap().iter() {
            println!("{:#?}", b);
            sbuf += (b.to_save() + "\n").as_str();
        }
        sbuf.pop();
        file.write(sbuf.as_bytes()).expect("failed to write file");
    }
}
#[derive(Clone, Debug)]
struct Binding {
    pub note: u8,
    pub keys: Vec<Keycode>,
    pub label: String,
    pub selected: bool,
}
static CELL: OnceLock<JoinHandle<()>> = OnceLock::new();
const KEYCODEMAP: phf::Map<&'static str, Keycode> = phf::phf_map! {
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
pub fn enigo_map(k: Keycode) -> enigo::keycodes::Key {
    match k {
        Key0 => Key::Layout('0'),
        Key1 => Key::Layout('1'),
        Key2 => Key::Layout('2'),
        Key3 => Key::Layout('3'),
        Key4 => Key::Layout('4'),
        Key5 => Key::Layout('5'),
        Key6 => Key::Layout('6'),
        Key7 => Key::Layout('7'),
        Key8 => Key::Layout('8'),
        Key9 => Key::Layout('9'),
        A => Key::Layout('a'),
        B => Key::Layout('b'),
        C => Key::Layout('c'),
        D => Key::Layout('d'),
        E => Key::Layout('e'),
        F => Key::Layout('f'),
        G => Key::Layout('g'),
        H => Key::Layout('h'),
        I => Key::Layout('i'),
        J => Key::Layout('j'),
        K => Key::Layout('k'),
        L => Key::Layout('l'),
        M => Key::Layout('m'),
        N => Key::Layout('n'),
        O => Key::Layout('o'),
        P => Key::Layout('p'),
        Q => Key::Layout('q'),
        R => Key::Layout('r'),
        S => Key::Layout('s'),
        T => Key::Layout('t'),
        U => Key::Layout('u'),
        V => Key::Layout('v'),
        W => Key::Layout('w'),
        X => Key::Layout('x'),
        Y => Key::Layout('y'),
        Z => Key::Layout('z'),
        F1 => Key::F1,
        F2 => Key::F2,
        F3 => Key::F3,
        F4 => Key::F4,
        F5 => Key::F5,
        F6 => Key::F6,
        F7 => Key::F7,
        F8 => Key::F8,
        F9 => Key::F9,
        F10 => Key::F10,
        F11 => Key::F11,
        F12 => Key::F12,
        Escape => Key::Escape,
        LControl => Key::LControl,
        RControl => Key::RControl,
        LShift => Key::LShift,
        RShift => Key::RShift,
        LAlt => Key::Raw(18),
        RAlt => Key::Raw(18),
        Meta => Key::Meta,
        Enter => Key::Return,
        Up => Key::UpArrow,
        Down => Key::DownArrow,
        Left => Key::LeftArrow,
        Right => Key::RightArrow,
        Backspace => Key::Backspace,
        CapsLock => Key::CapsLock,
        Tab => Key::Tab,
        Home => Key::Home,
        End => Key::End,
        PageUp => Key::PageUp,
        PageDown => Key::PageDown,
        Insert => Key::Insert,
        Delete => Key::Delete,
        Numpad0 => Key::Numpad0,
        Numpad1 => Key::Numpad1,
        Numpad2 => Key::Numpad2,
        Numpad3 => Key::Numpad3,
        Numpad4 => Key::Numpad4,
        Numpad5 => Key::Numpad5,
        Numpad6 => Key::Numpad6,
        Numpad7 => Key::Numpad7,
        Numpad8 => Key::Numpad8,
        Numpad9 => Key::Numpad9,
        NumpadSubtract => Key::Raw(109),
        NumpadAdd => Key::Raw(107),
        NumpadDivide => Key::Raw(111),
        NumpadMultiply => Key::Raw(106),
        Grave => Key::Raw(192),
        Minus => Key::Raw(189),
        Equal => Key::Raw(61),
        LeftBracket => Key::Raw(219),
        RightBracket => Key::Raw(221),
        BackSlash => Key::Raw(220),
        Semicolon => Key::Raw(186),
        Apostrophe => Key::Raw(48),
        Comma => Key::Raw(188),
        Dot => Key::Raw(190),
        Slash => Key::Raw(191),
        _ => Key::Layout('0'),
    }
}
impl Binding {
    pub fn str_keycode(s: &str) -> Keycode {
        match KEYCODEMAP.get(s) {
            Some(k) => *k,
            None => Key0,
        }
    }
    pub fn keycode_str(k: Keycode) -> &'static str {
        match KEYCODEMAP
            .entries()
            .find_map(|(st, ke)| if *ke == k { Some(st) } else { None })
        {
            Some(key) => *key,
            None => "Key0",
        }
    }
    pub fn from_string(input: String) -> Binding {
        let mut v: Vec<&str> = input.split(',').collect();
        let mut o = Vec::<device_query::keymap::Keycode>::new();
        let l = *v.get(0).expect("failed to get first");
        let code = v
            .get(1)
            .expect("no second position")
            .parse::<u8>()
            .expect("failed to parse to number");
        v.remove(0);
        v.remove(0);
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
    pub fn to_save(&self) -> String {
        let mut o = String::new();
        o.push_str((self.label.to_string() + ",").as_str());
        o.push_str((self.note.to_string() + ",").as_str());
        for b in &self.keys {
            o.push_str((Self::keycode_str(*b).to_owned() + ",").as_str());
        }

        o.pop();
        o
    }
}
impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("a").show(&ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                // ui.text_edit_singleline(&mut self.savefilename);
                // ui.heading("Savefile Name:")
                if ui.button("save").clicked() {
                    self.save_to_file("cfg.txt".to_string());
                }
                if ui.button("add").clicked() {
                    let mut k = self.dev_state.get_keys();
                    k.reverse();
                    self.binds.lock().unwrap().push(Binding {
                        note: 0,
                        keys: k,
                        label: "label".to_owned(),
                        selected: false,
                    });
                }
            });
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                // ui.text_edit_singleline(&mut self.savefilename);
                // ui.heading("Savefile Name:")
                ui.heading("MIDI2KB")
            });
        });
        egui::CentralPanel::default().show(&ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
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
            })
        });
    }
}
