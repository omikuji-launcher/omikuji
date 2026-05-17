#include <QtCore/QString>
#include <QtGui/QFont>
#include <QtGui/QGuiApplication>

static QFont g_default_font;
static bool g_default_captured = false;

extern "C" void omikuji_capture_default_font() {
    if (!g_default_captured) {
        g_default_font = QGuiApplication::font();
        g_default_captured = true;
    }
}

extern "C" void omikuji_set_app_font(const char* family) {
    if (!g_default_captured) {
        omikuji_capture_default_font();
    }
    if (!family || family[0] == '\0') {
        QGuiApplication::setFont(g_default_font);
        return;
    }
    QFont f = g_default_font;
    f.setFamily(QString::fromUtf8(family));
    QGuiApplication::setFont(f);
}
