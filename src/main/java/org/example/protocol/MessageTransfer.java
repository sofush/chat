package org.example.protocol;

import java.io.IOException;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.time.Instant;

public class MessageTransfer {
    private MessageTransfer() {}

    /**
     * Writes a message to a socket.
     * @param socket The socket to write the message to.
     * @param message The message to write.
     */
    public static void send(Socket socket, Message message) throws IOException {
        // Prepare the header for sending.
        message.header.finish(message.content.capacity());

        // Put the header and content bytes into a buffer.
        var buffer = ByteBuffer.allocate(message.header.length);
        buffer.putInt(message.header.length);
        buffer.putInt(message.header.type.toInt());
        buffer.putLong(message.header.timestamp.toEpochMilli());
        buffer.put(message.content.clear());

        // Write the buffer contents to the socket output stream.
        socket.getOutputStream().write(buffer.array(), 0, buffer.limit());
    }

    /**
     * Reads a message from a socket.
     * @param socket The socket to read from.
     */
    public static Message receive(Socket socket) throws IOException {
        // Read a message header from the socket.
        MessageHeader header = MessageTransfer.receiveHeader(socket);
        int contentLength = header.length - MessageHeader.SIZE;

        if (header.type == MessageType.INVALID || header.length < MessageHeader.SIZE)
            return null;

        // Read the rest of the message from the socket.
        ByteBuffer buffer = ByteBuffer.allocate(contentLength);
        int read = socket.getInputStream().readNBytes(buffer.array(), 0, buffer.capacity());

        if (read == -1)
            throw new IOException("End of stream.");

        return new Message(header, buffer.array());
    }

    /**
     * Reads a message header from the socket.
     * @param socket The socket to read a message header from.
     * @return The message header.
     */
    private static MessageHeader receiveHeader(Socket socket) throws IOException {
        var buffer = ByteBuffer.allocate(MessageHeader.SIZE);
        int read = socket.getInputStream().readNBytes(buffer.array(), 0, buffer.capacity());

        if (read < buffer.capacity())
            throw new IOException("End of stream.");

        int length = buffer.getInt();
        MessageType type = MessageType.from(buffer.getInt());
        Instant timestamp = Instant.ofEpochMilli(buffer.getLong());
        return new MessageHeader(length, type, timestamp);
    }
}
