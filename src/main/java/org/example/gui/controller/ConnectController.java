package org.example.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.Parent;
import javafx.scene.control.TextField;
import org.example.TcpClient;
import org.example.entity.User;
import org.example.gui.GuiApplication;
import org.example.protocol.Message;
import org.example.protocol.MessageTransfer;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

public class ConnectController {
    private final static String CHAT_FXML = "/fxml/chat.fxml";
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
        TcpClient client;
        User user;

        // Attempt to connect to the server.
        try {
            user = this.createUser();

            if (user.isInvalid()) {
                this.logger.warn("Could not create a user with the provided input.");
                return;
            }

            client = new TcpClient(
                this.hostTextField.getText(),
                Integer.parseInt(this.portTextField.getText())
            );

            // Send the provided username.
            Message msg = Message.create(MessageType.UPDATE_USER);
            msg.addArgument(user.getRoom());
            msg.addArgument(user.getUsername());
            MessageTransfer.send(client.getSocket(), msg);
        } catch (IOException e) {
            this.logger.warn("Could not connect to the server.", e);
            return;
        } catch (NumberFormatException e) {
            this.logger.warn("User has typed a port that is not an integer.", e);
            return;
        }

        this.switchToChatScene(client, user);
    }

    private void switchToChatScene(TcpClient client, User user) {
        Parent chatParent;

        // Load the "chat" scene.
        try {
            chatParent = GuiApplication.loadScene(CHAT_FXML, (controller) -> {
                ((ChatController)controller).setClient(client);
                ((ChatController)controller).setUser(user);
            });
        } catch (Exception e) {
            client.close();
            this.logger.error("Could not load chat room scene at path {}.", CHAT_FXML, e);
            return;
        }

        // Show the "chat" scene.
        this.app.getStage().getScene().setRoot(chatParent);
    }

    private User createUser() {
        return new User(
            this.usernameTextField.getText(),
            "Rum 1"
        );
    }
}
