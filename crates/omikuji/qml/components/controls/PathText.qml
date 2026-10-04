import QtQuick
import omikuji 1.0

// usable only when the text is actually accessible. so not on items that are already clickable
TextEdit {
    readOnly: true
    selectByMouse: true
    color: Theme.accent
    selectionColor: Theme.alpha(Theme.accent, 0.35)
    selectedTextColor: Theme.text
    font.family: Theme.mono
    font.pixelSize: Theme.type.caption.size
    wrapMode: TextEdit.WrapAnywhere
}
