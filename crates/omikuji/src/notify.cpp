#include <QtCore/QCoreApplication>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtCore/QVariantMap>
#include <QtDBus/QDBusConnection>
#include <QtDBus/QDBusMessage>

extern "C" void omikuji_notify(const char* title, const char* body) {
    QDBusMessage msg = QDBusMessage::createMethodCall(
        QStringLiteral("org.freedesktop.Notifications"),
        QStringLiteral("/org/freedesktop/Notifications"),
        QStringLiteral("org.freedesktop.Notifications"),
        QStringLiteral("Notify"));

    msg << QCoreApplication::applicationName()
        << static_cast<unsigned int>(0)
        << QStringLiteral("io.github.reakjra.omikuji")
        << QString::fromUtf8(title)
        << QString::fromUtf8(body)
        << QStringList()
        << QVariantMap()
        << -1;

    QDBusConnection::sessionBus().send(msg);
}
