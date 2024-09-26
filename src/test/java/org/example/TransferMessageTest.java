package org.example;

import junit.framework.Test;
import junit.framework.TestCase;
import junit.framework.TestSuite;
import org.example.entity.User;
import org.example.protocol.Message;
import org.example.protocol.MessageTransfer;
import org.example.protocol.MessageType;
import org.example.server.TcpServer;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.Socket;

public class TransferMessageTest extends TestCase {
    public TransferMessageTest(String testName) {
        super(testName);
    }

    public static Test suite() {
        return new TestSuite(TransferMessageTest.class);
    }

    public void testBroadcastMessage() throws IOException, InterruptedException {
        InetSocketAddress address = new InetSocketAddress(0);
        Thread serverThread;

        try (TcpServer server = new TcpServer(address);
             TcpClient clientA = new TcpClient("localhost", server.getPort());
             TcpClient clientB = new TcpClient("localhost", server.getPort()))
        {
            serverThread = new Thread(server);
            serverThread.start();

            String room = "room-1";
            User userA = new User("user-a", room);
            User userB = new User("user-b", room);
            String message = "Hello, world!";

            sendUpdateUserMessage(userA, clientA.getSocket());
            sendUpdateUserMessage(userB, clientB.getSocket());

            // Send a message from user A to user B.
            Message broadcastMessage = Message.create(MessageType.BROADCAST);
            broadcastMessage.addArgument(room); // Name of the room to broadcast in.
            broadcastMessage.addArgument(userA.getUsername()); // Username of the sender.
            broadcastMessage.addArgument(message); // Message contents.
            MessageTransfer.send(clientA.getSocket(), broadcastMessage);

            // User B attempts to read the message.
            Message received = MessageTransfer.receive(clientB.getSocket());

            // Check that the received message is the one that was sent.
            assertNotNull(received);
            assertEquals(
                received.getHeader().getType().toInt(),
                MessageType.BROADCAST.toInt()
            );
            assertEquals(
                (String) received.getArguments().nth(2),
                message
            );
        }

        serverThread.join();
    }

    private static void sendUpdateUserMessage(User user, Socket socket)
        throws IOException
    {
        Message msg = Message.create(MessageType.UPDATE_USER);
        msg.addArgument(user.getRoom());
        msg.addArgument(user.getUsername());
        MessageTransfer.send(socket, msg);
    }
}
