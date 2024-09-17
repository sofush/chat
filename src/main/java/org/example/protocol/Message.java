package org.example.protocol;

import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;

public class Message {
    MessageHeader header;
    ByteBuffer content;

    Message(MessageHeader header, byte[] content) {
        this.header = header;
        this.content = ByteBuffer.wrap(content);
    }

    /**
     * Create a new message of the given type.
     * @param type The type of the message.
     * @return The message that has been created.
     */
    public static Message create(MessageType type) {
        return new Message(
            new MessageHeader(0, type, null),
            new byte[0]
        );
    }

    /**
     * Adds an argument to the body of the message.
     * @param argument The argument to add.
     */
    public void addArgument(Object argument) {
        switch (argument) {
            case String string -> {
                byte[] bytes = string.getBytes(StandardCharsets.UTF_8);
                this.addBytes(ByteBuffer.wrap(bytes));
            }
            case ByteBuffer buffer -> this.addBytes(buffer);
            case byte[] buffer -> this.addBytes(ByteBuffer.wrap(buffer));
            default ->
                    throw new IllegalArgumentException("Argument type is not supported.");
        }
    }

    private void addBytes(ByteBuffer buffer) {
        int bufferLength = buffer.limit() - buffer.position();
        int newCapacity = this.content.limit() - this.content.position() + bufferLength;
        ByteBuffer newBuffer = ByteBuffer.allocate(newCapacity);
        newBuffer.put(this.content.clear());
        newBuffer.put(buffer);
        this.content = newBuffer;
    }

    public MessageHeader getHeader() {
        return this.header;
    }
}
