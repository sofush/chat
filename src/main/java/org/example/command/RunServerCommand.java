package org.example.command;

import org.example.TcpServer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.net.InetSocketAddress;

public class RunServerCommand implements Runnable {
    private final Logger logger;
    private final int port;

    public RunServerCommand(int port) {
        this.logger = LoggerFactory.getLogger(RunServerCommand.class);
        this.port = port;
    }

    @Override
    public void run() {
        InetSocketAddress address = new InetSocketAddress(port);

        try (TcpServer server = new TcpServer(address)) {
            logger.info("Starting TCP server on port " + port + "...");
            server.run();
        } catch (IOException e) {
            logger.error("Could not start TCP server on port " + port + ".");
        }
    }
}
