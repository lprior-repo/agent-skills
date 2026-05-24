---
name: makepad-2.0-splash
description: |
  CRITICAL: Use for Makepad 2.0 Splash scripting. Triggers on:
  makepad splash, makepad scripting, script_mod, makepad script,
  makepad hot reload, makepad runtime, Splash language,
  脚本, 热重载, Splash, 运行时
---

# Makepad 2.0 Splash Skill

## Overview

Splash is the scripting language used inside `script_mod!` blocks. It defines UI structure, templates, and inline event handlers at runtime.

## Basics

### script_mod! Structure

```rust
script_mod! {
    use mod.prelude.widgets.*

    // Let bindings create reusable templates
    let MyCard = View{
        width: Fill height: Fit
        padding: 16
        draw_bg.color: #2a2a3d
    }

    // State
    let state = {
        counter: 0
    }
    mod.state = state

    // UI
    startup() do #(App::script_component(vm)){
        ui: Root{
            on_startup: || ui.main_view.render()
            main_window := Window{
                body +: {
                    main_view := View{
                        on_render: || Label{text: "Count: " + state.counter}
                    }
                    Button{text: "Click" on_click: || counter += 1}
                }
            }
        }
    }
}
```

## Data Types

### Numbers
```splash
let x = 42
let y = 3.14
let z = x + y * 2
```

### Strings
```splash
let name = "Hello"
let greeting = name + " World"
```

### Booleans
```splash
let flag = true
let negated = !flag
```

### Arrays
```splash
let items = []
items.push({name: "Item 1", done: false})
items.push({name: "Item 2", done: true})
items.len()  // 2
items[0].name  // "Item 1"
```

### Objects
```splash
let person = {
    name: "Alice"
    age: 30
}
person.name  // "Alice"
```

## Control Flow

### If/Else
```splash
if x > 10 {
    Label{text: "Big"}
} else {
    Label{text: "Small"}
}
```

### For Loop
```splash
for item in items {
    ItemCard{label.text: item.name}
}
```

### While Loop
```splash
let i = 0
while i < 10 {
    Label{text: i.to_string()}
    i += 1
}
```

## Functions

```splash
fn greet(name) {
    return "Hello, " + name
}

fn add(a, b) {
    return a + b
}
```

## State Management

### mod.state

Store shared state in `mod.state`:

```splash
mod.state = {
    counter: 0
    items: []
}

fn increment() {
    mod.state.counter += 1
}

fn add_item(text) {
    mod.state.items.push({text: text, done: false})
}
```

## Templates

### Let Binding
```splash
let Card = View{
    width: Fill height: Fit
    padding: 16
    draw_bg.color: #2a2a3d
    title := Label{text: "Title"}
    body := Label{text: "Body"}
}

// Use with overrides
Card{title.text: "Custom Title"}
```

### Template Inheritance
```splash
let PrimaryButton = Button{
    draw_bg.color: #4488ff
    draw_text.color: #fff
}

let DangerButton = PrimaryButton{
    draw_bg.color: #ff4444
}
```

## Hot Reload

Splash code in `script_mod!` blocks can be hot-reloaded:

```bash
cargo run -- --hot
```

Changes to `script_mod!` content apply without recompilation.

## Debug Logging

```splash
std.println("Debug: " + value.to_string())
```

## Built-in Functions

| Function | Description |
|----------|-------------|
| `std.println(x)` | Print to console |
| `str.to_string()` | Convert to string |
| `num.to_string()` | Convert number to string |
| `"text".len()` | String length |
| `"text".parse_json()` | Parse JSON |
| `array.push(x)` | Add to array |
| `array.len()` | Array length |
| `array.remove(i)` | Remove index |
