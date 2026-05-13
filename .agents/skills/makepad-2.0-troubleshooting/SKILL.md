---
name: makepad-2.0-troubleshooting
description: |
  CRITICAL: Use for Makepad 2.0 errors and debugging. Triggers on:
  makepad error, makepad bug, makepad not working, makepad debugging,
  makepad FAQ, makepad help, widget not showing, layout broken,
  错误, 调试, 问题, 故障排除, FAQ
---

# Makepad 2.0 Troubleshooting Skill

## Common Issues

### Widget Not Showing / Invisible UI

**Most common cause: Missing `height: Fit` on containers**

```splash
// WRONG - height defaults to Fill, creates circular dependency = 0px
View{
    flow: Down
    Label{text: "Invisible!"}
}

// CORRECT
View{
    height: Fit
    flow: Down
    Label{text: "Visible!"}
}
```

### Widget Access Errors ("widget not found")

**Cause: Using `:` instead of `:=` for named widgets**

```splash
// WRONG - 'my_button' is not addressable
my_button := Button{ text: "Click" }  // No!

// CORRECT
my_button := Button{ text: "Click" }  // Yes, := makes it addressable
```

**Cause: Widget not registered**

```rust
fn run(vm: &mut ScriptVm) -> Self {
    crate::makepad_widgets::script_mod(vm);  // Missing!
    App::from_script_mod(vm, self::script_mod)
}
```

### Property Not Found Errors

**Cause: Forgetting `+:` merge operator**

```splash
// WRONG - replaces ALL draw_bg properties
draw_bg: { color: #f00 }

// CORRECT - only changes color
draw_bg +: { color: #f00 }
```

### Animation Not Working

**Cause: Animator on unsupported widget**

Only View, Button, CheckBox etc. support Animator. Label does NOT.

```splash
// WRONG - Label doesn't support animator
Label{
    animator: Animator{ hover: {...} }
    text: "Won't animate"
}

// CORRECT - wrap Label in View with animator
View{
    animator: Animator{ hover: {...} }
    Label{text: "Will animate!"}
}
```

### Hex Color Parsing Error

**Cause: Hex color contains 'e'**

```splash
// WRONG - 'e' in 2ecc71 is interpreted as exponent
color: #2ecc71

// CORRECT - use #x prefix
color: #x2ecc71
```

### Render Not Updating

**Cause: Not calling render()**

```splash
// In script_mod!, must call render to update
on_click: || {
    state.counter += 1
    ui.counter_label.render()  // This is needed!
}
```

### Type Mismatch

**Cause: Wrong type for property**

```splash
// WRONG - width expects Size, not string
width: "Fill"

// CORRECT
width: Fill
```

## Debug Techniques

### Print Debugging

```splash
std.println("Value: " + value.to_string())
```

### Check Widget IDs

```rust
// In Rust
println!("Button area: {:?}", button.area());
```

### Enable Debug Output

```bash
RUST_LOG=debug cargo run
```

## Error Messages

| Error | Cause | Fix |
|--------|-------|-----|
| "variable X not found" | Using `:` instead of `:=` | Use `:=` for named widgets |
| "widget not registered" | Missing `script_mod(vm)` | Call `crate::makepad_widgets::script_mod(vm)` |
| "height: Fill in Fit = 0px" | Circular dependency | Add `height: Fit` to container |
| "property not found" | Wrong property name | Check API docs |
| "expected X got Y" | Type mismatch | Use correct type |

## Still Stuck?

1. Check the layout skill (`height: Fit` rule)
2. Check the DSL skill (`:=` vs `:` rule)
3. Check the events skill (render() calling)
4. Simplify to minimal repro case
