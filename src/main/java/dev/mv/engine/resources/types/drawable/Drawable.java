package dev.mv.engine.resources.types.drawable;

import dev.mv.engine.parsing.XMLParser;
import dev.mv.engine.resources.Resource;
import dev.mv.engine.parsing.Parser;
import dev.mv.engine.render.shared.DrawContext;
import dev.mv.engine.utils.BinaryFunction;
import dev.mv.engine.utils.Into;

import java.io.IOException;
import java.io.InputStream;
import java.util.function.*;

public abstract class Drawable extends Into {
    protected int cnvsW, cnvsH;
    protected String resId;

    public Drawable(int canvasWidth, int canvasHeight) {
        this.cnvsW = canvasWidth;
        this.cnvsH = canvasHeight;
    }

    public abstract void parse(Parser parser);

    public abstract void draw(DrawContext ctx, int x, int y, float rot, int ox, int oy);

    public void draw(DrawContext ctx, int x, int y, int width, int height, float rot, int ox, int oy) {
        setCnvsW(width);
        setCnvsH(height);
        draw(ctx, x, y, rot, ox, oy);
    }

    public int getCnvsW() {
        return cnvsW;
    }

    public void setCnvsW(int cnvsW) {
        this.cnvsW = cnvsW;
    }

    public int getCnvsH() {
        return cnvsH;
    }

    public void setCnvsH(int cnvsH) {
        this.cnvsH = cnvsH;
    }

    protected static class Transformations {//all take in width and height and return the value in px
        private BinaryOperator<Integer> transX, transY;
        private BinaryFunction<Integer, Float> scaleX, scaleY;
        private BinaryOperator<Integer> rotation;
        private BinaryOperator<Integer> originX, originY;

        private boolean wasOXNull = false;
        private boolean wasOYNull = false;

        public Transformations() {
            transX = (w, h) -> 0;
            transY = (w, h) -> 0;
            scaleX = (w, h) -> 1f;
            scaleY = (w, h) -> 1f;
            rotation = (w, h) -> 0;
            originX = (w, h) -> w / 2;
            originY = (w, h) -> h / 2;
        }

        public BinaryOperator<Integer> transX() {
            return transX;
        }

        public BinaryOperator<Integer> transY() {
            return transY;
        }

        public BinaryFunction<Integer, Float> scaleX() {
            return scaleX;
        }

        public BinaryFunction<Integer, Float> scaleY() {
            return scaleY;
        }

        public BinaryOperator<Integer> rotation() {
            return rotation;
        }

        public BinaryOperator<Integer> originX() {
            return originX;
        }

        public BinaryOperator<Integer> originY() {
            return originY;
        }

        public void setOriginsIfNull(BinaryOperator<Integer> originX, BinaryOperator<Integer> originY) {
            if (this.originX == null) { this.originX = originX; wasOXNull = true; }
            if (this.originY == null) { this.originY = originY; wasOYNull = true; }
        }

        public void restoreOriginsToNull() {
            if (wasOXNull) { originX = null; wasOXNull = false; }
            if (wasOYNull) { originY = null; wasOYNull = false; }
        }

        public void setTransX(BinaryOperator<Integer> transX) {
            this.transX = transX;
        }

        public void setTransY(BinaryOperator<Integer> transY) {
            this.transY = transY;
        }

        public void setScaleX(BinaryFunction<Integer, Float> scaleX) {
            this.scaleX = scaleX;
        }

        public void setScaleY(BinaryFunction<Integer, Float> scaleY) {
            this.scaleY = scaleY;
        }

        public void setRotation(BinaryOperator<Integer> rotation) {
            this.rotation = rotation;
        }

        public void setOriginX(BinaryOperator<Integer> originX) {
            this.originX = originX;
        }

        public void setOriginY(BinaryOperator<Integer> originY) {
            this.originY = originY;
        }
    }

    protected static class DrawOptions {
        private boolean filled = true;
        private BinaryOperator<Integer> strokeWidth = (w, h) -> 2;
        private Drawable border = null;

        public boolean filled() {
            return filled;
        }

        public BinaryOperator<Integer> strokeWidth() {
            return strokeWidth;
        }

        public Drawable border() {
            return border;
        }

        public void setFilled(boolean filled) {
            this.filled = filled;
        }

        public void setStrokeWidth(BinaryOperator<Integer> strokeWidth) {
            this.strokeWidth = strokeWidth;
        }

        public void setBorder(Drawable border) {
            this.border = border;
        }
    }

    public Drawable() {}

    public void load(InputStream inputStream, String resId) throws IOException {
        parse(new XMLParser(inputStream));
    }
}
