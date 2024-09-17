package org.example.protocol;

import java.util.Arrays;

public enum MessageType {
    UNKNOWN(0),
    TEMPERATURE(1),
    AIR_HUMIDITY(2),
    SOIL_MOISTURE(3);

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
                .findFirst().orElse(MessageType.UNKNOWN);
    }
}
