# Screen OCR focused-window trigger

Minimal Rust workspace with two programs:

- `screen-ocr-sender`: runs a local HTTP trigger. Every `POST /capture` captures the currently focused, non-minimized window and sends the PNG to the receiver.
- `screen-ocr-receiver`: accepts the PNG, runs Tesseract OCR, and returns JSON.

The sender intentionally has no monitor selection, region mode, polling loop, or watch mode.

## Sender configuration

Edit `config.sender.toml`:

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

The trigger is bound to loopback only. The receiver can be on another machine in the LAN.

## Build

```bash
nix develop
cargo build --release
```

## Start receiver

On the OCR machine:

```bash
./target/release/screen-ocr-receiver --config config.receiver.toml
```

## Start sender

Inside the graphical `dwl` session:

```bash
./target/release/screen-ocr-sender --config config.sender.toml
```

It must inherit the graphical-session environment, especially `WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`, and the PipeWire/portal session variables.

## Trigger manually

```bash
curl -fsS -X POST http://127.0.0.1:4490/capture | jq
```

The shell command does not create a graphical window, so the application that was already focused remains the focused capture target.

## dwl key binding

In `config.h`, add a command near the other command arrays:

```c
static const char *ocrcmd[] = {
    "sh", "-c",
    "curl -fsS -X POST http://127.0.0.1:4490/capture "
    ">/tmp/screen-ocr-last.json 2>/tmp/screen-ocr-last.err",
    NULL
};
```

Then add a key inside `static const Key keys[]`:

```c
{ MODKEY|WLR_MODIFIER_SHIFT, XKB_KEY_o, spawn, {.v = ocrcmd} },
```

This example binds `Mod+Shift+O`.

Rebuild and reinstall dwl using the same method you normally use, for example:

```bash
make clean
sudo make install
```

Then restart the compositor session.

If your dwl tree uses the `SHCMD` helper, the equivalent key is:

```c
{ MODKEY|WLR_MODIFIER_SHIFT, XKB_KEY_o, spawn,
  SHCMD("curl -fsS -X POST http://127.0.0.1:4490/capture >/tmp/screen-ocr-last.json 2>/tmp/screen-ocr-last.err") },
```

Use only one of the two forms.

## Inspect the last result

```bash
jq . /tmp/screen-ocr-last.json
cat /tmp/screen-ocr-last.err
```

## Wayland note

The focused-window lookup and capture use XCap's `Window::all`, `Window::is_focused`, and `Window::capture_image` APIs. XCap marks Wayland window capture as available but not fully supported in every scenario. On a `dwl` setup, the sender must be launched from the same user and graphical session. Depending on the installed portal/PipeWire stack, the compositor may display a permission dialog or reject window capture.
