#include <QtCore/QCoreApplication>
#include <QtCore/QDir>
#include <QtCore/QFileInfo>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <QtCore/QLocale>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtCore/QTranslator>
#include <string>

namespace {
QTranslator* s_translator = nullptr;
const QString k_i18n_dir = QStringLiteral(":/qt/qml/omikuji/i18n");
// the source language only ships for its plural forms, the picker already lists it by hand
const QString k_source_lang = QStringLiteral("en");
}

extern "C" void omikuji_install_translator(const char* lang) {
    QCoreApplication* app = QCoreApplication::instance();
    if (!app) return;

    if (s_translator) {
        app->removeTranslator(s_translator);
        delete s_translator;
        s_translator = nullptr;
    }

    QString code = lang ? QString::fromUtf8(lang) : QString();
    QLocale locale = (code.isEmpty() || code == QStringLiteral("system"))
        ? QLocale::system()
        : QLocale(code);

    QTranslator* t = new QTranslator(app);
    if (t->load(locale, QStringLiteral("omikuji"), QStringLiteral("_"), k_i18n_dir)) {
        app->installTranslator(t);
        s_translator = t;
    } else {
        delete t;
    }
}

extern "C" const char* omikuji_available_languages_json() {
    static thread_local std::string buf;
    QJsonArray arr;
    const QStringList files = QDir(k_i18n_dir)
        .entryList(QStringList() << QStringLiteral("omikuji_*.qm"), QDir::Files);
    for (const QString& file : files) {
        QString code = QFileInfo(file).completeBaseName();
        code.remove(0, static_cast<int>(QStringLiteral("omikuji_").size()));
        if (code.isEmpty() || code == k_source_lang) continue;
        const QString name = QLocale(code).nativeLanguageName();
        QJsonObject obj;
        obj[QStringLiteral("code")] = code;
        obj[QStringLiteral("name")] = name.isEmpty() ? code : name;
        arr.append(obj);
    }
    buf = QJsonDocument(arr).toJson(QJsonDocument::Compact).toStdString();
    return buf.c_str();
}
