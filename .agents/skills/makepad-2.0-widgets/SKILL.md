---
name: makepad-2.0-widgets
description: |
  CRITICAL: Use for Makepad 2.0 widget catalog. Triggers on:
  makepad widgets, makepad View, makepad Button, makepad Label,
  makepad TextInput, makepad CheckBox, makepad DropDown, makepad Slider,
  makepad widget list, makepad components, makepad UI components,
  组件, 控件, 视图, 按钮, 标签, 输入框, 复选框, 下拉框, 滑块
---

# Makepad 2.0 Widget Catalog

## Container Widgets

### View
Basic container for layout.
```splash
View{
    width: Fill height: Fit
    flow: Down spacing: 10
}
```

### SolidView
Container with solid background.
```splash
SolidView{
    width: Fill height: 100
    draw_bg.color: #2a2a3d
}
```

### RoundedView
Container with rounded corners.
```splash
RoundedView{
    width: Fill height: Fit
    draw_bg.color: #2a2a3d
    draw_bg.border_radius: 8.0
}
```

### ScrollYView
Vertical scrolling container.
```splash
ScrollYView{
    width: Fill height: Fill
    flow: Down
    // Content
}
```

## Interactive Widgets

### Button
Basic button with text.
```splash
Button{ text: "Click me" }
```

### ButtonFlat
Flat button variant.
```splash
ButtonFlat{ text: "Flat" }
```

### ButtonFlatter
Flatter button variant.
```splash
ButtonFlatter{ text: "Flatter" }
```

### CheckBox
Checkbox with label.
```splash
CheckBox{ text: "Option" }
```

### Toggle
Toggle switch.
```splash
Toggle{}
```

### RadioButton
Radio button for groups.
```splash
RadioButton{ text: "Option A" }
```

### Slider
Horizontal slider.
```splash
Slider{ width: 200 }
```

### TextInput
Text input field.
```splash
TextInput{
    empty_text: "Enter text..."
}
```

### DropDown
Dropdown select.
```splash
DropDown{
    text: "Select option"
}
```

## Display Widgets

### Label
Text display.
```splash
Label{
    text: "Hello World"
    draw_text.color: #fff
}
```

### H1, H2, H3, H4
Heading text.
```splash
H1{ text: "Title" }
H2{ text: "Subtitle" }
```

### P
Paragraph text.
```splash
P{ text: "Long text..." }
```

## Layout Widgets

### ScrollXView
Horizontal scrolling.
```splash
ScrollXView{ width: Fill height: 60 }
```

### ScrollXYView
Both-axis scrolling.
```splash
ScrollXYView{ width: Fill height: Fill }
```

### Splitter
Resizable split panel.
```splash
Splitter{ axis: SplitterAxis.Horizontal }
```

### Filler
Spacer widget.
```splash
Filler{}
```

### Hr
Horizontal rule/divider.
```splash
Hr{ draw_bg.color: #2a2a4a }
```

## Navigation

### LinkLabel
Clickable link.
```splash
LinkLabel{ text: "Click here" }
```

## Containers with Special Behavior

### PortalList
Virtualized list for large datasets.
```splash
PortalList{ width: Fill height: Fill }
```

### TabBar
Tab navigation.
```splash
TabBar{}
```

### DesktopWindow
Desktop window frame.
```splash
DesktopWindow{}
```

### Window
Basic window.
```splash
Window{}
```

## Widget Properties Reference

### Common Properties
| Property | Type | Description |
|---------|------|-------------|
| `width` | Size | Width (Fill/Fit/pixels) |
| `height` | Size | Height |
| `flow` | Flow | Layout direction |
| `padding` | Inset | Inner spacing |
| `margin` | Inset | Outer spacing |
| `align` | Align | Child alignment |
| `spacing` | number | Gap between children |

### Draw Properties
| Property | Type | Description |
|---------|------|-------------|
| `draw_bg` | DrawProps | Background styling |
| `draw_text` | DrawText | Text styling |
| `show_bg` | bool | Show background |
| `visible` | bool | Visibility |

### Animation Support
| Widget | Animator | Tween |
|--------|----------|-------|
| View | Yes | Yes |
| SolidView | Yes | Yes |
| RoundedView | Yes | Yes |
| Button | Yes | Yes |
| ButtonFlat | Yes | Yes |
| ButtonFlatter | Yes | Yes |
| CheckBox | Yes | Yes |
| Toggle | Yes | Yes |
| RadioButton | Yes | Yes |
| LinkLabel | Yes | Yes |
| TextInput | Yes | Yes |
| Label | **No** | Yes |
| H1-H4 | **No** | Yes |
| P | **No** | Yes |
| Slider | **No** | Yes |
| DropDown | **No** | Yes |
| Splitter | **No** | Yes |
| Hr | **No** | Yes |
| Filler | **No** | **No** |

**Note:** Widgets marked "No" for Animator do NOT support `animator: Animator{...}`. Adding it has no effect.
