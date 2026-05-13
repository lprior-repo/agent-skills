---
name: makepad-2.0-migration
description: |
  CRITICAL: Use for migrating from Makepad 1.x to 2.0. Triggers on:
  makepad migration, makepad 1.x to 2.0, live_design, Live, LiveHook,
  makepad upgrade, deprecated, legacy makepad, 迁移, 升级
---

# Makepad 2.0 Migration Guide

## Overview

Makepad 2.0 represents a fundamental architectural shift from the 1.x compile-time `live_design!` system to the runtime `script_mod!` system. This skill covers migration patterns and common breaking changes.

## Key Breaking Changes

### 1. live_design! -> script_mod!

**Old (1.x):**
```rust
live_design!{
    use makepad_widgets::theme_desktop_dark::*;
    App = {{App}} {
        ui: <DesktopWindow> {
            button = <Button> {}
        }
    }
}

#[derive(Live)]
pub struct App { ... }
impl LiveHook for App { ... }
```

**New (2.0):**
```rust
app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                body +: {
                    button := Button{ text: "Click" }
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
```

### 2. Live -> Script

| Old | New |
|-----|-----|
| `#[derive(Live)]` | `#[derive(Script, ScriptHook)]` |
| `impl LiveHook` | Uses `ScriptHook` automatically via derive |
| `LiveId` | `LiveId` (unchanged) |
| `live_design!{}` | `script_mod!{}` |

### 3. Widget Registration

**Old:**
```rust
App = {{App}} {
    ui: <DesktopWindow> {
        my_widget = <MyWidget> {}
    }
}
```

**New:**
```rust
mod.widgets.MyWidgetBase = #(MyWidget::register_widget(vm))
mod.widgets.MyWidget = set_type_default() do mod.widgets.MyWidgetBase{
    width: Fill height: Fit
}
```

### 4. Property Assignment

| Old | New |
|-----|-----|
| `width: (value)` | `width: value` |
| `draw_bg: { color: #fff }` | `draw_bg +: { color: #fff }` (merge) |
| `instance hover: 0.0` | `hover: instance(0.0)` |

### 5. Event Handling

**Old:**
```rust
fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
    match event.state() {
        EventState::Handled => return,
        _ => {}
    }
    // ...
}
```

**New:**
```rust
impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(my_button)).clicked(actions) {
            // Handle click
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

### 6. apply_over + live! -> script_apply_eval!

**Old:**
```rust
item.apply_over(cx, live!{ height: (height) });
```

**New:**
```rust
script_apply_eval!(cx, item, {
    height: #(height)
});
```

### 7. Theme Access

**Old:**
```rust
color: (THEME_COLOR_BG)
```

**New:**
```rust
color: theme.color_bg_app
```

## Migration Checklist

- [ ] Replace `live_design!` with `script_mod!`
- [ ] Add `#[source] source: ScriptObjectRef` to all Script-derived structs
- [ ] Change `#[derive(Live)]` to `#[derive(Script, ScriptHook)]`
- [ ] Replace `impl LiveHook` with `impl MatchEvent` for event handling
- [ ] Change all `key: (value)` to `key: value`
- [ ] Change `draw_bg: {}` to `draw_bg +: {}` for merging
- [ ] Update widget registration to use `register_widget(vm)` pattern
- [ ] Replace `apply_over + live!` with `script_apply_eval!`
- [ ] Update theme constants from `(THEME_X)` to `theme.color_x`
- [ ] Add `app_main!(App)` and `AppMain` impl
- [ ] Call `crate::makepad_widgets::script_mod(vm)` in `App::run`

## Common Errors

### "widget method not found"
Usually means widget not registered. Call `crate::makepad_widgets::script_mod(vm)` before building app.

### "variable X not found in scope"
Widget declared with `:` instead of `:=`. Named widgets require `:=`.

### "height: Fill inside Fit = invisible"
Circular dependency. Add `height: Fit` to container Views.

### "draw_bg replaced all properties"
Forgot `+:` merge operator. Use `draw_bg +: {}` not `draw_bg: {}`.
