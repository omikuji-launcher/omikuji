#include <QtCore/QByteArray>
#include <QtCore/QString>
#include <QtCore/QtGlobal>

extern "C" void omikuji_qt_log_message(int level, const char* category, const char* message);

namespace {

QtMessageHandler s_previous = nullptr;

void forward(QtMsgType type, const QMessageLogContext& context, const QString& message) {
    const QByteArray utf8 = message.toUtf8();
    omikuji_qt_log_message(type, context.category ? context.category : "default", utf8.constData());
    if (s_previous) {
        s_previous(type, context, message);
    }
}

}

extern "C" void omikuji_install_qt_log() {
    s_previous = qInstallMessageHandler(forward);
}
