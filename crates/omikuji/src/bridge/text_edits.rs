use std::ffi::c_void;
use std::pin::Pin;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtGui/QTextDocument>);
        type QTextDocument;

        #[cxx_name = "setUndoRedoEnabled"]
        fn set_undo_redo_enabled(self: Pin<&mut QTextDocument>, enable: bool);
    }

    unsafe extern "C++Qt" {
        include!(<QtQuick/QQuickTextDocument>);
        #[qobject]
        type QQuickTextDocument;

        #[cxx_name = "textDocument"]
        fn text_document(self: &QQuickTextDocument) -> *mut QTextDocument;

        include!(<QtQuick/QQuickItem>);
        #[qobject]
        type QQuickItem;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type TextEdits = super::TextEditsRust;
    }

    unsafe extern "RustQt" {
        #[cxx_name = "setUndoRedoEnabled"]
        #[qinvokable]
        unsafe fn set_undo_redo_enabled(
            self: &TextEdits,
            doc: *mut QQuickTextDocument,
            enabled: bool,
        );

        #[cxx_name = "setObservesViewport"]
        #[qinvokable]
        unsafe fn set_observes_viewport(self: &TextEdits, item: *mut QQuickItem, enabled: bool);
    }
}

unsafe extern "C" {
    fn omikuji_set_observes_viewport(item: *mut c_void, enabled: bool);
}

#[derive(Default)]
pub struct TextEditsRust;

impl qobject::TextEdits {
    fn set_undo_redo_enabled(&self, doc: *mut qobject::QQuickTextDocument, enabled: bool) {
        let Some(doc) = (unsafe { doc.as_ref() }) else {
            return;
        };
        let Some(inner) = (unsafe { doc.text_document().as_mut() }) else {
            return;
        };
        unsafe { Pin::new_unchecked(inner) }.set_undo_redo_enabled(enabled);
    }

    fn set_observes_viewport(&self, item: *mut qobject::QQuickItem, enabled: bool) {
        unsafe { omikuji_set_observes_viewport(item.cast(), enabled) };
    }
}
