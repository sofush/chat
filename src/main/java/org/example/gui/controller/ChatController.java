package org.example.gui.controller;

import javafx.concurrent.WorkerStateEvent;
import javafx.fxml.FXML;
import javafx.event.Event;
import javafx.scene.Parent;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.TextField;
import javafx.scene.layout.VBox;
import org.example.TcpClient;
import org.example.gui.GuiApplication;
import org.example.gui.ReadMessageService;
import org.example.protocol.Message;
import org.example.protocol.MessageTransfer;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.Closeable;
import java.io.IOException;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.Arrays;
import java.util.stream.Collectors;

public class ChatController implements Closeable {
    private static final String MESSAGE_FXML = "/fxml/message.fxml";
    private final Logger logger = LoggerFactory.getLogger(ChatController.class);
    private TcpClient client;
    private ReadMessageService readMessageService;
    private String username;

    @FXML private ScrollPane messageScrollPane;
    @FXML private TextField messageTextField;
    @FXML private VBox messageContainer;

    public void setUsername(String username) {
        this.username = username;
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
        String sender, content;

        switch (msg.getHeader().getType()) {
            case BROADCAST, UNICAST -> {
                sender = (String) msg.getArguments().nth(0);
                content = (String) msg.getArguments().nth(1);
            }
            default -> { return; }
        }

        try {
            Parent messageNode = GuiApplication.loadScene(MESSAGE_FXML, (controller) -> {
                ChatMessageController c = (ChatMessageController) controller;

                c.getMessageContentLabel().setText(content);
                c.getMessageSenderLabel().setText(sender);

                var formatter = DateTimeFormatter
                    .ofPattern("uuuu-MM-dd HH:mm:ss.SSS")
                    .withZone(ZoneId.systemDefault());
                String formattedTimestamp = formatter.format(msg.getHeader().getTimestamp());
                c.getMessageTimestampLabel().setText("sendt " + formattedTimestamp);
            });

            this.messageContainer.getChildren().add(messageNode);
        } catch (Exception ex) {
            this.logger.error("Could not add message to scroll pane.", ex);
        }
    }

    @FXML
    public void sendMessage(Event ignored) {
        if (this.client == null)
            throw new IllegalStateException("Client must not be null.");

        String content = this.messageTextField.getText();
        Message msg;

        if (content.toLowerCase().startsWith("/msg ")) {
            String[] splits = content.split(" ");
            if (splits.length <= 2) return;

            String recipient = splits[1];
            String messageContents = Arrays.stream(splits)
                .skip(2)
                .collect(Collectors.joining());

            if (messageContents.isBlank()) return;

            msg = Message.create(MessageType.UNICAST);
            msg.addArgument(this.username);
            msg.addArgument(messageContents);
            msg.addArgument(recipient);
        } else {
            msg = Message.create(MessageType.BROADCAST);
            msg.addArgument(this.username);
            msg.addArgument(content);
        }

        try {
            MessageTransfer.send(this.client.getSocket(), msg);
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
