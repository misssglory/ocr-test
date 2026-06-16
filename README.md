# Screen OCR full-screen trigger

Minimal Rust workspace with two services:

- `screen-ocr-receiver`: receives an image and runs Tesseract OCR.
- `screen-ocr-sender`: listens locally for `POST /capture`, captures the complete primary monitor, and forwards the PNG to the receiver.

The sender intentionally has no watch mode, focused-window logic, region capture, or monitor-selection CLI.

## Sender configuration

```toml
[trigger]
bind = "127.0.0.1:4490"

[server]
url = "http://192.168.1.33:4489"
token = "dev-secret-change-me"
timeout_secs = 30

[capture]
device_id = "dwl-desktop"
```

## Run

```bash
cargo run --release -p screen-ocr-sender -- --config config.sender.toml
```

Trigger a capture:

```bash
curl -sS -X POST http://127.0.0.1:4490/capture | jq
```

During debugging, do not use `curl -f`, because `-f` hides the JSON error body returned by the sender. To print both status and body:

```bash
curl -sS -X POST \
  -w '\nHTTP %{http_code}\n' \
  http://127.0.0.1:4490/capture
```

## dwl key binding

Add near the command declarations in `config.h`:

```c
static const char *ocrcmd[] = {
    "sh", "-c",
    "curl -sS -X POST http://127.0.0.1:4490/capture "
    ">/tmp/screen-ocr-last.json "
    "2>/tmp/screen-ocr-last.err",
    NULL
};
```

Add to `static const Key keys[]`:

```c
{ MODKEY|WLR_MODIFIER_SHIFT, XKB_KEY_o, spawn, {.v = ocrcmd} },
```

Rebuild and restart `dwl`:

```bash
make clean
sudo make install
```

The binding is `Mod + Shift + O`.

## Wayland note

Run the sender in the same user session as `dwl`, preserving at least:

```text
WAYLAND_DISPLAY
XDG_RUNTIME_DIR
DBUS_SESSION_BUS_ADDRESS
```

If capture still fails, call the endpoint without `-f` and inspect the JSON error body, plus start the sender with:

```bash
RUST_LOG=debug ./target/release/screen-ocr-sender --config config.sender.toml
```
