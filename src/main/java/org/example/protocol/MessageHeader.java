package org.example.protocol;

import java.time.Instant;

public class MessageHeader {
    /**
     * The size of a message header in bytes.
     */
    final static int SIZE = 16;

    /**
     * The length of the message in bytes.
     */
    int length;

    /**
     * The type of the message (text, file, etc.)
     */
    MessageType type;

    /**
     * A timestamp of when the message was sent.
     */
    Instant timestamp;

    MessageHeader(int length, MessageType type, Instant timestamp) {
        this.length = length;
        this.type = type;
        this.timestamp = timestamp;
    }

    /**
     * Prepares a message header for sending by updating member variables.
     * @param contentLength The length of the message's content in bytes.
     */
    void finish(int contentLength) {
        this.setLength(MessageHeader.SIZE + contentLength);
        this.timestamp = Instant.now();
    }

    /**
     * Sets the value field of this message header. The length includes the message header's byte size.
     * @param newValue The value to set length to.
     * @throws IllegalArgumentException If the new value is less than the size of a message header.
     */
    void setLength(int newValue) throws IllegalArgumentException {
        if (newValue < MessageHeader.SIZE)
            throw new IllegalArgumentException("A message must be at least as long as the header.");

        this.length = newValue;
    }

    public Instant getTimestamp() {
        return this.timestamp;
    }

    public MessageType getType() {
        return this.type;
    }
}
