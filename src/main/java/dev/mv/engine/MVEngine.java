package dev.mv.engine;

import dev.mv.engine.async.AsyncManager;
import dev.mv.engine.audio.Audio;
import dev.mv.engine.exceptions.Exceptions;
import dev.mv.engine.exceptions.handle.ExceptionHandler;
import dev.mv.engine.game.Game;
import dev.mv.engine.input.Input;
import dev.mv.engine.input.InputCollector;
import dev.mv.engine.input.InputProcessor;
import dev.mv.engine.input.processors.MainInputProcessor;
import dev.mv.engine.physics.Physics;
import dev.mv.engine.render.WindowCreateInfo;
import dev.mv.engine.render.opengl.OpenGLWindow;
import dev.mv.engine.render.shared.Window;
import dev.mv.engine.render.shared.font.BitmapFont;
import dev.mv.engine.resources.*;
import dev.mv.engine.test.Test;
import dev.mv.engine.utils.Infallible;
import dev.mv.engine.utils.logger.Logger;
import dev.mv.engine.utils.misc.Version;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.glfw.GLFWErrorCallback;
import org.lwjgl.opengl.GL;

import java.util.ArrayList;
import java.util.List;

import static org.lwjgl.glfw.GLFW.glfwInit;
import static org.lwjgl.glfw.GLFW.glfwTerminate;

public class MVEngine implements AutoCloseable {

    private static volatile MVEngine instance;

    public String VERSION_STR = "v0.5.3";
    public Version VERSION = Version.parse(VERSION_STR);
    private ApplicationConfig.RenderingAPI renderingApi = ApplicationConfig.RenderingAPI.OPENGL;
    private ApplicationConfig applicationConfig;
    private InputCollector inputCollector;
    private ExceptionHandler exceptionHandler;
    private Game game;
    private final List<Looper> loopers;
    private Physics physics;
    private Audio audio;
    private final ResourceLoader resourceLoader;
    private ResourceManager resourceManager;

    private MVEngine() {
        exceptionHandler = ExceptionHandler.Default.INSTANCE;
        loopers = new ArrayList<>();
        resourceLoader = new ResourceLoader("mvengine");
        resourceManager = new ResourceManager();
        Logger.setLoggerOutput((s, logLevel) -> {
            if (logLevel == Logger.LogLevel.WARN || logLevel == Logger.LogLevel.ERROR) {
                System.err.print(s);
            } else {
                System.out.print(s);
            }
        });
    }

    public static MVEngine instance() {
        MVEngine result = instance;
        if (result != null) {
            return result;
        }
        synchronized (MVEngine.class) {
            if (instance == null) {
                throw new IllegalStateException("MVEngine not initialised");
            }
            return instance;
        }
    }

    public static synchronized MVEngine init() {
        return init(new ApplicationConfig());
    }

    public static synchronized MVEngine init(ApplicationConfig config) {
        if (instance != null) {
            throw new IllegalStateException("MVEngine already initialised");
        }

        if (config == null) {
            config = new ApplicationConfig();
        }
        instance = new MVEngine();
        Exceptions.readExceptionINI(MVEngine.class.getResourceAsStream("/assets/mvengine/exceptions.ini"));
        instance.audio = Audio.init(config.getSimultaneousAudioSources());
        boolean _2d = config.getDimension() == ApplicationConfig.GameDimension.ONLY_2D || config.getDimension() == ApplicationConfig.GameDimension.COMBINED;
        boolean _3d = config.getDimension() == ApplicationConfig.GameDimension.ONLY_3D || config.getDimension() == ApplicationConfig.GameDimension.COMBINED;
        if (_2d) {
            instance.physics = Physics.init();
        }

        //This isn't useless
        MainInputProcessor.getInstance();

        AsyncManager.init(config.getAmountAsyncWorkers());

        instance.applicationConfig = config;
        GLFWErrorCallback.createPrint(System.err).set();

        if (!glfwInit()) {
            Exceptions.send("GLFW_INIT");
        }

        if (config.getRenderingApi() == ApplicationConfig.RenderingAPI.VULKAN) {
            instance.renderingApi = ApplicationConfig.RenderingAPI.VULKAN;
        } else {
            instance.renderingApi = ApplicationConfig.RenderingAPI.OPENGL;
        }

        return instance;
    }

    public static void terminate() {
        instance().close();
    }

    public ApplicationConfig.RenderingAPI getRenderingApi() {
        return renderingApi;
    }

    public void rollbackRenderingApi() {
        if (renderingApi != ApplicationConfig.RenderingAPI.OPENGL) {
            renderingApi = ApplicationConfig.RenderingAPI.OPENGL;
        }
    }

    public Window createWindow(WindowCreateInfo info) {
        if (info == null) {
            info = new WindowCreateInfo();
        }

        if (renderingApi == ApplicationConfig.RenderingAPI.OPENGL) {
            return new OpenGLWindow(info);
        } else {
            throw new RuntimeException("Please use OpenGL as the rendering API, since it is the only one supported at this moment.");
        }
    }

    @Override
    public synchronized void close() {
        audio.terminate();
        if (physics != null) physics.terminate();
        glfwTerminate();
        instance = null;
    }

    public ApplicationConfig getApplicationConfig() {
        return applicationConfig;
    }

    public ExceptionHandler getExceptionHandler() {
        return exceptionHandler;
    }

    public void setExceptionHandler(ExceptionHandler handler) {
        exceptionHandler = handler;
    }

    public Game getGame() {
        return game;
    }

    public void setGame(Game game) {
        this.game = game;
    }

    public void registerLooper(Looper loopable) {
        loopers.add(loopable);
    }

    public List<Looper> getLoopers() {
        return loopers;
    }

    public Physics getPhysics() {
        return physics;
    }

    public Audio getAudio() {
        return audio;
    }

    public ResourceManager getResourceManager() {
        return resourceManager;
    }

    public void loadResources(ProgressAction action) {
        Env.setResourceReady();
        resourceLoader.markFont("default", ResourcePath.internal("/font/default/roboto.png"), ResourcePath.internal("/font/default/roboto.fnt"));
        resourceLoader.loadAll(action);
    }

    public void createResources() {
        R.font.get("mvengine.default").load();
    }

    public static @NotNull Infallible finish() {
        return Infallible.exit(0);
    }
}
