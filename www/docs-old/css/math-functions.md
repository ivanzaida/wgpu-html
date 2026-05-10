---
title: CSS Math Functions
---

# CSS Math Functions

lui implements a full CSS math expression parser and evaluator. All numeric CSS properties that accept lengths can use `calc()`, `min()`, `max()`, `clamp()`, and 18 trigonometric/mathematical functions. Math expressions are parsed into an AST, stored as part of `CssLength`, and evaluated at layout time in `length.rs`.

## `calc()`

The `calc()` function performs arithmetic with mixed CSS units:

```css
width: calc(100% - 40px);
font-size: calc(1rem + 2px);
margin: calc(var(--spacing) * 2);
height: calc(100vh - 60px);
padding: calc(10px + 2vw);
```

### Supported Operators

| Operator | Description | Example |
|---|---|---|
| `+` | Addition | `calc(100px + 20px)` |
| `-` | Subtraction | `calc(100% - 50px)` |
| `*` | Multiplication (at least one operand must be a number) | `calc(10px * 2)`, `calc(var(--scale) * 100%)` |
| `/` | Division (right operand must be a number) | `calc(100px / 2)`, `calc(100% / var(--cols))` |

### Operator Precedence

`calc()` follows standard mathematical precedence:
1. Parentheses `()`
2. Multiplication `*` and division `/`
3. Addition `+` and subtraction `-`

```css
calc(10px + 20px * 3)    /* → 70px, not 90px */
calc((10px + 20px) * 3)  /* → 90px */
```

### Mixing Units

`calc()` can mix different unit types in sums and differences:

```css
width: calc(100% - 32px);        /* percentage minus pixels */
height: calc(100vh - 80px);      /* viewport minus pixels */
margin: calc(2em + 10px);        /* em plus pixels */
font-size: calc(1vw + 0.5rem);   /* viewport plus rem */
```

The evaluator at layout time resolves each operand to a `f32` pixel value before combining them. Relative units (`%`, `em`, `rem`, `vw`, `vh`, `vmin`, `vmax`) are resolved against the appropriate reference:
- `%` → containing block dimension
- `em` / `rem` → font size
- `vw` / `vh` → viewport dimensions

### Nesting

`calc()` can be nested inside other `calc()` expressions:

```css
width: calc(calc(100% - 40px) / 3);
```

## `min()`

Returns the smallest value from a comma-separated list:

```css
width: min(100%, 800px);           /* responsive: at most 800px */
height: min(50vh, 600px);          /* at most 50vh or 600px */
font-size: min(2vw, 24px);         /* responsive typography */
padding: min(2vw, 16px);           /* responsive spacing */
```

The function evaluates each argument and returns the minimum. Any number of arguments is supported:

```css
width: min(100%, 800px, calc(100vw - 40px));
```

## `max()`

Returns the largest value from a comma-separated list:

```css
width: max(300px, 50%);            /* at least 300px */
font-size: max(14px, 1.2vw);       /* at least 14px */
height: max(100vh, 600px);         /* at least viewport or 600px */
margin: max(16px, 2vw);            /* minimum comfortable spacing */
```

## `clamp()`

Clamps a value between a minimum and maximum. Syntax: `clamp(MIN, PREFERRED, MAX)`:

```css
font-size: clamp(14px, 2vw, 24px);            /* fluid typography */
width: clamp(300px, 50%, 800px);              /* responsive container */
padding: clamp(8px, 2vw, 32px);               /* responsive padding */
margin: clamp(16px, 5vw, 64px);               /* responsive margins */
```

```
font-size: clamp(14px, 2vw, 24px);

Viewport width →  0px     700px    1200px
                 ├────────┼────────┤
font-size:       14px     14px     24px
values:          14px      2vw      2vw
                 (min)    (pref)   (max)
```

The three arguments must be provided. The result is `max(MIN, min(PREFERRED, MAX))`.

## 18 CSS Math Function AST Nodes

The full `CssMathExpr` AST supports 18 mathematical functions:

```rust
pub enum CssMathExpr {
  Length(CssLength),       // length value leaf
  Number(f32),             // number value leaf
  Add(Box<CssMathExpr>, Box<CssMathExpr>),
  Sub(Box<CssMathExpr>, Box<CssMathExpr>),
  Mul(Box<CssMathExpr>, Box<CssMathExpr>),
  Div(Box<CssMathExpr>, Box<CssMathExpr>),
  Function(CssNumericFunction, Vec<CssMathExpr>),
}
```

### Trigonometric Functions

