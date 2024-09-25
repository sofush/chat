package org.example.protocol;

import java.util.Arrays;

public enum MessageType {
    INVALID(0),
    BROADCAST(1),
    UNICAST(2),
    FILE(3),
    UPDATE_USER(4);

    private final int value;

    MessageType(int value) {
        this.value = value;
    }

    public int toInt() {
        return value;
    }

    public static MessageType from(int value) {
        return Arrays.stream(MessageType.values())
            .filter((variant) -> variant.toInt() == value)
            .findFirst()
            .orElse(INVALID);
    }
}
