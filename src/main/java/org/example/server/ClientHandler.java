package org.example.server;

import org.example.protocol.Message;
import org.example.protocol.MessageArguments;
import org.example.protocol.MessageTransfer;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.Closeable;
import java.io.IOException;
import java.net.Socket;

public class ClientHandler implements Runnable, Closeable {
    private final Socket client;
    private final Logger logger;

    public ClientHandler(Socket client) {
        this.client = client;
        this.logger = LoggerFactory.getLogger(ClientHandler.class);
    }

    public void handleMessage(Message message) {
        MessageType type = message.getHeader().getType();
        this.logger.info("Got message of type " + type + ".");

        MessageArguments args = message.getArguments();

        switch (message.getHeader().getType()) {
            case BROADCAST, UNICAST -> {
                String arg = (String) args.nth(0);
                this.logger.debug("Message contents: " + arg);
            }
            case FILE -> {}
        }
    }

    @Override
    public void run() {
        while (!Thread.interrupted() && client.isConnected() && !client.isClosed()) {
            try {
                Message message = MessageTransfer.receive(client);
                if (message == null) continue;
                this.handleMessage(message);
            } catch (IOException e) {
                this.logger.info("Client disconnected.");
                break;
            }
        }

        try {
            this.close();
        } catch (IOException e) {
            this.logger.error("Could not close ClientHandler.", e);
        }
    }

    @Override
    public void close() throws IOException {
        this.logger.info("Closing client handler.");
        this.client.close();
    }
}
