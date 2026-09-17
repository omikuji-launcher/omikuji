// keyboard/controller navigation checks from Nav.js. that thing is brain damage beyond human comprehension.
import QtQuick
import QtTest
import omikuji 1.0
import "../../qml/components/lib/Nav.js" as Nav

TestCase {
    id: testCase
    name: "FocusTrap"
    width: 600
    height: 400
    when: windowShown

    component Box: Rectangle {
        property int activations: 0
        readonly property bool navigable: true
        function navActivate() { activations++ }
        width: 80
        height: 40
    }

    Component {
        id: trapScene

        Item {
            id: scope
            width: 600
            height: 400

            Keys.onPressed: (event) => trap.handleKey(event)

            FocusTrap {
                id: trap
                objectName: "trap"
                host: null
                scope: scope
                active: true
                sections: [top, content]
            }

            Column {
                id: top

                Rectangle {
                    objectName: "field"
                    width: 200
                    height: 40
                    readonly property bool navigable: true
                    readonly property Item navFocusTarget: input

                    TextInput {
                        id: input
                        objectName: "input"
                        anchors.fill: parent
                    }
                }

                Box { objectName: "button" }
            }

            Loader {
                id: content
                y: 200
                sourceComponent: Row {
                    Box { objectName: "sibling" }

                    Box {
                        id: badge
                        objectName: "badge"
                        function navFallback() { return Nav.collect(badge.parent)[0] || null }
                        function navActivate() { enabled = false }
                    }

                    Box {
                        objectName: "remove"
                        function navActivate() { visible = false }
                    }
                }
            }
        }
    }

    function scene() {
        const root = createTemporaryObject(trapScene, testCase.parent)
        waitForRendering(root)
        return root
    }

    function focusedName(root) {
        const item = root.Window.window.activeFocusItem
        return item ? item.objectName : ""
    }

    function test_tab_forwards_into_the_inner_input() {
        const root = scene()
        root.forceActiveFocus()
        keyClick(Qt.Key_Tab)
        compare(focusedName(root), "input")
        compare(findChild(root, "trap")._lastInside, findChild(root, "field"))
    }

    function test_enter_calls_navActivate_once() {
        const root = scene()
        const button = findChild(root, "button")
        button.forceActiveFocus()
        keyClick(Qt.Key_Return)
        compare(button.activations, 1)
    }

    function test_disabled_control_inside_a_loader_uses_its_fallback() {
        const root = scene()
        findChild(root, "badge").forceActiveFocus()
        keyClick(Qt.Key_Return)
        tryVerify(() => focusedName(root) === "sibling")
    }

    function test_hidden_control_parks_focus_in_place() {
        const root = scene()
        const remove = findChild(root, "remove")
        remove.forceActiveFocus()
        keyClick(Qt.Key_Return)
        tryVerify(() => root.Window.window.activeFocusItem === root)
        keyClick(Qt.Key_Left)
        tryVerify(() => focusedName(root) === "badge")
    }
}
