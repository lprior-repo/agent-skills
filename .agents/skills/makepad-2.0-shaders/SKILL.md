---
name: makepad-2.0-shaders
description: |
  CRITICAL: Use for Makepad 2.0 shaders and draw_bg. Triggers on:
  makepad shaders, makepad draw_bg, makepad pixel shader, Sdf2d,
  makepad draw, makepad GPU, makepad instance, makepad uniform,
  着色器, GPU, 渲染, 实例变量,  uniform
---

# Makepad 2.0 Shaders Skill

## Overview

Makepad uses GPU shaders for all rendering. Every visual element is drawn using either built-in shaders or custom `draw_bg` with Sdf2d.

## Draw Shader Properties

### Built-in draw_bg

```splash
View{
    draw_bg +: {
        color: #2a2a3d
        border_radius: 8.0
    }
}
```

### Custom draw_bg with Sdf2d

```splash
View{
    width: 100 height: 100
    draw_bg +: {
        instance rotation: 0.0
        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let cx = self.rect_size.x * 0.5
            let cy = self.rect_size.y * 0.5
            let r = min(cx, cy) * 0.8
            sdf.circle(cx, cy, r)
            sdf.fill(#4488ff)
            return sdf.result
        }
    }
}
```

## Sdf2d Reference

### Shapes
```splash
sdf.circle(cx, cy, r)
sdf.rect(x, y, w, h)
sdf.movable_triangle(a, b, c)
sdf.line(x1, y1, x2, y2, thickness)
```

### Operations
```splash
sdf.fill(color)           // Fill shape
sdf.stroke(color, width)  // Stroke outline
sdf.union()               // Combine shapes
sdf.intersect()           // Intersection
sdf.subtract()            // Cut out
```

## Instance Variables

Instance variables are per-widget GPU data that can be animated:

```splash
draw_bg +: {
    instance hover: 0.0
    instance rotation: 0.0
    pixel: fn() {
        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
        // Use self.hover and self.rotation
        return sdf.result
    }
}
```

## Common Patterns

### Rounded Rectangle
```splash
draw_bg +: {
    instance hover: 0.0
    color: mix(#2a2a3d, #3a3a5d, self.hover)
    border_radius: 8.0
}
```

### Circle
```splash
draw_bg +: {
    pixel: fn() {
        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
        let r = min(self.rect_size.x, self.rect_size.y) * 0.5
        sdf.circle(self.rect_size.x * 0.5, self.rect_size.y * 0.5, r)
        sdf.fill(#4488ff)
        return sdf.result
    }
}
```

## Best Practices

1. **Use instance variables for animation** -- GPU interpolates
2. **Minimize shader complexity** -- Complex pixel shaders are slow
3. **Use built-in when possible** -- `draw_bg.color` is optimized
4. **Cache Sdf2d results** -- When shapes don't change
