package org.example;

import org.example.protocol.Message;
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

    private void sleep() {
        try {
            long ms = (long)(Math.random() * 250.0);
            Thread.sleep(ms);
        } catch (InterruptedException ignored) {}
    }

    @Override
    public void run() {
        while (!Thread.interrupted()) {
            this.sleep();
            var message = new Message();

            try {
                message.send(this.socket);
            } catch (IOException e) {
                this.logger.error("Could not send message to server.", e);
                break;
            }
        }

        try {
            this.close();
        } catch (IOException e) {
            this.logger.error("Could not close TCP client.", e);
        }
    }

    @Override
    public void close() throws IOException {
        this.logger.info("Closing TCP client.");
        this.socket.close();
    }
}
