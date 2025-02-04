# Shape Manipulation and Transformation Tool

## Overview
This tool provides a structured approach to manipulate and transform basic geometric shapes using transformations such as translation, scaling, rotation, and more. The system supports combining shapes and exporting the final result.

---

## Data Structures

### **Vec2**
Represents a 2D vector.
- `x (f32)`: X component
- `y (f32)`: Y component

### **Transform**
Represents the transformation of a shape.
- `t (Vec2)`: Translation vector
- `s (Vec2)`: Scale vector
- `o (Vec2)`: Origin point for transformations
- `r (f32)`: Rotation angle (in degrees)

---

## Primitives

### **Triangle (`tri`)**
A triangle defined by three points and a transformation.
- `a (Vec2)`: First vertex
- `b (Vec2)`: Second vertex
- `c (Vec2)`: Third vertex
- `t (Transform)`: Transformation

### **Rectangle (`rect`)**
A rectangle defined by two points and additional parameters.
- `a (Vec2)`: First corner point
- `b (Vec2)`: Second corner point
- `x (i32)`: X-coordinate of the top-left corner
- `y (i32)`: Y-coordinate of the top-left corner
- `w (i32)`: Width of the rectangle
- `h (i32)`: Height of the rectangle
- `t (Transform)`: Transformation

### **Arc (`arc`)**
A circular arc defined by its center, radius, and angle.
- `c (Vec2)`: Center of the arc
- `r (i32)`: Radius of the arc
- `a (f32)`: Angle of the arc (in degrees)
- `tc (i32)`: Number of triangles used to approximate the arc
- `t (Transform)`: Transformation

---

## Syntax
### **Shape Selection**
Use the `>` symbol followed by the shape type to select a shape.

Example:
```plaintext
>circle
```

---

## Functions

### **1. `transform (Transform)`**
Applies a specified transformation to the currently selected shape.

### **2. `apply`**
Applies the current shape's transformation to its triangles, updating its geometry.

### **3. `recenter`**
Sets the transformation origin to the center of the currently selected shape.

### **4. `combine (Shape)`**
Adds another shape to the currently selected shape, merging their geometries.

### **5. `export`**
Exports the current shape and exits the program. This step is required to finalize the shape manipulation process.

---

## Example Usage
```plaintext
circle = arc[c[x100y100]r50a45];
>circle;
transform t[t[x50y0]r45.3];
recenter;
export;
```

---

## Notes
- Use transformations to modify the shape's position, size, rotation, and origin.
- Ensure to use `export` to save the final shape.
- Combining shapes allows for the creation of complex geometries.

