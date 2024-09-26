package org.example.gui.util;

import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import org.example.TcpClient;
import org.example.entity.User;
import org.example.gui.GuiApplication;
import org.example.gui.controller.ChatController;
import org.example.gui.controller.ChatMessageController;
import org.example.gui.controller.ConnectController;
import org.example.gui.controller.SeparatorController;
import org.example.protocol.Message;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.FileNotFoundException;
import java.io.IOException;
import java.net.URL;
import java.util.function.Consumer;

public class SceneLoaderUtil {
    private static final String MESSAGE_FXML = "/fxml/message.fxml";
    private static final String CONNECT_FXML = "/fxml/connect.fxml";
    private static final String CHAT_FXML = "/fxml/chat.fxml";
    private static final String SEPARATOR_FXML = "/fxml/separator.fxml";

    public static Parent loadSeparator(String roomName) throws IOException {
        return SceneLoaderUtil.loadScene(SEPARATOR_FXML, (controller) -> {
            ((SeparatorController) controller).getRoomNameLabel().setText(roomName);
        });
    }

    public static Parent loadChatScene(User user, TcpClient client) throws IOException {
        return SceneLoaderUtil.loadScene(CHAT_FXML, (controller) -> {
            ((ChatController) controller).setClient(client);
            ((ChatController) controller).setUser(user);
        });
    }

    public static Parent loadChatMessageScene(Message message) throws IOException {
        return SceneLoaderUtil.loadScene(MESSAGE_FXML, (controller) -> {
            ChatMessageController c = (ChatMessageController) controller;
            c.initializeFrom(message);
        });
    }

    public static Parent loadConnectScene(GuiApplication app) throws IOException {
        return SceneLoaderUtil.loadScene(CONNECT_FXML, (controller) ->
            ((ConnectController)controller).setApp(app));
    }

    private static Parent loadScene(String path, Consumer<Object> func)
        throws IOException
    {
        Logger logger = LoggerFactory.getLogger(GuiApplication.class);
        URL fxmlUrl = GuiApplication.class.getResource(path);

        if (fxmlUrl == null)
            throw new FileNotFoundException(
                "Could not find FXML file at " + path
            );

        // Load FXML.
        FXMLLoader fxmlLoader = new FXMLLoader(fxmlUrl);
        Parent parent = fxmlLoader.load();

        // Prepare controller.
        Object controller = fxmlLoader.getController();

        // Let the caller configure the controller through func argument.
        if (func != null) {
            if (controller == null) {
                logger.error(String.format(
                    "FXML at %s does not have a controller.",
                    fxmlUrl
                ));
            } else {
                func.accept(controller);
            }
        }

        return parent;
    }
}
