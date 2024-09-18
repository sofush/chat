package org.example.gui;

import javafx.application.Application;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.stage.Stage;

import java.io.FileNotFoundException;
import java.net.URL;

public class GuiApplication extends Application {
    @Override
    public void start(Stage stage) throws Exception {
        URL fxmlUrl = GuiApplication.class.getResource("/fxml/main.fxml");

        if (fxmlUrl == null)
            throw new FileNotFoundException("Could not find main.fxml file.");

        // Prepare FXML.
        FXMLLoader fxmlLoader = new FXMLLoader(fxmlUrl);
        Parent root = fxmlLoader.load();
        GuiController controller = fxmlLoader.getController();

        // Configure connection details.
        // TODO let the user enter these values
        controller.setHost("localhost");
        controller.setPort(3000);

        // Prepare the stage.
        Scene scene = new Scene(root);
        stage.setScene(scene);
        stage.setTitle("Chat app");
        stage.show();
    }

    public static void main(String[] args) {
        launch(args);
    }
}
