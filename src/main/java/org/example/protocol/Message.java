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
        if (argument instanceof String s) {
            byte[] bytes = s.getBytes(StandardCharsets.UTF_8);
            this.addBytes(ByteBuffer.wrap(bytes));
        } else if (argument instanceof ByteBuffer b) {
            this.addBytes(b);
        } else if (argument instanceof byte[] b) {
            this.addBytes(ByteBuffer.wrap(b));
        } else {
            throw new IllegalArgumentException("Argument type is not supported.");
        }
    }

    /**
     * Pushes the given bytes to the end of the message content buffer.
     * @param buffer A buffer containing the bytes to push.
     */
    private void addBytes(ByteBuffer buffer) {
        int bufferLength = buffer.limit() - buffer.position();
        int newCapacity = this.content.limit() - this.content.position() + bufferLength + 4;
        ByteBuffer newBuffer = ByteBuffer.allocate(newCapacity);
        newBuffer.put(this.content.clear());
        newBuffer.putInt(bufferLength);
        newBuffer.put(buffer);
        this.content = newBuffer;
    }

    public MessageArguments getArguments() {
        return new MessageArguments(this);
    }

    public MessageHeader getHeader() {
        return this.header;
    }
}
