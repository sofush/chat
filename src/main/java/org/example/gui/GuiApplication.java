package org.example.gui;

import javafx.application.Application;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.stage.Stage;
import org.example.gui.util.SceneLoaderUtil;

public class GuiApplication extends Application {
    private Stage stage;

    public static void main(String[] args) {
        launch(args);
    }

    public Stage getStage() {
        return this.stage;
    }

    @Override
    public void start(Stage stage) throws Exception {
        this.stage = stage;
        Parent connectParent = SceneLoaderUtil.loadConnectScene(this);

        // Prepare the stage.
        stage.setScene(new Scene(connectParent));
        stage.setWidth(800);
        stage.setHeight(600);
        stage.setTitle("Chat app");
        stage.show();
    }
}
