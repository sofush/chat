package org.example.server;

import org.example.protocol.Message;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.Closeable;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.ServerSocket;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.SubmissionPublisher;

public class TcpServer implements Runnable, Closeable {
    private final Logger logger;
    private final ServerSocket channel;
    private final ExecutorService executors;
    private final SubmissionPublisher<Message> publisher;

    public TcpServer(InetSocketAddress address) throws IOException {
        this.logger = LoggerFactory.getLogger(TcpServer.class);
        this.publisher = new SubmissionPublisher<>();
        this.channel = new ServerSocket(address.getPort());
        this.logger.info("TCP server is listening on port " + this.channel.getLocalPort() + ".");
        this.executors = Executors.newCachedThreadPool();
    }

    private void acceptClient() {
        try {
            var client = this.channel.accept();
            this.executors.execute(new ClientHandler(this.publisher, client));
            this.logger.info("A client has connected.");
        } catch (IOException e) {
            this.logger.error("Could not accept or register client.", e);
        }
    }

    @Override
    public void run() {
        while (!Thread.interrupted()) {
            this.acceptClient();
        }

        this.executors.shutdownNow();
    }

    @Override
    public void close() throws IOException {
        this.logger.info("Closing TCP server.");
        this.channel.close();
        this.executors.close();
    }
}
