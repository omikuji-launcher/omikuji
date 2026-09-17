// keyboard/controller navigation checks from Nav.js. that thing is brain damage beyond human comprehension.
import QtQuick
import QtTest
import "../../qml/components/lib/Nav.js" as Nav

TestCase {
    id: testCase
    name: "Spatial"
    width: 900
    height: 700
    when: windowShown

    component Box: Rectangle {
        readonly property bool navigable: true
        width: 40
        height: 40
    }

    component CardGrid: Flickable {
        id: grid
        property int keyIndex: 0
        property alias cards: repeater
        readonly property bool navigable: true
        clip: true
        contentHeight: flow.height + 400

        function navRectItem() { return repeater.itemAt(keyIndex) }
        function navTargets() {
            const out = []
            for (let i = 0; i < repeater.count; i++) out.push(repeater.itemAt(i))
            return out
        }
        function navEnter(card) { keyIndex = card.index }

        Flow {
            id: flow
            width: grid.width
            spacing: 10

            Repeater {
                id: repeater
                model: 8

                Rectangle {
                    required property int index
                    objectName: "card" + index
                    width: 160
                    height: 220
                }
            }
        }
    }

    Component {
        id: railScene

        Item {
            width: 900
            height: 700

            Column {
                y: 60
                spacing: 2

                Box { objectName: "cat0"; width: 160 }
                Box { objectName: "cat1"; width: 160 }
                Box { objectName: "cat2"; width: 160 }
                Item { width: 1; height: 24 }
                Box { objectName: "steam"; width: 160 }
                Box { objectName: "epic"; width: 160 }
            }

            Box { objectName: "downloads"; y: 520; width: 160 }
            Box { objectName: "search"; x: 300; y: 10; width: 300; height: 34 }
            Box { objectName: "draw"; x: 760; y: 10; height: 34 }

            CardGrid { objectName: "grid"; x: 180; y: 60; width: 700; height: 640 }
        }
    }

    Component {
        id: stackedScene

        Item {
            width: 400
            height: 700

            component Rows: Flickable {
                id: rows
                property string prefix: ""
                width: 300
                height: 200
                clip: true
                contentHeight: column.height

                Column {
                    id: column

                    Repeater {
                        model: 10

                        Box {
                            required property int index
                            objectName: rows.prefix + index
                            width: 300
                        }
                    }
                }
            }

            Flickable {
                anchors.fill: parent
                contentHeight: 700

                Rows { objectName: "fields"; prefix: "field" }
                Box { objectName: "checkAll"; y: 260; width: 300 }
                Rows { objectName: "games"; prefix: "game"; y: 310 }
            }
        }
    }

    function scene(component) {
        const root = createTemporaryObject(component, testCase.parent)
        waitForRendering(root)
        return root
    }

    function move(root, name, dx, dy) {
        const from = findChild(root, name)
        from.forceActiveFocus()
        const to = Nav.spatialMove(root, from, dx, dy)
        if (to) Nav.revealAncestors(to, root)
        return to ? to.objectName : ""
    }

    function test_rail_ignores_the_overlapping_grid() {
        const root = scene(railScene)
        compare(move(root, "cat2", 0, 1), "steam")
        compare(move(root, "epic", 0, 1), "downloads")
        compare(move(root, "downloads", 0, -1), "epic")
    }

    function test_rail_and_grid_trade_sideways() {
        const root = scene(railScene)
        compare(move(root, "cat1", 1, 0), "grid")
        compare(move(root, "grid", -1, 0), "cat2")
    }

    function test_entering_the_grid_picks_the_card_under_the_line() {
        const root = scene(railScene)
        const grid = findChild(root, "grid")
        grid.keyIndex = 0
        compare(move(root, "draw", 0, 1), "grid")
        compare(grid.keyIndex, 3)
    }

    function test_half_scrolled_cards_do_not_steal_the_top_bar() {
        const root = scene(railScene)
        findChild(root, "grid").contentY = 150
        compare(move(root, "search", 1, 0), "draw")
    }

    function test_moves_stay_inside_their_scroll_list() {
        const root = scene(stackedScene)
        const fields = findChild(root, "fields")
        compare(move(root, "checkAll", 0, -1), "field4")
        for (let i = 4; i < 9; i++) compare(move(root, "field" + i, 0, 1), "field" + (i + 1))
        verify(fields.contentY > 0)
        compare(move(root, "field9", 0, 1), "checkAll")
        compare(move(root, "checkAll", 0, 1), "game0")
    }
}
