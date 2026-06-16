# Screen OCR Bridge

A small Rust workspace with two binaries:

- `screen-ocr-sender`: captures a monitor or configured region and sends PNG/JPEG bytes.
- `screen-ocr-receiver`: receives the image, runs Tesseract immediately, optionally stores the image/text, and returns OCR output as JSON.

## Data flow

```text
Device A                                      Device B
screen capture -> PNG/JPEG -> HTTP POST ---> receiver -> Tesseract OCR
                    <--- JSON OCR result <---
```

The transport is intentionally plain HTTP for simple LAN/VPN use. Screenshots may contain secrets, so do not expose port `4489` directly to the public internet. Use a trusted LAN, WireGuard/Tailscale, or put the receiver behind an HTTPS reverse proxy.

## Requirements

### Receiver

- Rust 1.85+
- Tesseract 5 available as `tesseract`
- The traineddata files for every language configured in `config.receiver.toml`

Check installed OCR languages:

```bash
tesseract --list-langs
```

### Sender on Linux

XCap needs the native X11/Wayland/PipeWire development libraries. The included `flake.nix` provides a development shell for NixOS/Linux.

```bash
nix develop
```

On Debian/Ubuntu, XCap documents these build dependencies:

```bash
sudo apt-get install pkg-config libclang-dev libxcb1-dev libxrandr-dev \
  libdbus-1-dev libpipewire-0.3-dev libwayland-dev libegl-dev
```

Install Tesseract on the receiver, for example:

```bash
sudo apt-get install tesseract-ocr tesseract-ocr-eng tesseract-ocr-rus
```

## Configure

Use the same long random token on both devices.

Receiver, `config.receiver.toml`:

```toml
[server]
bind = "0.0.0.0:4489"
token = "replace-with-a-long-random-token"
max_upload_mb = 20

[ocr]
command = "tesseract"
languages = "eng+rus"
psm = 6
oem = 1
timeout_secs = 30

[storage]
save_images = true
save_text = true
directory = "./received"
```

Sender, `config.sender.toml`:

```toml
[server]
url = "http://192.168.1.33:4489"
token = "replace-with-a-long-random-token"
timeout_secs = 30

[capture]
device_id = "desktop-one"
prefer_primary = true
format = "png"
jpeg_quality = 90
interval_ms = 1000
send_unchanged = false
```

For a crop, add:

```toml
[capture.region]
x = 100
y = 100
width = 1200
height = 800
```

## Run

Start the receiver on Device B:

```bash
cargo run --release -p screen-ocr-receiver -- --config config.receiver.toml
```

List monitors on Device A:

```bash
cargo run --release -p screen-ocr-sender -- \
  --config config.sender.toml list-monitors
```

Capture once:

```bash
cargo run --release -p screen-ocr-sender -- \
  --config config.sender.toml once
```

Watch continuously and OCR only changed frames:

```bash
cargo run --release -p screen-ocr-sender -- \
  --config config.sender.toml watch
```

Set `send_unchanged = true` to send every interval even when the encoded screenshot hash is unchanged.

## Receiver API

### Health

```bash
curl http://127.0.0.1:4489/healthz
```

### OCR image upload

```bash
curl -X POST http://127.0.0.1:4489/v1/ocr \
  -H 'Authorization: Bearer dev-secret-change-me' \
  -H 'Content-Type: image/png' \
  -H 'X-Device-Id: manual-test' \
  --data-binary @screenshot.png
```

Example response:

```json
{
  "request_id": "9b892dd2-7c14-49e4-89ab-cbc3f85dd165",
  "device_id": "desktop-one",
  "text": "Recognized text",
  "elapsed_ms": 241,
  "image_sha256": "...",
  "image_saved_to": "./received/9b892dd2-7c14-49e4-89ab-cbc3f85dd165.png",
  "text_saved_to": "./received/9b892dd2-7c14-49e4-89ab-cbc3f85dd165.txt"
}
```

## OCR tuning

Useful Tesseract page segmentation modes:

- `psm = 3`: automatic page segmentation.
- `psm = 6`: one uniform block of text; a good default for app windows.
- `psm = 11`: sparse text scattered across the screen.
- `psm = 13`: one raw text line.

For small UI text, crop the region before sending. It reduces network traffic and generally improves OCR accuracy.

## Notes

- Wayland may show a portal permission dialog, depending on compositor and desktop environment.
- macOS requires Screen Recording permission for the sender.
- Windows may require allowing the receiver through Windows Firewall on private networks.
- The bearer token authenticates requests but does not encrypt the image. Use a VPN or TLS when the network is not trusted.
