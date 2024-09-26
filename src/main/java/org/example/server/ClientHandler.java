package org.example.server;

import org.example.entity.User;
import org.example.protocol.Message;
import org.example.protocol.MessageTransferUtil;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.Closeable;
import java.io.IOException;
import java.net.Socket;
import java.util.concurrent.Flow;
import java.util.concurrent.SubmissionPublisher;

public class ClientHandler implements Runnable, Closeable, Flow.Subscriber<Message> {
    private final User user;
    private final Socket client;
    private final Logger logger;
    private final SubmissionPublisher<Message> publisher;
    private Flow.Subscription subscription;

    public ClientHandler(SubmissionPublisher<Message> publisher, Socket client) {
        this.publisher = publisher;
        this.publisher.subscribe(this);
        this.client = client;
        this.logger = LoggerFactory.getLogger(ClientHandler.class);
        this.user = new User(null, null);
    }

    public void handleMessage(Message message) {
        MessageType type = message.getHeader().getType();
        this.logger.info("Got message of type " + type + ".");

        switch (message.getHeader().getType()) {
            case BROADCAST, UNICAST -> this.publisher.offer(message, null);
            case UPDATE_USER -> {
                String room = (String) message.getArguments().nth(0);
                String username = (String) message.getArguments().nth(1);
                this.user.setUsername(username);
                this.user.setRoom(room);
            }
            case FILE -> throw new RuntimeException("Not implemented yet.");
        }
    }

    @Override
    public void run() {
        while (!Thread.interrupted() && client.isConnected() && !client.isClosed()) {
            try {
                Message message = MessageTransferUtil.receive(client);
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
            case INVALID, FILE, UPDATE_USER -> { return; }
            case UNICAST -> {
                String room = (String) msg.getArguments().nth(0);
                String sender = (String) msg.getArguments().nth(1);
                String recipient = (String) msg.getArguments().nth(3);
                String displayName = this.user.getUsername();

                if (displayName == null || !room.contentEquals(this.user.getRoom()))
                    return;

                boolean isSender = displayName.contentEquals(sender);
                boolean isRecipient = displayName.contentEquals(recipient);

                if (!(isSender || isRecipient))
                    return;
            }
            case BROADCAST -> {
                String room = (String) msg.getArguments().nth(0);
                if (!this.user.getRoom().contentEquals(room))
                    return;
            }
        }

        try {
            MessageTransferUtil.send(this.client, msg);
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
