---
name: makepad-2.0-dsl
description: |
  CRITICAL: Use for Makepad 2.0 DSL syntax and property system. Triggers on:
  makepad dsl, script_mod!, makepad syntax, makepad property, makepad 2.0 syntax,
  colon syntax, merge operator, named instance, let binding, mod.widgets,
  register_widget, script_component, type_default, widgets_internal
---

# Makepad 2.0 DSL Syntax Skill

## Overview

Makepad 2.0 replaced the compile-time `live_design!` macro with the runtime `script_mod!` macro, powered by the Splash scripting language. This skill covers the complete DSL syntax, property system, registration patterns, and common pitfalls.

## Key Syntax Rules

### Property Assignment: Colon, NOT Equals

```
key: value          // CORRECT - colon syntax
key = value         // WRONG - old 1.x syntax, no longer works
```

### Named Instances: `:=` Operator

Use `:=` to create addressable, named widget instances:

```
my_button := Button{ text: "Click me" }
```

### Merge Operator: `+:`

The `+:` operator extends/merges with the parent instead of replacing:

```
draw_bg +: { color: #f00 }  // MERGES - only changes color
draw_bg: { color: #f00 }    // REPLACES all of draw_bg
```

### Dot-Path Shorthand

```
draw_bg.color: #f00
// is equivalent to:
draw_bg +: { color: #f00 }
```

## Script Module Structure

```rust
use makepad_widgets::*;
app_main!(App);

script_mod!{
    use mod.prelude.widgets.*
    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(800, 600)
                body +: {
                    my_button := Button{ text: "Click" }
                }
            }
        }
    }
}

impl App {
    fn run(vm: &mut ScriptVm) -> Self {
        crate::makepad_widgets::script_mod(vm);
        App::from_script_mod(vm, self::script_mod)
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[source] source: ScriptObjectRef,
    #[live] ui: WidgetRef,
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(ids!(my_button)).clicked(actions) {
            log!("Button clicked!");
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
```

## Common Pitfalls

### 1. Missing `#[source] source: ScriptObjectRef`
All `Script`-derived structs MUST have this field.

### 2. Missing `height: Fit` on Containers
Default height is `Fill`. In a `Fit` parent, `Fill` creates circular dependency = 0 height = invisible.

### 3. Confusing `:` vs `:=`
- `key: value` -- sets a property
- `name := Widget{}` -- creates a named, addressable child

### 4. Forgetting `+:` Merge Operator
```
draw_bg: { color: #f00 }   // WRONG - replaces ALL
draw_bg +: { color: #f00 } // CORRECT - merges
```

### 5. Hex Colors Containing 'e' Need `#x` Prefix
```
color: #x2ecc71  // CORRECT
color: #2ecc71   // WRONG - 'e' interpreted as exponent
```

## Syntax Quick Reference

| Old (live_design!) | New (script_mod!) |
|--------------------|-------------------|
| `<BaseWidget>` | `mod.widgets.BaseWidget{}` or `BaseWidget{}` |
| `{{StructName}}` | `#(Struct::register_widget(vm))` |
| `(THEME_COLOR_X)` | `theme.color_x` |
| `instance hover: 0.0` | `hover: instance(0.0)` |
| `draw_bg: {}` (replace) | `draw_bg +: {}` (merge) |
| `item.apply_over(cx, live!{...})` | `script_apply_eval!(cx, item, {...})` |
