package org.example.gui;

import javafx.concurrent.ScheduledService;
import javafx.concurrent.Task;
import org.example.TcpClient;
import org.example.protocol.Message;
import org.example.protocol.MessageTransfer;

public class ReadMessageService extends ScheduledService<Message> {
    static class ReadMessageTask extends Task<Message> {
        private final TcpClient client;

        public ReadMessageTask(TcpClient client) {
            this.client = client;
        }

        @Override
        protected Message call() throws Exception {
            return MessageTransfer.receive(this.client.getSocket());
        }
    }

    private final TcpClient client;

    public ReadMessageService(TcpClient client) {
        this.client = client;
    }

    @Override
    protected Task<Message> createTask() {
        return new ReadMessageTask(this.client);
    }
}
