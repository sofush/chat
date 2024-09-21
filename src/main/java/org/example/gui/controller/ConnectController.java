package org.example.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.Parent;
import javafx.scene.control.TextField;
import org.example.TcpClient;
import org.example.gui.GuiApplication;
import org.example.protocol.Message;
import org.example.protocol.MessageTransfer;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

public class ConnectController {
    private final Logger logger = LoggerFactory.getLogger(ConnectController.class);
    private GuiApplication app;

    @FXML private TextField hostTextField;
    @FXML private TextField portTextField;
    @FXML private TextField usernameTextField;

    public void setApp(GuiApplication app) {
        this.app = app;
    }

    @FXML
    public void connectToServer() {
        String path = "/fxml/chat.fxml";
        Parent chatParent;
        TcpClient client;

        String username = this.usernameTextField.getText();
        if (username.isBlank()) return;

        // Connect to the server.
        try {
            client = new TcpClient(
                this.hostTextField.getText(),
                Integer.parseInt(this.portTextField.getText())
            );

            Message msg = Message.create(MessageType.CHANGE_USERNAME);
            msg.addArgument(username);
            MessageTransfer.send(client.getSocket(), msg);
        } catch (IOException e) {
            this.logger.warn("Could not connect to the server.");
            return;
        } catch (NumberFormatException e) {
            this.logger.warn("User has typed a port that is not an integer.");
            return;
        }

        // Load the "chat" scene.
        try {
            chatParent = GuiApplication.loadScene(path, (controller) -> {
                ((ChatController)controller).setClient(client);
            });
        } catch (Exception e) {
            client.close();
            this.logger.error("Could not load scene at path {}.", path);
            return;
        }

        // Show the "chat" scene.
        this.app.getStage().getScene().setRoot(chatParent);
    }
}
