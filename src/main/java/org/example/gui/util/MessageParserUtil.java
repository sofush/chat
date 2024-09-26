package org.example.gui.util;

import org.example.entity.User;
import org.example.protocol.Message;
import org.example.protocol.MessageType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.Arrays;
import java.util.stream.Collectors;

public class MessageParserUtil {
    private static final Logger logger = LoggerFactory.getLogger(MessageParserUtil.class);

    private MessageParserUtil() {}

    /**
     * Parses user input into a Message.
     * @param user The user that sent the input string.
     * @param userInput The input string.
     * @return A message if the input could be parsed, or null.
     */
    public static Message parse(User user, String userInput) {
        if (userInput.toLowerCase().startsWith("/msg ")) {
            try {
                return MessageParserUtil.tryParseUnicastMessage(user, userInput);
            } catch (IllegalArgumentException e) {
                logger.warn("Could not parse unicast message.", e);
                return null;
            }
        } else {
            Message msg = Message.create(MessageType.BROADCAST);
            msg.addArgument(user.getRoom());
            msg.addArgument(user.getUsername());
            msg.addArgument(userInput);
            return msg;
        }
    }

    /**
     * Parses a unicast message from user input.
     * @param user The user that sent the input string.
     * @param userInput The input string.
     * @return A message if the input could be parsed, or null.
     * @throws IllegalArgumentException If the input string is not in the correct format.
     */
    private static Message tryParseUnicastMessage(User user, String userInput)
        throws IllegalArgumentException
    {
        String[] splits = userInput.split(" ");
        if (splits.length <= 2)
            throw new IllegalArgumentException("Missing recipient or message contents.");

        String recipient = splits[1];
        String messageContents = Arrays.stream(splits)
            .skip(2)
            .collect(Collectors.joining(" "));

        if (messageContents.isBlank())
            throw new IllegalArgumentException("Message contents is blank.");

        Message msg = Message.create(MessageType.UNICAST);
        msg.addArgument(user.getRoom());
        msg.addArgument(user.getUsername());
        msg.addArgument(messageContents);
        msg.addArgument(recipient);
        return msg;
    }
}
