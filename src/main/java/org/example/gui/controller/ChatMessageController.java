package org.example.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.Label;
import org.example.protocol.Message;
import org.example.protocol.MessageArguments;

import java.time.Instant;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;

public class ChatMessageController {
    @FXML private Label messageSenderLabel;
    @FXML private Label messageContentLabel;
    @FXML private Label messageTimestampLabel;

    public void initializeFrom(Message message) throws IllegalArgumentException {
        MessageArguments args = message.getArguments();

        switch (message.getHeader().getType()) {
            case INVALID, FILE, UPDATE_USER ->
                throw new IllegalArgumentException("Invalid message type.");
            case BROADCAST -> {
                this.messageSenderLabel.setText((String) args.nth(1));
            }
            case UNICAST -> {
                String sender = (String) args.nth(1);
                String recipient = (String) args.nth(3);
                String senderStr = recipient == null
                    ? sender
                    : String.format("%s → %s", sender, recipient);

                this.messageSenderLabel.setText(senderStr);
            }
        }

        // Set the message content text.
        this.messageContentLabel.setText((String) args.nth(2));

        // Set the timestamp text.
        DateTimeFormatter formatter = DateTimeFormatter
            .ofPattern("uuuu-MM-dd HH:mm:ss.SSS")
            .withZone(ZoneId.systemDefault());

        this.messageTimestampLabel.setText(String.format(
            "sendt %s",
            formatter.format(message.getHeader().getTimestamp())
        ));
    }

    public Label getMessageSenderLabel() {
        return messageSenderLabel;
    }

    public Label getMessageContentLabel() {
        return messageContentLabel;
    }

    public Label getMessageTimestampLabel() {
        return messageTimestampLabel;
    }
}
