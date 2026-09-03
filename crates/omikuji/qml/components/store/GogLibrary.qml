import QtQuick

StoreLibraryBase {
    id: root

    iconName: "gog"
    loginTitle: qsTr("Login to GOG")
    loginDescription: qsTr("To sync your GOG library, sign in on gog.com and paste the authorization code from the redirect URL.")
    loginUrl: root.storeModel ? root.storeModel.get_login_url() : ""
    toolName: "gogdl"
}
