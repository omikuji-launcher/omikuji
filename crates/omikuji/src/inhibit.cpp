#include <QtCore/QCoreApplication>
#include <QtCore/QString>
#include <QtDBus/QDBusConnection>
#include <QtDBus/QDBusConnectionInterface>
#include <QtDBus/QDBusMessage>

namespace {

struct Backend {
    const char* service;
    const char* path;
    const char* iface;
};

const Backend kBackends[] = {
    {"org.freedesktop.ScreenSaver", "/org/freedesktop/ScreenSaver",
     "org.freedesktop.ScreenSaver"},
    {"org.freedesktop.PowerManagement.Inhibit", "/org/freedesktop/PowerManagement/Inhibit",
     "org.freedesktop.PowerManagement.Inhibit"},
};

// nothing owns these on a bare wm, and an unanswered call is worse than no call
const Backend* backend() {
    static const Backend* picked = []() -> const Backend* {
        QDBusConnectionInterface* bus = QDBusConnection::sessionBus().interface();
        if (!bus) {
            return nullptr;
        }
        for (const Backend& b : kBackends) {
            if (bus->isServiceRegistered(QString::fromLatin1(b.service)).value()) {
                return &b;
            }
        }
        return nullptr;
    }();
    return picked;
}

QDBusMessage message(const Backend* b, const char* method) {
    return QDBusMessage::createMethodCall(QString::fromLatin1(b->service),
                                          QString::fromLatin1(b->path),
                                          QString::fromLatin1(b->iface),
                                          QString::fromLatin1(method));
}

} // namespace

extern "C" unsigned int omikuji_inhibit(const char* reason) {
    const Backend* b = backend();
    if (!b) {
        return 0;
    }

    QDBusMessage msg = message(b, "Inhibit");
    msg << QCoreApplication::applicationName() << QString::fromUtf8(reason);

    QDBusMessage reply = QDBusConnection::sessionBus().call(msg, QDBus::Block, 1000);
    if (reply.type() != QDBusMessage::ReplyMessage || reply.arguments().isEmpty()) {
        return 0;
    }
    return reply.arguments().first().toUInt();
}

extern "C" void omikuji_uninhibit(unsigned int cookie) {
    const Backend* b = backend();
    if (!b || cookie == 0) {
        return;
    }

    QDBusMessage msg = message(b, "UnInhibit");
    msg << cookie;
    QDBusConnection::sessionBus().send(msg);
}
