# Style attributes
This page is dedicated to all the attributes usable in the `style_expr!` rust macro.
Some attributes are a more complex struct. Their names are then linked and can be found in the structs section.

## General
All attributes can have common values, such as:
- `none` - is evaluated based on the attribute. For numbers this means 0.
- `auto` - falls back to `DEFAULT_STYLE` provided by MVEngine when no special use is implemented. For example, `width: auto`  will be the contents width.
- `inherit` - copies the value from the parent if there is one.
- `unset` - falls back to `DEFAULT_STYLE` provided by MVEngine.

### x: i32
The `x` attribute only affects elements when `position: absolute`.

### y: i32
The `y` attribute only affects elements when `position: absolute`.

### width: i32
The `width` attribute specifies the elements width.

### height: i32
The `height` attribute specifies the elements width.

### padding: SideStyle
The `padding` struct sets the inner distance to the content.

### margin: SideStyle
The `margin` struct sets the outer distance to the content.

### origin: Origin
The `origin` attribute specifies where the `x` and `y` attributes refer to. For example:
```
<Div style="origin: bottom_right; position: absolute; x: 100%;"/>
```
This code will place the div on the right edge of the parent or the window.
Values:
- `bottom_left`
- `bottom_right`
- `top_left`
- `top_right`
- `center`

### position: Position
The `position` attribute specifies if the element is positioned by the ui system or manually by the user using the `x` and `y` attributes. Values:
- `relative` - by the ui
- `absolute` - by the user

### direction: Direction
The `direction` attribute specifies how the content flows inside a container.
Values:
- `vertical`
- `horizontal`

### child_align_x/child_align_y: ChildAlign
The `child_align_x` attribute specifies how elements are aligned on the x/y-axis. The behavior depends on the `direction`.
Values:
- `start`
- `middle`
- `end`

The follwing code will center a div:
```
<Div style="width: 100; height: 100%; child_align_x: middle; child_align_y: middle;">
    <Div/>
</Div>
```

### background: ShapeStyle
Specifies how the background of an element looks like.
For more info look at ShapeStyle struct.

### border: ShapeStyle
Specifies how the border of an element looks like.
For more info look at ShapeStyle struct.

### text: TextStyle
--> TextStyle

### transform: TransformStyle
--> TransformStyle

### overflow_x/overflow_y: Overflow
Specifies how large content is handled.
Values:
- `always` - will always display a scroll bar.
- `never` - will never display a scroll bar and the content will be cut of.
- `normal` - The scrollbar appears only when there is content overflow.

### scrollbar: ScrollBarStyle
--> ScrollBarStyle