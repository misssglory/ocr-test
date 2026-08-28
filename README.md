# Screen OCR Region Bridge

A small Rust workspace for Wayland/NixOS:

1. A local HTTP trigger starts region selection with `slurp`.
2. `grim` captures only the selected rectangle as PNG to stdout.
3. The sender runs local Tesseract OCR through stdin/stdout.
4. Only UTF-8 text and metadata are sent to the remote receiver.
5. The receiver optionally saves each result as a `.txt` file.

No screenshot is written to disk or sent over the network.

## Workspace

- `screen-ocr-sender`: local trigger, `slurp`, `grim`, Tesseract, HTTP client.
- `screen-ocr-receiver`: accepts text at `POST /v1/text`.
- `screen-ocr-common`: shared JSON structures.

## Requirements

`grim` and `slurp` must be installed system-wide and available in `PATH`.
The supplied development shell includes Rust, Tesseract, and the requested Wayland libraries.

```bash
nix develop
cargo build --release
```

Check OCR languages:

```bash
tesseract --list-langs
```

If `rus` is unavailable, use `languages = "eng"` or install the Russian trained data system-wide.

## Configuration

Edit `config.sender.toml` and set the receiver IP. The bearer token must match `config.receiver.toml`.

## Start receiver

On the receiving device:

```bash
./target/release/screen-ocr-receiver --config config.receiver.toml
```

Health check:

```bash
curl -sS http://127.0.0.1:4489/healthz
```

## Start sender

Run sender inside the same Wayland user session as `dwl`:

```bash
RUST_LOG=info ./target/release/screen-ocr-sender --config config.sender.toml
```

It must inherit `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR`.

## Trigger selection

```bash
curl -sS -X POST http://127.0.0.1:4490/capture
```

After the request, select a rectangle with the mouse. The endpoint waits until selection, capture, OCR, and remote delivery are complete.

Pressing Escape cancels `slurp`; the sender returns HTTP 409 with a JSON error.

For debugging, avoid `curl -f`, because it hides the response body on HTTP errors:

```bash
curl -sS -w '\nHTTP %{http_code}\n' -X POST http://127.0.0.1:4490/capture
```

## Direct pipeline test

Verify the external tools before testing Rust:

```bash
geometry="$(slurp -f '%x,%y %wx%h')" && \
  grim -g "$geometry" - | \
  tesseract stdin stdout -l eng+rus --psm 6
```

## dwl keyboard binding

Add this command definition to `config.h`:

```c
static const char *ocrcmd[] = {
    "curl", "-sS", "-X", "POST", "http://127.0.0.1:4490/capture",
    NULL
};
```

Add a key entry to `static const Key keys[]`:

```c
{ MODKEY|WLR_MODIFIER_SHIFT, XKB_KEY_o, spawn, {.v = ocrcmd} },
```

This binds `Mod + Shift + O`.

Rebuild and restart dwl:

```bash
make clean
sudo make install
```

The direct `curl` command writes the JSON response to dwl's inherited stdout. If dwl was started from a TTY, the response appears there. For persistent output, use a wrapper script or shell command that redirects stdout to a file.

## API

### Sender

`POST http://127.0.0.1:4490/capture`

### Receiver

`POST /v1/text`

```json
{
  "device_id": "dwl-desktop",
  "text": "recognized text",
  "image_sha256": "...",
  "monitor_name": "selected-region:100,200 800x600",
  "width": 800,
  "height": 600,
  "local_ocr_ms": 214
}
```

## Security

The default transport is plain HTTP and is intended only for a trusted local network. Bind the sender trigger to `127.0.0.1`; do not expose port `4490` to the LAN.
