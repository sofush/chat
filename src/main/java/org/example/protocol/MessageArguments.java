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
            while (n > 1) {
                int argumentLength = buffer.getInt();
                buffer.position(buffer.position() + argumentLength);
                n--;
            }

            int argumentLength = buffer.getInt();
            ByteBuffer slice = buffer.slice(buffer.position(), argumentLength);
            return this.parseNthArgument(n, slice);
        } catch (Exception ignored) {}

        return null;
    }

    private Object parseNthArgument(int n, ByteBuffer buffer) {
        switch (this.message.header.type) {
            case BROADCAST, UNICAST, CHANGE_USERNAME, SWITCH_ROOM -> {
                if (n == 0) return parseString(buffer);
            }
            case FILE -> {
                if (n == 0) return parseString(buffer);
                if (n == 1) return buffer;
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
