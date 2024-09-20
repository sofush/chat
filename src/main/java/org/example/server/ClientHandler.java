package org.example.server;

import org.example.protocol.Message;
import org.example.protocol.MessageTransfer;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.Closeable;
import java.io.IOException;
import java.net.Socket;
import java.util.concurrent.Flow;
import java.util.concurrent.SubmissionPublisher;

public class ClientHandler implements Runnable, Closeable, Flow.Subscriber<Message> {
    private User user;
    private final Socket client;
    private final Logger logger;
    private final SubmissionPublisher<Message> publisher;
    private Flow.Subscription subscription;

    public ClientHandler(SubmissionPublisher<Message> publisher, Socket client) {
        this.publisher = publisher;
        this.publisher.subscribe(this);
        this.client = client;
        this.logger = LoggerFactory.getLogger(ClientHandler.class);
        this.user = new User();
    }

    public void handleMessage(Message message) {
        MessageType type = message.getHeader().getType();
        this.logger.info("Got message of type " + type + ".");

        switch (message.getHeader().getType()) {
            case BROADCAST, UNICAST -> this.publisher.offer(message, null);
            case CHANGE_DISPLAY_NAME -> {
                String newDisplayName = (String) message.getArguments().nth(0);
                this.user.setDisplayName(newDisplayName);
            }
            case SWITCH_ROOM -> {
                String newRoomName = (String) message.getArguments().nth(0);
                this.user.setRoomName(newRoomName);
            }
            case FILE -> throw new RuntimeException("Not implemented yet.");
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

        this.close();
    }

    @Override
    public void onSubscribe(Flow.Subscription subscription) {
        this.subscription = subscription;
        this.subscription.request(Long.MAX_VALUE);
    }

    @Override
    public void onNext(Message msg) {
        switch (msg.getHeader().getType()) {
            case INVALID, FILE, CHANGE_DISPLAY_NAME, SWITCH_ROOM -> { return; }
            case UNICAST ->
                throw new RuntimeException("Not implemented yet.");
        }

        try {
            MessageTransfer.send(this.client, msg);
        } catch (IOException e) {
            this.logger.error("Could not send message.");
        }
    }

    @Override
    public void onError(Throwable throwable) {
        this.logger.error("Got subscriber error: ", throwable);
    }

    @Override
    public void onComplete() {
        this.logger.debug("No more messages from message broker.");
    }

    @Override
    public void close() {
        if (subscription != null)
            subscription.cancel();

        try {
            this.logger.info("Closing client handler.");
            this.client.close();
        } catch (IOException e) {
            this.logger.error("Could not close ClientHandler.", e);
        }
    }
}
