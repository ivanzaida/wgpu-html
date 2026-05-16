# Font measurement gaps

## min-content width differs from browser

Flex items with `min-width: auto` (the default) shrink down to their min-content width.
When items hit this floor, the flex-shrink factor no longer matters — font metrics determine the final size.

Our engine produces slightly different min-content widths than browsers, especially for:

- **Monospace fonts**: `font-family: monospace` character widths differ from browser defaults (e.g. Courier New vs our fallback). This causes visible differences in flex-shrink scenarios where items are clamped to min-content.
- **Proportional fonts**: Subtle glyph-width differences accumulate across characters.

### Example: flex-shrink demo (section 8)

```
Container: 200px, items: 150px each, shrink 0/1/3
Browser: s:0=150  s:1=40.33  s:3=47.1   (total 237.43, overflows)
```

Both s:1 and s:3 hit their min-content floor. s:3 is *wider* than s:1 despite 3x shrink because `.fx-c` uses monospace, which has wider min-content.

### What to fix

Align font fallback chains and default metrics with browser defaults so that min-content measurements match more closely. This affects any layout where `min-width: auto` is the limiting factor.
