import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.components as PlasmaComponents
import org.kde.plasma.plasma5support as Plasma5Support

PlasmoidItem {
    id: root
    property var filesystems: []
    property string updatedAt: ""
    property string readError: ""

    Plasma5Support.DataSource {
        engine: "executable"
        connectedSources: ["cat /tmp/bcachefs-plasma-status.json"]
        interval: 10000
        onNewData: function(source, data) {
            try {
                root.filesystems = JSON.parse(data["stdout"] || "[]")
                root.updatedAt = Qt.formatTime(new Date(), "hh:mm:ss")
                root.readError = ""
            } catch (e) {
                root.readError = String(e)
            }
        }
    }

    fullRepresentation: Controls.ScrollView {
        ColumnLayout {
            width: parent.width
            spacing: 12
            PlasmaComponents.Label {
                Layout.fillWidth: true
                text: root.readError.length > 0 ? "Could not read status: " + root.readError
                    : root.filesystems.length === 0 ? "Waiting for helper status…"
                    : "Updated " + root.updatedAt
                wrapMode: Text.Wrap
            }
            Repeater {
                model: root.filesystems
                delegate: ColumnLayout {
                    required property var modelData
                    Layout.fillWidth: true
                    spacing: 4
                    PlasmaComponents.Label {
                        text: modelData.mount
                        Layout.fillWidth: true
                        elide: Text.ElideMiddle
                        font.bold: true
                        font.pointSize: font.pointSize + 2
                    }
                    PlasmaComponents.Label { text: modelData.online + "/" + modelData.total + " devices online"; Layout.fillWidth: true }
                    PlasmaComponents.ProgressBar { from: 0; to: 100; value: modelData.percent; Layout.fillWidth: true }
                    PlasmaComponents.Label { text: modelData.used + " used of " + modelData.size + " (" + modelData.percent.toFixed(1) + "%)"; Layout.fillWidth: true; wrapMode: Text.Wrap }
                    PlasmaComponents.Label { text: "Free: " + modelData.free; Layout.fillWidth: true }
                    Repeater {
                        model: modelData.devices
                        delegate: PlasmaComponents.Label {
                            required property var modelData
                            text: modelData.name + " — " + modelData.state
                            Layout.fillWidth: true
                            wrapMode: Text.Wrap
                        }
                    }
                    PlasmaComponents.Label { text: "Reconcile: " + modelData.reconcile + " · scan pending: " + modelData.scan_pending; Layout.fillWidth: true; wrapMode: Text.Wrap }
                    PlasmaComponents.Label { text: modelData.reconcile_detail; Layout.fillWidth: true; wrapMode: Text.Wrap; font.family: "monospace"; opacity: 0.8 }
                    PlasmaComponents.Label { text: modelData.error; visible: modelData.error.length > 0; Layout.fillWidth: true; wrapMode: Text.Wrap }
                    PlasmaComponents.Label { text: "UUID " + modelData.uuid; Layout.fillWidth: true; font.pointSize: 8; opacity: 0.65; wrapMode: Text.Wrap }
                    Rectangle { Layout.fillWidth: true; height: 1; color: "#808080"; opacity: 0.35 }
                }
            }
        }
    }

    compactRepresentation: PlasmaComponents.Label {
        text: root.filesystems.length > 0 ? root.filesystems[0].online + "/" + root.filesystems[0].total + " disks" : "bcachefs"
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
    }
}
