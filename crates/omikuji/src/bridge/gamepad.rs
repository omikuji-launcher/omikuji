use std::pin::Pin;
use std::thread;
use std::time::{Duration, Instant};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use gilrs::{Axis, Button, EventType, Gilrs};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, controller_kind, cxx_name = "controllerKind")]
        #[qproperty(bool, synthesize_keys, cxx_name = "synthesizeKeys")]
        type GamepadBridge = super::GamepadBridgeRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn button_pressed(self: Pin<&mut GamepadBridge>, name: &QString);

        #[qsignal]
        fn button_released(self: Pin<&mut GamepadBridge>, name: &QString);

        #[qsignal]
        fn gamepad_connected(self: Pin<&mut GamepadBridge>);

        #[qsignal]
        fn gamepad_disconnected(self: Pin<&mut GamepadBridge>);

        #[qinvokable]
        fn start(self: Pin<&mut GamepadBridge>);
    }

    impl cxx_qt::Threading for GamepadBridge {}
}

unsafe extern "C" {
    fn omikuji_inject_key(key: i32, modifiers: i32, pressed: bool, auto_repeat: bool);
}

#[derive(Default)]
pub struct GamepadBridgeRust {
    started: bool,
    controller_kind: QString,
    synthesize_keys: bool,
}

mod qt_key {
    pub const ESCAPE: i32 = 0x0100_0000;
    pub const TAB: i32 = 0x0100_0001;
    pub const BACKTAB: i32 = 0x0100_0002;
    pub const RETURN: i32 = 0x0100_0004;
    pub const E: i32 = 0x45;
    pub const F: i32 = 0x46;
    pub const Q: i32 = 0x51;
    pub const LEFT: i32 = 0x0100_0012;
    pub const UP: i32 = 0x0100_0013;
    pub const RIGHT: i32 = 0x0100_0014;
    pub const DOWN: i32 = 0x0100_0015;
    pub const SHIFT_MODIFIER: i32 = 0x0200_0000;
    pub const CONTROL_MODIFIER: i32 = 0x0400_0000;
}

fn key_for(name: &str) -> Option<(i32, i32)> {
    match name {
        "dpad_up" => Some((qt_key::UP, 0)),
        "dpad_down" => Some((qt_key::DOWN, 0)),
        "dpad_left" => Some((qt_key::LEFT, 0)),
        "dpad_right" => Some((qt_key::RIGHT, 0)),
        "south" => Some((qt_key::RETURN, 0)),
        "east" => Some((qt_key::ESCAPE, 0)),
        "start" => Some((qt_key::E, 0)),
        "north" => Some((qt_key::F, 0)),
        "west" => Some((qt_key::Q, 0)),
        "rb" => Some((qt_key::TAB, qt_key::CONTROL_MODIFIER)),
        "lb" => Some((
            qt_key::TAB,
            qt_key::CONTROL_MODIFIER | qt_key::SHIFT_MODIFIER,
        )),
        "rt" => Some((qt_key::TAB, 0)),
        "lt" => Some((qt_key::BACKTAB, qt_key::SHIFT_MODIFIER)),
        _ => None,
    }
}

fn inject_tap(name: &str, auto_repeat: bool) {
    if let Some((key, mods)) = key_for(name) {
        unsafe {
            omikuji_inject_key(key, mods, true, auto_repeat);
            omikuji_inject_key(key, mods, false, auto_repeat);
        }
    }
}

fn is_direction(name: &str) -> bool {
    matches!(name, "dpad_up" | "dpad_down" | "dpad_left" | "dpad_right")
}

const REPEAT_DELAY: Duration = Duration::from_millis(300);
const REPEAT_INTERVAL: Duration = Duration::from_millis(140);

struct Repeat {
    name: &'static str,
    from_stick: bool,
    since: Instant,
    last: Instant,
}

impl Repeat {
    fn new(name: &'static str, from_stick: bool) -> Self {
        let now = Instant::now();
        Self {
            name,
            from_stick,
            since: now,
            last: now,
        }
    }

    fn due(&mut self) -> bool {
        let due = self.since.elapsed() >= REPEAT_DELAY && self.last.elapsed() >= REPEAT_INTERVAL;
        if due {
            self.last = Instant::now();
        }
        due
    }
}

fn button_name(button: Button) -> Option<&'static str> {
    match button {
        Button::South => Some("south"),
        Button::East => Some("east"),
        Button::West => Some("west"),
        Button::North => Some("north"),
        Button::DPadLeft => Some("dpad_left"),
        Button::DPadRight => Some("dpad_right"),
        Button::DPadUp => Some("dpad_up"),
        Button::DPadDown => Some("dpad_down"),
        Button::LeftTrigger => Some("lb"),
        Button::RightTrigger => Some("rb"),
        Button::LeftTrigger2 => Some("lt"),
        Button::RightTrigger2 => Some("rt"),
        Button::Start => Some("start"),
        Button::Select => Some("select"),
        Button::Mode => Some("mode"),
        Button::LeftThumb => Some("left_thumb"),
        Button::RightThumb => Some("right_thumb"),
        _ => None,
    }
}

