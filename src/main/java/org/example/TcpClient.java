package org.example;

import org.example.protocol.Message;
import org.example.protocol.MessageTransfer;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.Closeable;
import java.io.IOException;
import java.net.Socket;

public class TcpClient implements Runnable, Closeable {
    private final Logger logger;
    private final Socket socket;

    public TcpClient(String host, int port) throws IOException {
        this.logger = LoggerFactory.getLogger(TcpClient.class);
        this.logger.info(String.format(
            "Client is attempting to connect to server on %s:%d.",
            host,
            port
        ));
        this.socket = new Socket(host, port);
        this.logger.info("Client has connected to the server.");
    }

    public Socket getSocket() {
        return this.socket;
    }

    @Override
    public void run() {
        var message = Message.create(MessageType.BROADCAST);
        message.addArgument("Hello, world!");

        try {
            MessageTransfer.send(this.socket, message);
        } catch (IOException e) {
            this.logger.error("Could not send message to server.", e);
        }
    }

    @Override
    public void close() {
        this.logger.info("Closing TCP client.");

        try {
            this.socket.close();
        } catch (IOException e) {
            this.logger.error("Could not close TCP client socket.");
        }
    }
}
