---
sidebar_position: 2
---

# Styled Page

A page that exercises CSS styling, including colors, spacing, flexbox, and borders.

## Example Content

```html
<!DOCTYPE html>
<html>
<head>
    <style>
        body { margin: 0; background: #1a1a2e; color: #eee; font-family: sans-serif; }
        .card {
            background: #16213e; border-radius: 8px;
            padding: 24px; margin: 16px;
            border: 1px solid #0f3460;
        }
        .card h2 { color: #e94560; margin-top: 0; }
    </style>
</head>
<body>
    <div class="card"><h2>Card Title</h2><p>Card content with styled borders and backgrounds.</p></div>
</body>
</html>
```

## What It Shows

- CSS style blocks with class selectors
- Colors (hex with alpha)
- Background colors and borders with `border-radius`
- Padding, margins
- Font fallback to registered system fonts
- CSS cascade resolution (UA + author + specificity)
