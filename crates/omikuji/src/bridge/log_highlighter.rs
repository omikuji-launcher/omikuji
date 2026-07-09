#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtGui/QSyntaxHighlighter>);
        type QSyntaxHighlighter;

        include!(<QtGui/QTextDocument>);
        type QTextDocument;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
    }

    unsafe extern "C++Qt" {
        include!(<QtQuick/QQuickTextDocument>);
        #[qobject]
        type QQuickTextDocument;

        #[cxx_name = "textDocument"]
        fn text_document(self: &QQuickTextDocument) -> *mut QTextDocument;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QSyntaxHighlighter]
        #[qproperty(bool, active)]
        #[qproperty(QColor, error_color, cxx_name = "errorColor")]
        #[qproperty(QColor, warn_color, cxx_name = "warnColor")]
        #[qproperty(QColor, fixme_color, cxx_name = "fixmeColor")]
        #[qproperty(QColor, trace_color, cxx_name = "traceColor")]
        type LogHighlighter = super::LogHighlighterRust;
    }

    unsafe extern "RustQt" {
        #[cxx_name = "highlightBlock"]
        #[cxx_override]
        fn highlight_block(self: Pin<&mut LogHighlighter>, text: &QString);

        #[qinvokable]
        unsafe fn attach(self: Pin<&mut LogHighlighter>, doc: *mut QQuickTextDocument);

        #[qinvokable]
        fn refresh(self: Pin<&mut LogHighlighter>);

        #[cxx_name = "setRules"]
        #[qinvokable]
        fn set_rules(self: Pin<&mut LogHighlighter>, json: &QString);
    }

    unsafe extern "RustQt" {
        #[cxx_name = "setFormat"]
        #[inherit]
        fn set_format(self: Pin<&mut LogHighlighter>, start: i32, count: i32, color: &QColor);

        #[inherit]
        fn rehighlight(self: Pin<&mut LogHighlighter>);

        #[cxx_name = "setDocument"]
        #[inherit]
        unsafe fn set_document(self: Pin<&mut LogHighlighter>, doc: *mut QTextDocument);
    }
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QColor;
use omikuji_core::ui_settings::LogRule;
use regex::Regex;

#[derive(Default)]
pub struct LogHighlighterRust {
    pub active: bool,
    pub error_color: QColor,
    pub warn_color: QColor,
    pub fixme_color: QColor,
    pub trace_color: QColor,
    rules: Vec<(Regex, QColor)>,
}

fn parse_hex_color(s: &str) -> Option<QColor> {
    let hex = s.strip_prefix('#')?;
    let v = u32::from_str_radix(hex, 16).ok()?;
    let (r, g, b) = (((v >> 16) & 0xff) as i32, ((v >> 8) & 0xff) as i32, (v & 0xff) as i32);
    match hex.len() {
        6 => Some(QColor::from_rgba(r, g, b, 255)),
        8 => Some(QColor::from_rgba(r, g, b, ((v >> 24) & 0xff) as i32)),
        _ => None,
    }
}

fn wine_channel(line: &str) -> &str {
    let t = line.trim_start();
    match t.split_once(':') {
        Some((id, rest)) if !id.is_empty() && id.len() <= 8 && id.bytes().all(|b| b.is_ascii_hexdigit()) => rest,
        _ => t,
    }
}

impl qobject::LogHighlighter {
    fn highlight_block(mut self: Pin<&mut Self>, text: &cxx_qt_lib::QString) {
        if !self.active {
            return;
        }
        let line = text.to_string();
        let color = if let Some((_, c)) = self.rules.iter().find(|(re, _)| re.is_match(&line)) {
            c.clone()
        } else {
            let head = wine_channel(&line);
            let lower = line.to_lowercase();
            if head.starts_with("err:") || lower.contains("error") {
                self.error_color.clone()
            } else if head.starts_with("fixme:") {
                self.fixme_color.clone()
            } else if head.starts_with("warn:") || lower.contains("warning") {
                self.warn_color.clone()
            } else if head.starts_with("trace:") {
                self.trace_color.clone()
            } else {
                return;
            }
        };
        let count = line.encode_utf16().count() as i32;
        self.as_mut().set_format(0, count, &color);
    }

    fn set_rules(mut self: Pin<&mut Self>, json: &cxx_qt_lib::QString) {
        let raw: Vec<LogRule> = serde_json::from_str(&json.to_string()).unwrap_or_default();
        let compiled = raw
            .iter()
            .filter(|r| !r.pattern.is_empty())
            .filter_map(|r| Some((Regex::new(&r.pattern).ok()?, parse_hex_color(&r.color)?)))
            .collect();
        self.as_mut().rust_mut().get_mut().rules = compiled;
        self.as_mut().rehighlight();
    }

    fn attach(mut self: Pin<&mut Self>, doc: *mut qobject::QQuickTextDocument) {
        let Some(doc) = (unsafe { doc.as_ref() }) else { return };
        let inner = doc.text_document();
        unsafe { self.as_mut().set_document(inner) };
    }

    fn refresh(self: Pin<&mut Self>) {
        self.rehighlight();
    }
}
