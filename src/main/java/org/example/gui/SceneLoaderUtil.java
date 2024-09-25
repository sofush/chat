package org.example.gui;

import javafx.scene.Parent;
import org.example.gui.controller.ChatMessageController;
import org.example.protocol.Message;

import java.io.IOException;

public class SceneLoaderUtil {
    private static final String MESSAGE_FXML = "/fxml/message.fxml";

    public static Parent loadChatNode(Message message) throws IOException {
        return GuiApplication.loadScene(MESSAGE_FXML, (controller) -> {
            ChatMessageController c = (ChatMessageController) controller;
            c.initializeFrom(message);
        });
    }
}
