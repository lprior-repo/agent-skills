---
name: makepad-2.0-layout
description: |
  CRITICAL: Use for Makepad 2.0 layout system. Triggers on:
  makepad layout, makepad width, makepad height, makepad flex, makepad flow,
  makepad padding, makepad margin, makepad spacing, makepad align, makepad sizing,
  Fill, Fit, Inset, Flow.Down, Flow.Right, ScrollXView, ScrollYView,
  布局, 对齐, 间距, 填充, 排版, 滚动视图, 尺寸, 宽度, 高度
---

# Makepad 2.0 Layout System

Makepad uses a **layout turtle** system -- not CSS flexbox, not CSS grid. The turtle walks through children one by one.

## Walk System (Widget Sizing)

### width / height

| Syntax | Meaning |
|--------|---------|
| `width: Fill` | Fill all remaining horizontal space (default) |
| `width: Fit` | Shrink to fit content |
| `width: 200` | Fixed 200 pixels |
| `width: Fill{min: 100 max: 500}` | Fill with constraints |

### CRITICAL: height: Fit on Containers

**The number one layout bug in Makepad.**

Default height is `Fill`. When `Fill` is inside a `Fit` parent, it creates circular dependency = **0 pixels** = invisible UI.

**Rule: ALWAYS set `height: Fit` on every View unless parent has fixed or Fill height.**

## Layout System (Child Arrangement)

### flow (Direction)

| Syntax | Meaning | CSS Equivalent |
|--------|---------|----------------|
| `flow: Right` | Left-to-right, single line | flex-direction: row |
| `flow: Down` | Top-to-bottom, single column | flex-direction: column |
| `flow: Overlay` | Stack children on top | position: absolute |
| `flow: Flow.Right{wrap: true}` | Left-to-right with wrapping | flex-wrap: wrap |

### spacing, padding, align

```
View{
    flow: Down spacing: 12
    padding: 20
    align: Center
}
```

## Scrollable Containers

| Widget | Scroll Direction |
|--------|-----------------|
| `ScrollYView` | Vertical only |
| `ScrollXView` | Horizontal only |
| `ScrollXYView` | Both axes |

**ScrollYView uses `height: Fill` (not Fit) because it needs a fixed viewport.**

## Common Layout Patterns

### Vertical Page Layout
```
View{
    width: Fill height: Fit
    flow: Down spacing: 16 padding: 20
    Label{text: "Title"}
    Label{text: "Body"}
}
```

### Horizontal Toolbar
```
SolidView{
    width: Fill height: 44
    flow: Right spacing: 8
    align: Align{y: 0.5}
    ButtonFlatter{text: "File"}
    ButtonFlatter{text: "Edit"}
    Filler{}
    ButtonFlat{text: "Run"}
}
```

## Critical Rules Summary

1. **height: Fit on ALL containers** - forgetting = invisible UI
2. **width: Fill on root container** - never fixed pixel width on root
3. **new_batch: true when View has show_bg AND text children** - prevents text behind background
4. **Do not use Filler next to width: Fill siblings** - causes 50/50 split
5. **ScrollYView uses height: Fill** - needs fixed viewport
