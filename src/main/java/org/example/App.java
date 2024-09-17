package org.example;

import org.example.command.RunClientCommand;
import org.example.command.RunServerCommand;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class App {
    public static void main(String[] args) {
        String host = "localhost";
        int port = 3000;

        switch (args[0].toLowerCase()) {
            case "server" -> new RunServerCommand(port).run();
            case "client" -> new RunClientCommand(host, port).run();
            default -> {
                Logger logger = LoggerFactory.getLogger(App.class);
                logger.error("Program must be provided with an argument, expected `server` or `client`");
            }
        }
    }
}
