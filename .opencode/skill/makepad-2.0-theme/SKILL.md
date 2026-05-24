---
name: makepad-2.0-theme
description: |
  CRITICAL: Use for Makepad 2.0 theming and colors. Triggers on:
  makepad theme, makepad colors, makepad dark mode, makepad typography,
  makepad fonts, makepad styles, makepad design system,
  主题, 颜色, 暗色模式, 字体, 样式
---

# Makepad 2.0 Theme Skill

## Overview

Makepad 2.0 provides a theming system with built-in light/dark themes and custom theme support.

## Built-in Themes

```rust
// In App::run
crate::makepad_widgets::theme_mod(vm);
script_eval!(vm, {
    mod.theme = mod.themes.dark  // or mod.themes.light
});
```

## Theme Properties

### Colors
```splash
draw_bg.color: theme.color_bg_app
draw_text.color: theme.color_text
draw_bg.color: theme.color_hover
```

### Typography
```splash
draw_text.text_style.font_size: theme.font_size_text
draw_text.text_style.font_id: theme.font_text
```

### Spacing
```splash
padding: theme.space_1
margin: theme.space_2
```

### Border Radius
```splash
draw_bg.border_radius: theme.border_radius
```

## Theme Reference

### Dark Theme Colors
```rust
mod.themes.dark = {
    color_bg_app: #1a1a2e
    color_bg_window: #16162a
    color_text: #e8e8ff
    color_text_dim: #8888aa
    color_hover: #2a2a4a
    color_active: #3a3a5a
    color_selected: #4a4a6a
    color_border: #2a2a4a
}
```

### Light Theme Colors
```rust
mod.themes.light = {
    color_bg_app: #f0f0f5
    color_bg_window: #ffffff
    color_text: #1a1a2e
    color_text_dim: #6a6a8a
    color_hover: #e0e0e8
    color_active: #d0d0d8
    color_selected: #c0c0d0
    color_border: #d0d0d8
}
```

## Custom Theme

```splash
mod.theme = {
    color_bg_app: #x1a1a2e
    color_bg_window: #x16162a
    color_text: #xe8e8ff
    color_text_dim: #x8888aa
    color_hover: #x2a2a4a
    // ... all required fields
}
```

## Typography Scale

| Token | Size |
|-------|------|
| `theme.font_size_h1` | 32 |
| `theme.font_size_h2` | 24 |
| `theme.font_size_h3` | 20 |
| `theme.font_size_text` | 14 |
| `theme.font_size_caption` | 12 |
| `theme.font_size_small` | 10 |

## Dark/Light Mode Switching

```rust
impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(toggle_theme)).clicked(actions) {
            let new_theme = if is_dark {
                mod.themes.light
            } else {
                mod.themes.dark
            };
            is_dark = !is_dark;
            script_eval!(cx, {
                mod.theme = #(new_theme)
            });
        }
    }
}
```

## Best Practices

1. **Use theme tokens, not hardcoded colors** -- Enables theme switching
2. **Use `mod.themes.dark/light`** -- Built-in themes work out of the box
3. **Define all required fields** -- Custom themes must have all fields
4. **Use theme spacing for consistent layout** -- `theme.space_1` etc.
