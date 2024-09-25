package org.example.gui.controller;

import javafx.concurrent.WorkerStateEvent;
import javafx.fxml.FXML;
import javafx.event.Event;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.TextField;
import javafx.scene.layout.VBox;
import org.example.TcpClient;
import org.example.entity.User;
import org.example.gui.util.MessageParserUtil;
import org.example.gui.ReadMessageService;
import org.example.gui.util.SceneLoaderUtil;
import org.example.protocol.Message;
import org.example.protocol.MessageTransfer;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.Closeable;
import java.io.IOException;

public class ChatController implements Closeable {
    private final Logger logger = LoggerFactory.getLogger(ChatController.class);
    private TcpClient client;
    private ReadMessageService readMessageService;
    private User user;

    @FXML private ScrollPane messageScrollPane;
    @FXML private TextField messageTextField;
    @FXML private VBox messageContainer;

    public void setUser(User user) {
        this.user = user;
    }

    public void setClient(TcpClient client) {
        this.client = client;
        this.readMessageService = new ReadMessageService(this.client);
        this.readMessageService.setOnSucceeded(this::addMessage);
        this.readMessageService.start();

        // Scroll to the bottom when the inner container's height changes.
        this.messageContainer
            .heightProperty()
            .addListener(o -> this.messageScrollPane.setVvalue(1));
    }

    public void addMessage(WorkerStateEvent ignored) {
        Message msg = this.readMessageService.getValue();

        try {
            Node messageNode = SceneLoaderUtil.loadChatNode(msg);
            this.messageContainer.getChildren().add(messageNode);
        } catch (Exception ex) {
            this.logger.error("Could not add message to scroll pane.", ex);
        }
    }

    @FXML
    public void sendMessage(Event ignored) {
        if (this.client == null || !this.user.isValid())
            throw new IllegalStateException("Client and user must be valid.");

        String userInput = this.messageTextField.getText();
        Message msg = MessageParserUtil.parse(this.user, userInput);
        if (msg == null) return;

        try {
            MessageTransfer.send(this.client.getSocket(), msg);
            this.messageTextField.clear();
        } catch (IOException ex) {
            this.logger.error("Could not send message.", ex);
        }
    }

    @FXML
    public void switchRoomButtonClicked(Event e) {
        String id = ((Button)e.getSource()).getId();

        if (id == null) return;
        else id = id.toLowerCase();

        if (id.contains("one")) {
            this.user.setRoom("Rum 1");
        } else if (id.contains("two")) {
            this.user.setRoom("Rum 2");
        } else if (id.contains("three")) {
            this.user.setRoom("Rum 3");
        }

        try {
            Message msg = Message.create(MessageType.UPDATE_USER);
            msg.addArgument(this.user.getRoom());
            msg.addArgument(this.user.getUsername());
            MessageTransfer.send(this.client.getSocket(), msg);
            this.messageContainer.getChildren().clear();
        } catch (IOException ex) {
            this.logger.error("Could not switch rooms.", ex);
        }
    }

    @Override
    public void close() {
        if (this.client != null)
            this.client.close();
    }
}
