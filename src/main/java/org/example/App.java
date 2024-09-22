package org.example;

import org.example.server.TcpServer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.net.InetSocketAddress;

public class App {
    public static void main(String[] args) {
        Logger logger = LoggerFactory.getLogger(App.class);
        InetSocketAddress address = new InetSocketAddress("localhost", 0);

        try (TcpServer server = new TcpServer(address)) {
            logger.info("Starting TCP server...");
            server.run();
        } catch (IOException e) {
            logger.error("Could not start TCP server on port " + address.getPort() + ".");
        }
    }
}
