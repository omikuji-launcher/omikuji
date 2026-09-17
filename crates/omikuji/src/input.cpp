#include <QtCore/QEvent>
#include <QtCore/QObject>
#include <QtCore/QPointF>
#include <QtCore/QString>
#include <QtGui/QGuiApplication>
#include <QtGui/QKeyEvent>
#include <QtGui/QMouseEvent>
#include <QtGui/QWindow>
#include <qpa/qwindowsysteminterface.h>

namespace {

using ModeCallback = void (*)(void*, bool);

const quint32 kInjectedMarker = 0x4f4d494b;

bool isNavigationKey(int key) {
    switch (key) {
    case Qt::Key_Up:
    case Qt::Key_Down:
    case Qt::Key_Left:
    case Qt::Key_Right:
    case Qt::Key_Tab:
    case Qt::Key_Backtab:
        return true;
    default:
        return false;
    }
}

class InputModeFilter : public QObject {
public:
    InputModeFilter(void* ctx, ModeCallback callback, QObject* parent)
        : QObject(parent), m_ctx(ctx), m_callback(callback) {}

    bool eventFilter(QObject*, QEvent* event) override {
        switch (event->type()) {
        case QEvent::KeyPress: {
            const auto* key = static_cast<QKeyEvent*>(event);
            if (key->nativeModifiers() == kInjectedMarker || isNavigationKey(key->key())) {
                setKeyboard(true);
            }
            break;
        }
        case QEvent::MouseMove: {
            const QPointF pos = static_cast<QMouseEvent*>(event)->globalPosition();
            if (pos != m_lastPointer) {
                m_lastPointer = pos;
                setKeyboard(false);
            }
            break;
        }
        case QEvent::MouseButtonPress:
        case QEvent::Wheel:
            setKeyboard(false);
            break;
        default:
            break;
        }
        return false;
    }

private:
    void setKeyboard(bool keyboard) {
        if (keyboard == m_keyboard) {
            return;
        }
        m_keyboard = keyboard;
        if (keyboard) {
            QGuiApplication::setOverrideCursor(Qt::BlankCursor);
            for (QWindow* window : QGuiApplication::topLevelWindows()) {
                QWindowSystemInterface::handleLeaveEvent(window);
            }
        } else {
            QGuiApplication::restoreOverrideCursor();
        }
        m_callback(m_ctx, keyboard);
    }

    void* m_ctx;
    ModeCallback m_callback;
    bool m_keyboard = false;
    QPointF m_lastPointer;
};

} // namespace

extern "C" void omikuji_inject_key(int key, int modifiers, bool pressed, bool auto_repeat) {
    QWindowSystemInterface::handleExtendedKeyEvent(nullptr,
                                                   pressed ? QEvent::KeyPress : QEvent::KeyRelease,
                                                   key,
                                                   Qt::KeyboardModifiers(modifiers),
                                                   0,
                                                   0,
                                                   kInjectedMarker,
                                                   QString(),
                                                   auto_repeat);
}

extern "C" void omikuji_watch_input_mode(void* ctx, ModeCallback callback) {
    static InputModeFilter* filter = nullptr;
    if (filter || !qApp) {
        return;
    }
    filter = new InputModeFilter(ctx, callback, qApp);
    qApp->installEventFilter(filter);
}