| Function | Description | Example |
|---|---|---|
| `sin()` | Sine | `calc(100px * sin(45deg))` |
| `cos()` | Cosine | `calc(100px * cos(60deg))` |
| `tan()` | Tangent | `calc(100px * tan(30deg))` |
| `asin()` | Arc sine | `calc(100px * asin(0.5))` |
| `acos()` | Arc cosine | `calc(100px * acos(0.5))` |
| `atan()` | Arc tangent | `calc(100px * atan(1))` |
| `atan2()` | Two-argument arc tangent | `calc(100px * atan2(1, 1))` |

### Exponential Functions

| Function | Description | Example |
|---|---|---|
| `pow()` | Power | `calc(2px * pow(2, 3))` → 8px |
| `sqrt()` | Square root | `calc(sqrt(16) * 1px)` → 4px |
| `hypot()` | Hypotenuse | `calc(hypot(3, 4) * 1px)` → 5px |
| `log()` | Natural logarithm | `calc(log(2.718) * 1px)` |
| `exp()` | e raised to power | `calc(exp(1) * 1px)` |

### Other Math Functions

| Function | Description | Example |
|---|---|---|
| `abs()` | Absolute value | `calc(abs(-10) * 1px)` → 10px |
| `sign()` | Sign (-1, 0, 1) | `calc(sign(-5) * 10px)` → -10px |
| `mod()` | Modulus | `calc(mod(7, 3) * 1px)` → 1px |
| `rem()` | Remainder | `calc(rem(7, 3) * 1px)` |
| `round()` | Round to nearest | `calc(round(3.6) * 1px)` → 4px |

These functions are parsed into `CssMathExpr::Function(kind, args)` and can be used inside `calc()`, `min()`, `max()`, and `clamp()`. They can also be used as standalone values inside any length-accepting property.

## Evaluation at Layout Time

Math expressions are evaluated in `lui-layout/src/length.rs` during the layout pass. The evaluator:

1. Recursively walks the `CssMathExpr` AST
2. Resolves each `Length` leaf to a `f32` pixel value using the layout context (containing block size, font size, viewport size)
3. Evaluates `Number` leaves as `f32`
4. Performs arithmetic operations (`+`, `-`, `*`, `/`)
5. Evaluates function calls with their resolved arguments

The resulting pixel value is used for all subsequent layout calculations.

### `min()`, `max()`, `clamp()` in the AST

These parse directly into `CssLength` variants:

```rust
pub enum CssLength {
  // ...
  Calc(Box<CssMathExpr>),     // single calc() expression
  Min(Vec<CssLength>),        // min(a, b, c, ...)
  Max(Vec<CssLength>),        // max(a, b, c, ...)
  Clamp {                     // clamp(min, preferred, max)
    min: Box<CssLength>,
    preferred: Box<CssLength>,
    max: Box<CssLength>,
  },
}
```

At evaluation time, each inner `CssLength` is resolved to a pixel value, then `min`/`max`/`clamp` select or clamp as appropriate.

## Code Examples

### Fluid Typography System

```css
h1 { font-size: clamp(1.5rem, 4vw, 3rem); }
h2 { font-size: clamp(1.25rem, 3vw, 2rem); }
h3 { font-size: clamp(1rem, 2.5vw, 1.5rem); }
p { font-size: clamp(0.875rem, 1.5vw, 1.125rem); }
```

### Responsive Container

```css
.container {
  width: min(100% - 32px, 1200px);
  margin-inline: auto;
}
```

### Sidebar + Content Layout

```css
.layout {
  display: flex;
  gap: clamp(16px, 3vw, 32px);
}

.sidebar {
  width: clamp(200px, 25vw, 300px);
  flex-shrink: 0;
}

.content {
  flex: 1;
  min-width: 0;
}
```

### Dynamic Grid with calc()

```css
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(300px, 100%), 1fr));
  gap: clamp(12px, 2vw, 24px);
}
```

### Spacing Scale with calc()

```css
:root {
  --space-unit: clamp(4px, 1vw, 8px);
}

.section {
  padding: calc(var(--space-unit) * 4) calc(var(--space-unit) * 3);
}

.card {
  padding: calc(var(--space-unit) * 2);
  margin-bottom: calc(var(--space-unit) * 3);
}

.card + .card {
  margin-top: calc(var(--space-unit) * 2);
}
```

### CSS Math Function with Trigonometry

```css
/* Not typically practical but demonstrates capabilities */
.diagonal-box {
  width: calc(100px * cos(30deg));
  height: calc(100px * sin(30deg));
}

.golden-ratio {
  width: calc(100% / 1.618);
}
```