fn classify(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if n.contains("xbox") || n.contains("xinput") || n.contains("xb360") || n.contains("x-box") {
        "xbox"
    } else if n.contains("dualsense")
        || n.contains("dualshock")
        || n.contains("playstation")
        || n.contains("sony")
        || n.contains("ps3")
        || n.contains("ps4")
        || n.contains("ps5")
    {
        "ps"
    } else if n.contains("nintendo")
        || n.contains("pro controller")
        || n.contains("joy-con")
        || n.contains("joycon")
        || n.contains("switch")
        || n.contains("8bitdo")
    {
        "nintendo"
    } else if n.contains("steam") {
        "steam"
    } else {
        "xbox"
    }
}

fn detect_kind(gilrs: &Gilrs) -> &'static str {
    gilrs
        .gamepads()
        .next()
        .map_or("xbox", |(_, gp)| classify(gp.name()))
}

fn stick_dir(x: f32, y: f32, deadzone: f32) -> Option<&'static str> {
    if x.abs() < deadzone && y.abs() < deadzone {
        return None;
    }
    if x.abs() > y.abs() {
        Some(if x > 0.0 { "dpad_right" } else { "dpad_left" })
    } else {
        Some(if y > 0.0 { "dpad_up" } else { "dpad_down" })
    }
}

impl qobject::GamepadBridge {
    fn start(mut self: Pin<&mut Self>) {
        if self.started {
            return;
        }
        self.as_mut().rust_mut().get_mut().started = true;

        let synthesize = self.synthesize_keys;
        let qt_thread = self.as_mut().qt_thread();
        thread::spawn(move || {
            let mut gilrs = match Gilrs::new() {
                Ok(g) => g,
                Err(e) => {
                    tracing::error!("failed to init gilrs: {:?}", e);
                    return;
                }
            };

            for (id, gp) in gilrs.gamepads() {
                tracing::info!("connected at start: id={:?} name='{}'", id, gp.name());
            }

            let initial_kind = detect_kind(&gilrs);
            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GamepadBridge>| {
                obj.as_mut()
                    .set_controller_kind(QString::from(initial_kind));
            });

            let stick_deadzone = 0.5_f32;
            let mut stick_held: Option<&'static str> = None;
            let mut repeat: Option<Repeat> = None;

            let press = |name: &'static str, auto_repeat: bool| {
                if synthesize {
                    inject_tap(name, auto_repeat);
                }
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GamepadBridge>| {
                    obj.as_mut().button_pressed(&QString::from(name));
                });
            };

            loop {
                while let Some(event) = gilrs.next_event() {
                    match event.event {
                        EventType::ButtonPressed(button, _) => {
                            if let Some(name) = button_name(button) {
                                press(name, false);
                                if is_direction(name) {
                                    repeat = Some(Repeat::new(name, false));
                                }
                            }
                        }
                        EventType::ButtonReleased(button, _) => {
                            if let Some(name) = button_name(button) {
                                if repeat
                                    .as_ref()
                                    .is_some_and(|r| !r.from_stick && r.name == name)
                                {
                                    repeat = None;
                                }
                                let _ = qt_thread.queue(
                                    move |mut obj: Pin<&mut qobject::GamepadBridge>| {
                                        obj.as_mut().button_released(&QString::from(name));
                                    },
                                );
                            }
                        }
                        EventType::AxisChanged(axis, _, _)
                            if (axis == Axis::LeftStickX || axis == Axis::LeftStickY) =>
                        {
                            let pad = gilrs.gamepad(event.id);
                            let lx = pad.value(Axis::LeftStickX);
                            let ly = pad.value(Axis::LeftStickY);
                            let new_dir = stick_dir(lx, ly, stick_deadzone);
                            if new_dir != stick_held {
                                stick_held = new_dir;
                                match new_dir {
                                    Some(dir) => {
                                        press(dir, false);
                                        repeat = Some(Repeat::new(dir, true));
                                    }
                                    None if repeat.as_ref().is_some_and(|r| r.from_stick) => {
                                        repeat = None;
                                    }
                                    None => {}
                                }
                            }
                        }
                        EventType::Connected => {
                            let kind = classify(gilrs.gamepad(event.id).name());
                            let _ = qt_thread.queue(
                                move |mut obj: Pin<&mut qobject::GamepadBridge>| {
                                    obj.as_mut().set_controller_kind(QString::from(kind));
                                    obj.as_mut().gamepad_connected();
                                },
                            );
                        }
                        EventType::Disconnected => {
                            let _ = qt_thread.queue(|mut obj: Pin<&mut qobject::GamepadBridge>| {
                                obj.as_mut().gamepad_disconnected();
                            });
                        }
                        _ => {}
                    }
                }

                if let Some(r) = &mut repeat
                    && r.due()
                {
                    press(r.name, true);
                }

                thread::sleep(Duration::from_millis(8));
            }
        });
    }
}
