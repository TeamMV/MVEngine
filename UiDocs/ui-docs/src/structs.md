# Style Structs
This page is dedicated to structs used by the style attributes.

### SideStyle
Fields:
- `top: i32`
- `bottom: i32`
- `left: i32`
- `right: i32`

In the `style_expr!` rust macro, a SideStyle can be constructed from either 1, 2 or 4 components.
So the following is valid:
- `padding: 5cm;`
- `padding: 1cm, 2cm;`
- `padding: 1cm, 2cm, 3mm, 4mm;`

### ShapeStyle
Fields:
- `resource: BackgroundRes`
The resource field is used to specify the type of background used. `none` will display no background at all.
Values:
- - `color`
- - `texture`

- `color: Color`
The color field is used to set the background color.

- `texture: Drawable`
The drawable displayed on the background. This corresponds to the `<drawable>` R entry.

- `shape: Geometry`
The shape used for the background. This corresponds to the `<geometry>` R entry.

### TextStyle
Fields:
- `size: f32`
- `kerning: f32`
- `skew: f32`
- `stretch: Vec2f`
- `font: <font>` Font pulled from `<font>` R entry.
- `color: Color`
- `align_x: TextAlign` See: `child_align_x` style attribute
- `align_y: TextAlign` See: `child_align_y`

### TransformStyle
Fields:
- `translate: Vec2i`
- `scale: Vec2f`
- `rotate: f32`
- `origin: Origin` See: `origin` style attribute

### ScrollBarStyle
Fields:
- `track: ShapeStyle`
- `knob: ShapeStyle`
- `size: i32`