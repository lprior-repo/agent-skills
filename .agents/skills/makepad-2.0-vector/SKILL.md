---
name: makepad-2.0-vector
description: |
  CRITICAL: Use for Makepad 2.0 vector graphics and tweens. Triggers on:
  makepad vector, makepad SVG, makepad gradient, makepad tween,
  makepad path, makepad shapes, makepad animation,
  矢量, SVG, 渐变, 路径, 动画, 形状
---

# Makepad 2.0 Vector & Tween Skill

## Overview

Makepad 2.0 has two animation systems:
1. **Animator** - for widget shader `instance` variables (hover, focus states)
2. **Tween** - for vector/SVG property animations (paths, fills, opacity)

## Tween Syntax

Tweens are property values that animate over time:

```splash
draw_bg +: {
    opacity: Tween{ from: 0.0 to: 1.0 duration: 0.5 }
    fill: Tween{ from: #f00 to: #0f0 duration: 1.0 }
}
```

## Tween Properties

### opacity
```splash
draw_bg +: {
    opacity: Tween{ from: 0.0 to: 1.0 duration: 0.3 }
}
```

### fill / stroke
```splash
draw_bg +: {
    fill: Tween{ from: #2a2a3d to: #3a3a5d duration: 0.2 }
}
```

### cx, cy (center position)
```splash
draw_bg +: {
    cx: Tween{ from: 0.0 to: 100.0 duration: 0.5 }
    cy: Tween{ from: 0.0 to: 50.0 duration: 0.5 }
}
```

### Scale
```splash
draw_bg +: {
    scale_x: Tween{ from: 1.0 to: 1.5 duration: 0.3 }
    scale_y: Tween{ from: 1.0 to: 1.5 duration: 0.3 }
}
```

## Vector Graphics

### DrawField with SVG-like content

```splash
View{
    width: 100 height: 100
    draw_bg +: {
        // Use as background for vector content
        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            // Draw SVG-like content
            return sdf.result
        }
    }
}
```

### Gradient Fill

```splash
draw_bg +: {
    fill: LinearGradient{
        start: #f00
        end: #00f
        angle: 45.0
    }
}
```

### SVG Path Animation

```splash
let path = "M10,10 L90,90 L10,90 Z"
draw_bg +: {
    path: Tween{ from: path to: "M10,10 L90,10 L90,90 Z" duration: 1.0 }
}
```

## Tween Configuration

### loop_
```splash
opacity: Tween{ from: 0.0 to: 1.0 duration: 1.0 loop_: true }
```

### delay
```splash
opacity: Tween{ from: 0.0 to: 1.0 duration: 0.5 delay: 0.5 }
```

### ease
```splash
opacity: Tween{ from: 0.0 to: 1.0 duration: 0.5 ease: OutCubic }
```

## Animator vs Tween

| Feature | Animator | Tween |
|---------|----------|-------|
| Target | Widget instance vars | Vector/SVG properties |
| Syntax | `animator: Animator{...}` | `prop: Tween{...}` |
| Trigger | State change | Automatic |
| Loop | `Loop{}` in from | `loop_: true` |
| Best for | UI interactions | SVG animations |

## Common Patterns

### Fade In Animation
```splash
View{
    draw_bg +: {
        opacity: 0.0
    }
    // On mount/visible:
    draw_bg +: {
        opacity: Tween{ from: 0.0 to: 1.0 duration: 0.3 }
    }
}
```

### Pulse Animation
```splash
draw_bg +: {
    scale_x: Tween{ from: 1.0 to: 1.1 duration: 0.5 loop_: true }
    scale_y: Tween{ from: 1.0 to: 1.1 duration: 0.5 loop_: true }
}
```

### Color Transition
```splash
draw_bg +: {
    fill: Tween{ from: #2a2a3d to: #4488ff duration: 0.5 }
}
```
