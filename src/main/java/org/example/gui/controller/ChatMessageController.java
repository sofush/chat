package org.example.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.Label;

public class ChatMessageController {
    @FXML private Label messageSenderLabel;
    @FXML private Label messageContentLabel;
    @FXML private Label messageTimestampLabel;

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
