package org.example.gui.controller;

import javafx.fxml.FXML;
import javafx.event.Event;
import javafx.scene.control.TextField;
import javafx.scene.layout.VBox;
import javafx.scene.text.Text;
import org.example.TcpClient;
import org.example.gui.ReadMessageService;
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

    @FXML private TextField messageTextField;
    @FXML private VBox messageContainer;

    public void setClient(TcpClient client) {
        this.client = client;
        this.readMessageService = new ReadMessageService(this.client);
        this.readMessageService.setOnSucceeded((e) -> {
            Message msg = this.readMessageService.getValue();
            String content = (String) msg.getArguments().nth(0);
            this.messageContainer.getChildren().add(new Text(content));
        });
        this.readMessageService.start();
    }

    @FXML
    public void sendMessage(Event ignored) {
        if (this.client == null)
            throw new IllegalStateException("Client must not be null.");

        String content = this.messageTextField.getText();
        Message message = Message.create(MessageType.BROADCAST);
        message.addArgument(content);

        try {
            MessageTransfer.send(this.client.getSocket(), message);
            this.messageTextField.clear();
        } catch (IOException ex) {
            this.logger.error("Could not send message.", ex);
        }
    }

    @FXML
    public void roomOneButtonClicked(Event ignored) {}

    @Override
    public void close() {
        if (this.client != null)
            this.client.close();
    }
}
