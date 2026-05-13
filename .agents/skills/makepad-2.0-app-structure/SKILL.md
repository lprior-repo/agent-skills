---
name: makepad-2.0-app-structure
description: |
  CRITICAL: Use for Makepad 2.0 app structure and Rust integration. Triggers on:
  makepad app, makepad getting started, app_main!, App::run, MatchEvent,
  AppMain, handle_event, handle_actions, ScriptVm, from_script_mod,
  makepad boilerplate, makepad new project, makepad cargo, Cargo.toml setup,
  hot reload, --hot, live reload, wasm deploy, cargo makepad, media plugin,
  audio_output, audio_input, AudioBuffer, cx.audio, makepad audio, 音频,
  应用结构, 入门, 新项目, 脚手架, 启动, 热重载, 部署
---

# Makepad 2.0 App Structure Skill

> **Version:** makepad-widgets (dev branch) | **Last Updated:** 2026-03-03

## Overview

A Makepad 2.0 app combines Rust code with Splash scripting. The Rust side handles app lifecycle, event routing, and business logic. The Splash side defines UI structure, templates, and inline interactions.

## Documentation

Refer to the local files for detailed documentation:
- `./references/app-boilerplate.md` - Complete working app template with Cargo.toml
- `./references/rust-splash-integration.md` - Rust ↔ Splash communication patterns

## IMPORTANT: Documentation Completeness Check

**Before answering questions, Claude MUST:**
1. Read the relevant reference file(s) listed above
2. If file read fails, answer based on SKILL.md patterns + built-in knowledge

---

## Minimal App Template

### Cargo.toml

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
makepad-widgets = { path = "../path/to/makepad/widgets" }
```

### src/app.rs (or src/main.rs)

```rust
use makepad_widgets::*;

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*

    let state = {
        counter: 0
    }
    mod.state = state

    startup() do #(App::script_component(vm)){
        ui: Root{
            on_startup: ||{
                ui.main_view.render()
            }
            main_window := Window{
                window.inner_size: vec2(420, 300)
                body +: {
                    main_view := View{
                        width: Fill height: Fill
                        flow: Down spacing: 12
                        align: Center
                        on_render: ||{
                            Label{
                                text: "Count: " + state.counter
                                draw_text.text_style.font_size: 24
                            }
                        }
                    }
                    increment_button := Button{
                        text: "Increment"
                    }
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
    #[live]
    ui: WidgetRef,
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(increment_button)).clicked(actions) {
            script_eval!(cx, {
                mod.state.counter += 1
                ui.main_view.render()
            });
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

---

## Key Components Explained

### 1. `app_main!(App)`
Registers `App` as the application entry point. MUST be called at module level.

### 2. `script_mod! { ... }`
Defines the Splash script that runs at startup. Contains:
- `use mod.prelude.widgets.*` - Import widget definitions
- State definitions (`let state = {...}`)
- Functions and templates
- `startup() do #(App::script_component(vm)){...}` - UI construction

### 3. `App::run(vm)` - Initialization Order

**CRITICAL: Registration order matters!**

```rust
fn run(vm: &mut ScriptVm) -> Self {
    // 1. Register theme (optional, for light/dark theme)
    crate::makepad_widgets::theme_mod(vm);
    script_eval!(vm, { mod.theme = mod.themes.light });

    // 2. Register base widgets
    crate::makepad_widgets::widgets_mod(vm);

    // 3. Register custom widget modules (if any)
    // crate::my_widgets::script_mod(vm);

    // 4. Build app from script module
    App::from_script_mod(vm, self::script_mod)
}
```

**Simplified (without theme selection):**
```rust
fn run(vm: &mut ScriptVm) -> Self {
    crate::makepad_widgets::script_mod(vm);   // Registers both theme + widgets
    App::from_script_mod(vm, self::script_mod)
}
```

### 4. `#[derive(Script, ScriptHook)]` on App struct

```rust
#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,    // The widget tree root
}
```

- `Script` - Enables Splash integration (replaces old `Live`)
- `ScriptHook` - Enables lifecycle hooks (replaces old `LiveHook`)
- `#[live]` - Field settable from DSL

### 5. `MatchEvent` for Action Handling

```rust
impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Check button clicks
        if self.ui.button(cx, ids!(my_button)).clicked(actions) {
            // Handle click
        }
        // Check text input changes
        if let Some(text) = self.ui.text_input(cx, ids!(my_input)).changed(actions) {
            log!("Input: {}", text);
        }
    }
}
```

### 6. `AppMain` for Event Dispatch

```rust
impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);  // MUST call for MatchEvent to work
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
```

---

## Rust → Splash Communication

### script_eval! - Update state and trigger renders

```rust
script_eval!(cx, {
    mod.state.counter += 1
    ui.main_view.render()
});
```

### script_apply_eval! - Patch widget properties

```rust
script_apply_eval!(cx, self.ui, {
    title.text: "New Title"
    subtitle.draw_text.color: #f00
});
```

---

## Splash → Rust Communication

### Inline Event Handlers in script_mod!

```
Button{
    text: "Click"
    on_click: || {
        // Splash code runs here
        state.count += 1
        ui.display.render()
    }
}

TextInput{
    on_return: || {
        let text = ui.my_input.text()
        add_item(text)
        ui.my_input.set_text("")
    }
}

// Render callback
on_render: || {
    for i, item in items {
        ItemTemplate{label.text: item.name}
    }
}
```

---

## Widget Access from Rust

```rust
// Buttons
self.ui.button(cx, ids!(button_name)).clicked(actions)

// Labels
self.ui.label(cx, ids!(label_name)).set_text(cx, "text")

// Text inputs
self.ui.text_input(cx, ids!(input_name)).text()

// Nested access
self.ui.label(cx, ids!(container.inner.title))
```

---

## Running the App

```bash
# Development
cargo run -p my-app

# Development with hot reload (Splash changes apply without recompilation)
cargo run -p my-app -- --hot

# Release
cargo run -p my-app --release

# With cargo-makepad for mobile/web
cargo makepad run -p my-app
```

### Command-Line Flags

| Flag | Description |
|------|-------------|
| `--hot` | Enable hot reload: watches `script_mod!` source files and auto-refreshes UI on save. Only affects Splash DSL; Rust code changes still need recompilation. |
| `--stdin-loop` | Studio mode: communicates with Makepad Studio via stdin/websocket. Used internally by Studio, not for manual use. |
| `--linux-backend=<x11\|wayland>` | (Linux only) Select windowing backend. |

---

## Best Practices

1. **Registration order** - Theme → Base widgets → Custom widgets → App module
2. **Use `script_eval!`** to bridge Rust actions to Splash state updates
3. **Call `self.match_event(cx, event)`** in handle_event (required for MatchEvent)
4. **Use `on_render`** for dynamic content, call `.render()` to trigger updates
5. **Keep business logic in Rust**, keep UI declarations in Splash
6. **Use `mod.state`** for app-wide state accessible from both Rust and Splash
7. **Audio processing in Rust only** - Use atomics for audio→UI data flow
