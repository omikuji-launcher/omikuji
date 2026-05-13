#include <QtCore/QCoreApplication>
#include <QtCore/QString>
#include <QtCore/QByteArray>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <QtCore/QObject>
#include <QtGui/QAction>
#include <QtGui/QIcon>
#include <QtWidgets/QApplication>
#include <QtWidgets/QMenu>
#include <QtWidgets/QSystemTrayIcon>
#include <cstddef>
#include <cstdint>

extern "C" void omikuji_tray_event_show();
extern "C" void omikuji_tray_event_quit();
extern "C" void omikuji_tray_event_toggle();
extern "C" void omikuji_tray_event_game(const char* id, std::size_t len);

namespace {

char s_arg0[] = "omikuji";
char* s_argv[] = { s_arg0, nullptr };
int s_argc = 1;
QApplication* s_app = nullptr;

QSystemTrayIcon* s_tray = nullptr;
QMenu* s_menu = nullptr;
QMenu* s_recent_menu = nullptr;
QAction* s_show_action = nullptr;
QAction* s_quit_action = nullptr;
QString s_icon_path;

void ensure_tray_created() {
    if (s_tray) return;
    s_tray = new QSystemTrayIcon(QCoreApplication::instance());
    s_menu = new QMenu();
    s_recent_menu = new QMenu(QObject::tr("Recent games"));
    QAction* placeholder = s_recent_menu->addAction(QObject::tr("No recent games"));
    placeholder->setEnabled(false);

    s_menu->addMenu(s_recent_menu);
    s_menu->addSeparator();
    s_show_action = s_menu->addAction(QObject::tr("Show Omikuji"));
    s_quit_action = s_menu->addAction(QObject::tr("Quit"));

    QObject::connect(s_show_action, &QAction::triggered, []() {
        omikuji_tray_event_show();
    });
    QObject::connect(s_quit_action, &QAction::triggered, []() {
        omikuji_tray_event_quit();
    });

    s_tray->setContextMenu(s_menu);
    s_tray->setToolTip("Omikuji");

    QObject::connect(s_tray, &QSystemTrayIcon::activated,
                     [](QSystemTrayIcon::ActivationReason reason) {
        if (reason == QSystemTrayIcon::Trigger) {
            omikuji_tray_event_toggle();
        }
    });

    if (!s_icon_path.isEmpty()) {
        s_tray->setIcon(QIcon(s_icon_path));
    } else {
        QIcon themed = QIcon::fromTheme("omikuji");
        if (!themed.isNull()) s_tray->setIcon(themed);
    }
}

}

extern "C" void omikuji_app_init() {
    if (s_app) return;
    s_app = new QApplication(s_argc, s_argv);
    QApplication::setQuitOnLastWindowClosed(true);
}

extern "C" int omikuji_app_exec() {
    return s_app ? s_app->exec() : 1;
}

extern "C" void omikuji_app_set_quit_on_last_window_closed(bool v) {
    QApplication::setQuitOnLastWindowClosed(v);
}

extern "C" void omikuji_app_quit() {
    QCoreApplication::quit();
}

extern "C" void omikuji_tray_set_icon(const char* path) {
    if (!path) return;
    s_icon_path = QString::fromUtf8(path);
    if (s_tray) s_tray->setIcon(QIcon(s_icon_path));
}

extern "C" void omikuji_tray_set_enabled(bool enabled) {
    if (enabled) {
        if (!QSystemTrayIcon::isSystemTrayAvailable()) {
            qWarning("[tray] system tray not available on this desktop");
            return;
        }
        ensure_tray_created();
        s_tray->show();
    } else if (s_tray) {
        s_tray->hide();
    }
}

extern "C" void omikuji_tray_set_recent(const uint8_t* json_bytes, std::size_t len) {
    if (!s_recent_menu) return;
    s_recent_menu->clear();

    QJsonParseError err;
    QJsonDocument doc = QJsonDocument::fromJson(
        QByteArray(reinterpret_cast<const char*>(json_bytes), static_cast<int>(len)),
        &err);
    if (err.error != QJsonParseError::NoError || !doc.isArray()) {
        QAction* placeholder = s_recent_menu->addAction(QObject::tr("No recent games"));
        placeholder->setEnabled(false);
        return;
    }
    QJsonArray arr = doc.array();
    if (arr.isEmpty()) {
        QAction* placeholder = s_recent_menu->addAction(QObject::tr("No recent games"));
        placeholder->setEnabled(false);
        return;
    }
    for (const QJsonValue& v : arr) {
        if (!v.isObject()) continue;
        QJsonObject o = v.toObject();
        QString id = o.value("id").toString();
        QString name = o.value("name").toString();
        if (id.isEmpty() || name.isEmpty()) continue;
        QAction* a = s_recent_menu->addAction(name);
        QByteArray id_bytes = id.toUtf8();
        QObject::connect(a, &QAction::triggered, [id_bytes]() {
            omikuji_tray_event_game(id_bytes.constData(),
                                    static_cast<std::size_t>(id_bytes.size()));
        });
    }
}
