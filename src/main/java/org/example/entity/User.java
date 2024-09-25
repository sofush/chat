package org.example.entity;

public class User {
    private String username;
    private String room;

    public User(String username, String room) {
        this.username = username;
        this.room = room;
    }

    public String getRoom() {
        return room;
    }

    public void setRoom(String room) {
        this.room = room;
    }

    public String getUsername() {
        return username;
    }

    public void setUsername(String username) {
        this.username = username;
    }

    public boolean isValid() {
        if (this.room == null || this.username == null)
            return false;

        if (this.room.isBlank() || this.username.isBlank())
            return false;

        return !this.username.contains(" ");
    }
}
