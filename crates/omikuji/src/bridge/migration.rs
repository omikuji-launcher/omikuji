use cxx_qt::Threading;
use cxx_qt_lib::QString;
use std::pin::Pin;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, running)]
        type MigrationBridge = super::MigrationRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn output(self: Pin<&mut MigrationBridge>, line: QString);

        #[qsignal]
        fn finished(self: Pin<&mut MigrationBridge>, ok: bool, error: QString);

        #[qinvokable]
        fn pending(self: &MigrationBridge) -> bool;

        #[qinvokable]
        fn run(self: Pin<&mut MigrationBridge>);

        #[qinvokable]
        #[cxx_name = "restartApp"]
        fn restart_app(self: &MigrationBridge);
    }

    impl cxx_qt::Threading for MigrationBridge {}
}

#[derive(Default)]
pub struct MigrationRust {
    running: bool,
}

impl qobject::MigrationBridge {
    fn pending(&self) -> bool {
        omikuji_core::migration::pending()
    }

    fn run(mut self: Pin<&mut Self>) {
        if self.running {
            return;
        }
        self.as_mut().set_running(true);
        let qt = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let line_qt = qt.clone();
            let res = omikuji_core::migration::run(move |line| {
                let _ = line_qt.queue(move |mut obj: Pin<&mut qobject::MigrationBridge>| {
                    obj.as_mut().output(QString::from(&line));
                });
            });
            let (ok, err) = match res {
                Ok(_) => (true, String::new()),
                Err(e) => (false, format!("{:#}", e)),
            };
            let _ = qt.queue(move |mut obj: Pin<&mut qobject::MigrationBridge>| {
                obj.as_mut().set_running(false);
                obj.as_mut().finished(ok, QString::from(&err));
            });
        });
    }

    fn restart_app(&self) {
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe).spawn();
        }
        unsafe { libc::_exit(0) };
    }
}
