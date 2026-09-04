import QtQuick

StoreLibraryBase {
    id: root

    iconName: "amazon"
    loginTitle: qsTr("Login to Amazon Games")
    loginDescription: qsTr("To sync your Amazon Games library, sign in on amazon.com. When the page finishes redirecting, paste the whole address bar contents here.")
    loginUrl: root.storeModel ? root.storeModel.loginUrl : ""
    toolName: "nile"
}
