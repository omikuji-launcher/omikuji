import QtQuick
import QtQuick.Dialogs

QtObject {
    id: root

    property bool selectFolder: false
    property string title: ""
    property string filter: ""
    property string startFolder: ""

    signal picked(string path)

    function open() {
        let dialog = root.selectFolder ? root._folderDialog : root._fileDialog
        if (root.startFolder !== "") {
            dialog.currentFolder = "file://" + root.startFolder
        }
        dialog.open()
    }

    function _toPath(url) {
        let s = url.toString()
        return decodeURIComponent(s.startsWith("file://") ? s.substring(7) : s)
    }

    property FolderDialog _folderDialog: FolderDialog {
        title: root.title
        onAccepted: root.picked(root._toPath(selectedFolder))
    }

    property FileDialog _fileDialog: FileDialog {
        title: root.title
        nameFilters: root.filter === ""
            ? [qsTr("All files") + " (*)"]
            : [qsTr("Supported files") + " (" + root.filter + ")", qsTr("All files") + " (*)"]
        onAccepted: root.picked(root._toPath(selectedFile))
    }
}
