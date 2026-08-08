import QtQuick
import omikuji 1.0

Text {
    property real size: 10

    color: Theme.textSubtle
    font.pixelSize: size
    font.weight: Font.DemiBold
    font.capitalization: Font.AllUppercase
    font.letterSpacing: 1
    textFormat: Text.PlainText
}
