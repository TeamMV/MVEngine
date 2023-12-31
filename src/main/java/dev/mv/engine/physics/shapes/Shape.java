package dev.mv.engine.physics.shapes;

import dev.mv.engine.physics.Physics;
import org.joml.Vector2f;

public abstract class Shape {
    protected final BoundingBox boundingBox = new BoundingBox();
    protected float x, y;
    protected float rotation;
    protected Vector2f center;
    protected Physics physics;

    protected Shape(Physics physics, Vector2f center) {
        this.physics = physics;
        this.center = center;
        x = center.x;
        y = center.y;
    }

    protected Shape(Physics physics, float x, float y) {
        this.physics = physics;
        this.x = x;
        this.y = y;
        center = new Vector2f(x, y);
    }

    protected Shape(Physics physics, float x, float y, Vector2f center) {
        this.physics = physics;
        this.x = x;
        this.y = y;
        this.center = center;
    }

    public abstract boolean isCollidingWith(Shape shape);

    public abstract boolean equalsType(Shape shape);

    public abstract void scale(float factor);

    /**
     * Update the bounding box (protected variable), using a really fast mathematical approximation
     * for a rectangle, which fully encloses the shape, regardless of rotation.
     */
    public abstract void updateBoundingBox();

    public void moveX(float amount) {
        this.x += amount;
        this.center.x += amount;
        updateBoundingBox();
    }

    public void moveY(float amount) {
        this.y += amount;
        this.center.y += amount;
        updateBoundingBox();
    }

    public float getX() {
        return x;
    }

    public void setX(float x) {
        moveX(x - this.x);
    }

    public float getY() {
        return y;
    }

    public void setY(float y) {
        moveY(y - this.y);
    }

    public float getCenterX() {
        return center.x;
    }

    public void setCenterX(float x) {
        moveX(x - center.x);
    }

    public float getCenterY() {
        return center.y;
    }

    public void setCenterY(float y) {
        moveY(y - center.y);
    }

    public float getRotation() {
        return rotation;
    }

    public void setRotation(float rotation) {
        this.rotation = rotation % 360;
        if (this.rotation < 0) this.rotation += 360;
    }

    /**
     * Get a really fast mathematical approximation of a rectangle,
     * which will fully enclose the shape, regardless of rotation.
     * @return the bounding box.
     */
    public BoundingBox getBoundingBox() {
        return boundingBox;
    }

    public static class BoundingBox {

        float x, y, s;

        private BoundingBox() {
        }

        public boolean isColliding(BoundingBox b) {
            return x < b.x + b.s &&
                x + s > b.x &&
                y < b.y + b.s &&
                y + s> b.y;
        }
    }
}
