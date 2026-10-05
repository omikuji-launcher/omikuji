#include <QtQuick/QQuickItem>

// apparently cxx cant name enums nested in classes afaik so this
extern "C" void omikuji_set_observes_viewport(QQuickItem* item, bool enabled) {
    if (item) {
        item->setFlag(QQuickItem::ItemObservesViewport, enabled);
    }
}
