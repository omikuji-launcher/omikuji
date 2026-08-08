#include <QQmlApplicationEngine>
#include <QFileSystemWatcher>
#include <QDirIterator>
#include <QTimer>
#include <QUrl>
#include <QString>
#include <QStringList>
#include <QDir>
#include <QObject>

extern "C" void omikuji_start_qml_watcher(void* enginePtr, const char* qmlDir, const char* rootUrl) {
    auto* engine = reinterpret_cast<QQmlApplicationEngine*>(enginePtr);
    const QString dir = QString::fromUtf8(qmlDir);
    const QUrl url = QUrl(QString::fromUtf8(rootUrl));

    auto scan = [dir]() {
        QStringList paths;
        paths << dir;
        QDirIterator dirs(dir, QDir::Dirs | QDir::NoDotAndDotDot, QDirIterator::Subdirectories);
        while (dirs.hasNext()) paths << dirs.next();
        QDirIterator files(dir, QStringList{QStringLiteral("*.qml")}, QDir::Files, QDirIterator::Subdirectories);
        while (files.hasNext()) paths << files.next();
        return paths;
    };

    auto* watcher = new QFileSystemWatcher(engine);
    watcher->addPaths(scan());

    auto* debounce = new QTimer(engine);
    debounce->setSingleShot(true);
    debounce->setInterval(120);
    QObject::connect(debounce, &QTimer::timeout, engine, [engine, url]() {
        const auto oldRoots = engine->rootObjects();
        engine->clearComponentCache();
        engine->load(url);
        if (engine->rootObjects().size() > oldRoots.size()) {
            for (QObject* o : oldRoots) o->deleteLater();
        }
    });

    auto onChange = [watcher, scan, debounce](const QString&) {
        const QStringList current = watcher->files() + watcher->directories();
        for (const QString& p : scan()) {
            if (!current.contains(p)) watcher->addPath(p);
        }
        debounce->start();
    };
    QObject::connect(watcher, &QFileSystemWatcher::fileChanged, engine, onChange);
    QObject::connect(watcher, &QFileSystemWatcher::directoryChanged, engine, onChange);
}
