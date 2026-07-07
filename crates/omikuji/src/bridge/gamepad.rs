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

#[derive(Default)]
pub struct GamepadBridgeRust {
    started: bool,
    controller_kind: QString,
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
                obj.as_mut().set_controller_kind(QString::from(initial_kind));
            });

            let mut stick_held: Option<&'static str> = None;
            let mut stick_last_emit = Instant::now();
            let stick_repeat = Duration::from_millis(150);
            let stick_deadzone = 0.5_f32;

            let emit_button_press = |name: &'static str| {
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GamepadBridge>| {
                    obj.as_mut().button_pressed(&QString::from(name));
                });
            };

            loop {
                while let Some(event) = gilrs.next_event() {
                    match event.event {
                        EventType::ButtonPressed(button, _) => {
                            if let Some(name) = button_name(button) {
                                emit_button_press(name);
                            }
                        }
                        EventType::ButtonReleased(button, _) => {
                            if let Some(name) = button_name(button) {
                                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GamepadBridge>| {
                                    obj.as_mut().button_released(&QString::from(name));
                                });
                            }
                        }
                        EventType::AxisChanged(axis, _, _)
                            if (axis == Axis::LeftStickX || axis == Axis::LeftStickY) => {
                                let pad = gilrs.gamepad(event.id);
                                let lx = pad.value(Axis::LeftStickX);
                                let ly = pad.value(Axis::LeftStickY);
                                let new_dir = stick_dir(lx, ly, stick_deadzone);
                                if new_dir != stick_held {
                                    stick_held = new_dir;
                                    if let Some(dir) = new_dir {
                                        emit_button_press(dir);
                                        stick_last_emit = Instant::now();
                                    }
                                }
                            }
                        EventType::Connected => {
                            let kind = classify(gilrs.gamepad(event.id).name());
                            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GamepadBridge>| {
                                obj.as_mut().set_controller_kind(QString::from(kind));
                                obj.as_mut().gamepad_connected();
                            });
                        }
                        EventType::Disconnected => {
                            let _ = qt_thread.queue(|mut obj: Pin<&mut qobject::GamepadBridge>| {
                                obj.as_mut().gamepad_disconnected();
                            });
                        }
                        _ => {}
                    }
                }

                if let Some(dir) = stick_held
                    && stick_last_emit.elapsed() >= stick_repeat
                {
                    emit_button_press(dir);
                    stick_last_emit = Instant::now();
                }

                thread::sleep(Duration::from_millis(8));
            }
        });
    }
}
