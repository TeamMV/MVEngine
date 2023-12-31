package dev.mv.engine.files.os;

import dev.mv.engine.files.Directory;
import dev.mv.engine.utils.Utils;

import java.io.File;

public class OsxDirectory extends Directory {

    public OsxDirectory(String name) {
        super(name);
    }

    @Override
    protected File getFolder() {
        File dir = new File(Utils.getPath(System.getProperty("user.home"), "Library", "Application Support", "." + getName()));
        if (!dir.exists()) {
            dir.mkdirs();
        }
        if (!dir.isDirectory()) {
            dir.delete();
            dir.mkdirs();
        }
        return dir;
    }

}
