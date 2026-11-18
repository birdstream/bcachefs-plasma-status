# Bcachefs Status for Plasma

A small KDE Plasma 6 widget, backed by a Rust helper, for monitoring mounted bcachefs filesystems. It shows online devices, filesystem usage, free space, and `bcachefs reconcile status` information.

## Requirements

- KDE Plasma 6
- Rust and Cargo
- `bcachefs-tools`

## Build and install

Clone the repository:

```sh
git clone https://github.com/birdstream/bcachefs-plasma-status.git
cd bcachefs-plasma-status
```

Build the helper from the project root:

```sh
cargo build --release
```

Install the widget package:

```sh
mkdir -p ~/.local/share/plasma/plasmoids/io.github.birdstream.bcachefsstatus
cp -a package/. ~/.local/share/plasma/plasmoids/io.github.birdstream.bcachefsstatus/
```

Add **Bcachefs Status** from Plasma's widget picker.

## Run the helper

Start it manually:

```sh
./target/release/bcachefs-plasma-status --watch
```

The helper atomically writes status to `/tmp/bcachefs-plasma-status.json` every 10 seconds. Set `BCACHEFS_STATUS_FILE` to use a different path. The widget reads the status file every 10 seconds through Plasma's executable data engine.

The helper only reads mounted filesystem information and `/sys/fs/bcachefs`; it does not modify filesystems.

## Start automatically

Install the helper and enable its systemd user service:

```sh
install -Dm755 target/release/bcachefs-plasma-status ~/.local/bin/bcachefs-plasma-status
install -Dm644 systemd/bcachefs-plasma-status.service ~/.config/systemd/user/bcachefs-plasma-status.service
systemctl --user daemon-reload
systemctl --user enable --now bcachefs-plasma-status.service
loginctl enable-linger "$USER"
```

The service starts at system boot, including before login, restarts the helper if it exits, and continues running after logout.

## Package

The Plasma widget source is in `package/`. To create a `.plasmoid` archive:

```sh
zip -qr bcachefs-plasma-status.plasmoid package
```
