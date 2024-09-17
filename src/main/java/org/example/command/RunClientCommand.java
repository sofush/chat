package org.example.command;

import org.example.TcpClient;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

public class RunClientCommand implements Runnable {
    private final Logger logger;
    private final String host;
    private final int port;

    public RunClientCommand(String host, int port) {
        this.logger = LoggerFactory.getLogger(RunServerCommand.class);
        this.host = host;
        this.port = port;
    }

    @Override
    public void run() {
        try (TcpClient client = new TcpClient(this.host, this.port)) {
            client.run();
        } catch (IOException e) {
            this.logger.error(String.format(
                    "Client could not connect to TCP server at %s:%d.",
                    this.host,
                    this.port
            ));
        }
    }
}
