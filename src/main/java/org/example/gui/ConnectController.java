package org.example.gui;

import javafx.fxml.FXML;
import javafx.scene.Parent;
import javafx.scene.control.TextField;
import javafx.stage.Stage;
import org.example.TcpClient;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

public class ConnectController {
    private final Logger logger = LoggerFactory.getLogger(ConnectController.class);
    private GuiApplication app;

    @FXML private TextField hostTextField;
    @FXML private TextField portTextField;

    public void setApp(GuiApplication app) {
        this.app = app;
    }

    @FXML
    public void connectToServer() {
        String path = "/fxml/chat.fxml";
        Parent chatParent;
        TcpClient client;

        // Connect to the server.
        try {
            client = new TcpClient(
                this.hostTextField.getText(),
                Integer.parseInt(this.portTextField.getText())
            );
        } catch (IOException e) {
            this.logger.warn("Could not connect to the server.");
            return;
        } catch (NumberFormatException e) {
            this.logger.warn("User has typed a port that is not an integer.");
            return;
        }

        // Load the "chat" scene.
        try {
            chatParent = this.app.loadScene(path, (controller) -> {
                ((ChatController)controller).setClient(client);
            });
        } catch (Exception e) {
            client.close();
            this.logger.error("Could not load scene at path {}.", path);
            return;
        }

        // Show the "chat" scene.
        this.app.getStage().getScene().setRoot(chatParent);
    }
}
