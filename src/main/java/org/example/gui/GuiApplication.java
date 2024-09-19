package org.example.gui;

import javafx.application.Application;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.stage.Stage;
import org.example.gui.controller.ConnectController;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.FileNotFoundException;
import java.net.URL;
import java.util.function.Consumer;

public class GuiApplication extends Application {
    private Stage stage;

    public static void main(String[] args) {
        launch(args);
    }

    public Stage getStage() {
        return this.stage;
    }

    public static Parent loadScene(String path, Consumer<Object> func) throws Exception {
        Logger logger = LoggerFactory.getLogger(GuiApplication.class);
        URL fxmlUrl = GuiApplication.class.getResource(path);

        if (fxmlUrl == null)
            throw new FileNotFoundException(
                "Could not find FXML file at " + path
            );

        // Load FXML.
        FXMLLoader fxmlLoader = new FXMLLoader(fxmlUrl);
        Parent parent = fxmlLoader.load();

        // Prepare controller.
        Object controller = fxmlLoader.getController();

        // Let the caller configure the controller through func argument.
        if (func != null) {
            if (controller == null) {
                logger.error(String.format(
                    "FXML at %s does not have a controller.",
                    fxmlUrl
                ));
            } else {
                func.accept(controller);
            }
        }

        return parent;
    }

    @Override
    public void start(Stage stage) throws Exception {
        this.stage = stage;

        Parent connectParent = loadScene("/fxml/connect.fxml", (controller) -> {
            ((ConnectController)controller).setApp(this);
        });

        // Prepare the stage.
        stage.setScene(new Scene(connectParent));
        stage.setWidth(800);
        stage.setHeight(600);
        stage.setTitle("Chat app");
        stage.show();
    }
}
