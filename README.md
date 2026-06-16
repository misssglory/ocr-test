# Screen OCR bridge — client-side Tesseract

The sender captures the full primary monitor, runs Tesseract locally, and sends only recognized UTF-8 text as JSON to the receiver.

## Flow

`POST sender:4490/capture -> screenshot -> local Tesseract -> POST receiver:4489/v1/text`

## Sender requirements

Install Tesseract and the desired language data. For English and Russian on NixOS, add the relevant Tesseract language packages or use a Tesseract package containing those traineddata files.

Verify:

```bash
tesseract --version
tesseract --list-langs
```

## Build

```bash
nix develop
cargo build --release
```

## Run receiver

```bash
./target/release/screen-ocr-receiver --config config.receiver.toml
```

Check it:

```bash
curl http://127.0.0.1:4489/healthz
```

## Run sender

Edit `config.sender.toml` and set the receiver IP, then:

```bash
./target/release/screen-ocr-sender --config config.sender.toml
```

Trigger capture:

```bash
curl -sS -X POST http://127.0.0.1:4490/capture | jq
```

For Russian and English:

```toml
[ocr]
languages = "eng+rus"
```

## dwl binding

In `config.h`:

```c
static const char *ocrcmd[] = {
    "sh", "-c",
    "curl -sS -X POST http://127.0.0.1:4490/capture "
    ">/tmp/screen-ocr-last.json "
    "2>/tmp/screen-ocr-last.err",
    NULL
};
```

Add to `keys[]`:

```c
{ MODKEY|WLR_MODIFIER_SHIFT, XKB_KEY_o, spawn, {.v = ocrcmd} },
```

Then rebuild and restart dwl.

## Direct receiver test

```bash
curl -sS -X POST http://RECEIVER_IP:4489/v1/text \
  -H 'Authorization: Bearer dev-secret-change-me' \
  -H 'Content-Type: application/json' \
  -d '{
    "device_id":"test",
    "text":"hello",
    "image_sha256":"manual-test",
    "monitor_name":"manual",
    "width":0,
    "height":0,
    "local_ocr_ms":0
  }' | jq
```
