---
name: makepad-2.0-events
description: |
  CRITICAL: Use for Makepad 2.0 event and action handling. Triggers on:
  makepad event, makepad action, MatchEvent, handle_event, handle_actions,
  on_click, on_render, on_return, on_startup, script_eval!, script_apply_eval!,
  button clicked, text changed, slider changed, checkbox toggled,
  Hit, FingerDown, FingerUp, KeyDown, KeyUp, Focus, ids!,
  事件, 动作, 点击, 输入, 回调, 交互, 事件处理, 剪贴板, 选择, 弹出窗口
---

# Makepad 2.0 Event & Action System

## Overview

Makepad 2.0 uses a **two-layer event system**:

1. **Splash Layer** -- Inline event handlers in `script_mod!` (`on_click`, `on_render`, `on_return`, `on_startup`)
2. **Rust Layer** -- `MatchEvent` trait with `handle_actions`, `handle_timer`, etc.

## Splash Inline Event Handlers

### on_click -- Button/widget click

```splash
add_button := Button{
    text: "Add"
    on_click: ||{
        let text = ui.todo_input.text()
        if text != "" {
            add_todo(text)
            ui.todo_input.set_text("")
        }
    }
}
```

### on_render -- Dynamic rendering

```splash
main_view := View{
    width: Fill height: Fill
    on_render: ||{
        counter_label := Label{
            text: "Count: " + state.counter
        }
    }
}
```

### on_return -- TextInput enter key

```splash
todo_input := TextInput{
    empty_text: "What needs to be done?"
    on_return: || ui.add_button.on_click()
}
```

### on_startup -- App startup

```splash
ui: Root{
    on_startup: ||{
        ui.main_view.render()
    }
}
```

## Rust Event Handling -- MatchEvent Trait

```rust
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

## Widget Action API

### Button
```rust
.clicked(actions) -> bool           // Was clicked
.pressed(actions) -> bool           // Was pressed down
```

### TextInput
```rust
.changed(actions) -> Option<String>  // Text changed
.returned(actions) -> Option<(String, KeyModifiers)>  // Enter pressed
```

### CheckBox
```rust
.changed(actions) -> Option<bool>    // Toggled
```

## script_eval! Macro

```rust
script_eval!(cx, {
    mod.state.counter += 1
    ui.main_view.render()
});
```

## script_apply_eval! Macro

```rust
script_apply_eval!(cx, item, {
    height: #(height)
    draw_bg +: { color: #(bg_color) }
});
```

## Event Flow Diagram

```
User Input -> Platform Event Loop -> AppMain::handle_event
    |                                        |
    +--> self.match_event(cx, event)         |
    |       |                                |
    |       +--> MatchEvent::handle_actions  |
    |       |       |                        |
    |       |       +--> self.ui.button(...).clicked() |
    |       |       +--> script_eval! -> Splash state -> render |
    |       |                                |
    +--> self.ui.handle_event()              |
            |                                |
            +--> Widget tree event propagation
            +--> Splash on_click handlers
```
