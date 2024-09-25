package org.example.protocol;

import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;

public class MessageArguments {
    private final Message message;

    MessageArguments(Message message) {
        this.message = message;
    }

    /**
     * Retrieves the Nth argument in the message body (if it exists).
     * @param n The index of the argument to retrieve.
     * @return The argument at the Nth index, or null if it doesn't exist.
     */
    public Object nth(int n) {
        ByteBuffer buffer = this.message.content.duplicate().clear();

        try {
            for (int i = 0; i < n; i++) {
                int argumentLength = buffer.getInt();
                buffer.position(buffer.position() + argumentLength);
            }

            int argumentLength = buffer.getInt();
            ByteBuffer slice = buffer.slice(buffer.position(), argumentLength);
            return this.parseNthArgument(n, slice);
        } catch (Exception ignored) {}

        return null;
    }

    private Object parseNthArgument(int n, ByteBuffer buffer) {
        switch (this.message.header.type) {
            case UPDATE_USER -> {
                if (n == 0) return parseString(buffer); // room
                if (n == 1) return parseString(buffer); // username
            }
            case BROADCAST -> {
                if (n == 0) return parseString(buffer); // room
                if (n == 1) return parseString(buffer); // sender
                if (n == 2) return parseString(buffer); // message
            }
            case UNICAST -> {
                if (n == 0) return parseString(buffer); // room
                if (n == 1) return parseString(buffer); // sender
                if (n == 2) return parseString(buffer); // message
                if (n == 3) return parseString(buffer); // recipient
            }
            case FILE -> {
                if (n == 0) return parseString(buffer); // filename
                if (n == 1) return buffer; // file contents
            }
        }

        return null;
    }

    private String parseString(ByteBuffer buffer) {
        return new String(
            buffer.array(),
            buffer.position() + buffer.arrayOffset(),
            buffer.limit() - buffer.position(),
            StandardCharsets.UTF_8
        );
    }
}
