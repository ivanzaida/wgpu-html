---
sidebar_position: 1
---

# Simple Page

A minimal HTML page rendered with lui. Demonstrates the basic pipeline: parse, register fonts, and display.

## Source

See `demo/lui-demo/html/` for example HTML files. The simplest possible page:

```html
<!DOCTYPE html>
<html>
<body>
    <h1>Hello, lui!</h1>
    <p>This is a simple HTML page rendered with GPU acceleration.</p>
</body>
</html>
```

## Running

```bash
cargo run -p lui-demo -- demo/lui-demo/html/simple.html
```

## What It Shows

- HTML parsing of basic elements (`<html>`, `<body>`, `<h1>`, `<p>`)
- Block layout with default margins
- Text rendering from registered system fonts
- Default UA stylesheet behavior (heading sizes, body margin)
