---
name: makepad-2.0-performance
description: |
  CRITICAL: Use for Makepad 2.0 performance optimization. Triggers on:
  makepad performance, makepad profiling, makepad optimization, GC,
  draw batching, makepad benchmark, makepad speed, makepad memory,
  性能, 优化, 内存, GC, 渲染性能
---

# Makepad 2.0 Performance Skill

## Overview

Makepad 2.0 has a sophisticated GPU rendering pipeline. This skill covers performance optimization strategies, profiling tools, and common performance pitfalls.

## GPU Rendering Architecture

Makepad renders to GPU every frame. Key concepts:

- **Draw Calls** -- Each distinct shader program + geometry = one draw call
- **Instance Batching** -- Same shader, different instance data = combined into one draw call
- **Redraw Regions** -- Only dirty regions are re-rendered

## Performance Rules

### 1. Minimize Draw Calls

**Bad:** Many small Views with different backgrounds
```splash
View{ draw_bg.color: #f00 }
View{ draw_bg.color: #0f0 }
View{ draw_bg.color: #00f }
```

**Good:** Batch similar elements
```splash
View{
    flow: Right
    ColoredCard{ color: #f00 }
    ColoredCard{ color: #0f0 }
    ColoredCard{ color: #00f }
}
```

### 2. Use `new_batch: true` for Static Content

```splash
ScrollYView{
    width: Fill height: Fill
    new_batch: true
    // Static content that doesn't change
}
```

### 3. Avoid Redraws

Only call `redraw(cx)` when something actually changed:
```rust
// Bad - redraws every frame
fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
    self.redraw(cx);
}

// Good - only when state changes
if self.value_changed {
    self.value_changed = false;
    self.redraw(cx);
}
```

### 4. Use `height: Fit` Where Appropriate

`height: Fill` inside `height: Fill` parent = infinite growth = bad performance.

## Profiling

### Enable Debug Draw Call Count

```rust
// In Cargo.toml
[profile.dev]
opt-level = 0

// Or via environment
MAKPAD_DEBUG_DRAW_CALLS=1 cargo run
```

### Memory Profiling

Makepad uses reference counting for GC. Large object graphs cause GC pressure.

**Tips:**
- Break up large state into smaller chunks
- Use `Scope::new()` for temporary scopes
- Avoid circular references

## Common Performance Issues

### Symptom: High CPU with simple UI
- **Cause:** Too many redraws
- **Fix:** Only call `redraw(cx)` when state changes

### Symptom: Laggy scrolling
- **Cause:** Too many draw calls in scroll view
- **Fix:** Use `new_batch: true`, simplify children

### Symptom: High memory usage
- **Cause:** Large state object, reference cycles
- **Fix:** Break into smaller pieces, use weak references

### Symptom: Slow startup
- **Cause:** Too many widgets created at startup
- **Fix:** Use lazy initialization, only create visible widgets

## Benchmarking

```rust
use std::time::Instant;

let start = Instant::now();
// ... render code ...
let elapsed = start.elapsed();
println!("Render time: {:?}", elapsed);
```

## Best Practices

1. **Batch similar draw calls** -- Use `new_batch: true`
2. **Only redraw when needed** -- Track dirty state
3. **Use appropriate heights** -- `Fit` when content-bounded
4. **Limit widget depth** -- Deep nesting = slower layout
5. **Cache computed values** -- Don't recalculate every frame
6. **Use `#[rust]` for expensive state** -- Keeps it off GPU
