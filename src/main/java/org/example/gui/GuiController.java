package org.example.gui;

import javafx.fxml.FXML;
import javafx.event.Event;
import javafx.scene.control.Button;
import org.example.TcpClient;
import org.example.protocol.Message;
import org.example.protocol.MessageTransfer;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.Closeable;
import java.io.IOException;

public class GuiController implements Closeable {
    private final Logger logger = LoggerFactory.getLogger(GuiController.class);
    private TcpClient client = null;
    private String host = null;
    private int port = -1;

    public void trySend(Message message) throws Exception {
        if (this.client == null) {
            if (this.host == null || this.port == -1)
                throw new IllegalStateException("Host and port have not been configured.");

            this.client = new TcpClient(this.host, this.port);
        }

        MessageTransfer.send(this.client.getSocket(), message);
    }

    public void setHost(String host) {
        this.host = host;
    }

    public void setPort(int port) {
        this.port = port;
    }

    @FXML
    public void roomOneButtonClicked(Event ignored) {
        Message msg = Message.create(MessageType.BROADCAST);
        msg.addArgument("Hello, world!");

        try {
            this.trySend(msg);
        } catch (Exception e) {
            this.logger.error("Could not send message.", e);
        }
    }

    @Override
    public void close() throws IOException {
        this.client.close();
    }
}
