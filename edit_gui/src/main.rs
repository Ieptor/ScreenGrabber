use druid::widget::prelude::*;
use druid::widget::{Label, Flex, Align};
use druid::{AppLauncher, LocalizedString, PlatformError, WindowDesc, Widget, Lens};

#[derive(Clone, Data, Lens)]
struct AppState {
    // Add your application state fields here
}

fn main() -> Result<(), PlatformError> {
    // Create the main window description
    let main_window = WindowDesc::new(build_ui())
        .window_size((700.0, 500.0))
        .resizable(false)
        .title(LocalizedString::new("EDIT GUI"));

    // Create the initial app state
    let initial_state = AppState {};

    // Launch the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)?;

    Ok(())
}

fn build_ui() -> impl Widget<AppState> {
    // Create a simple UI with a label centered in the window
    Flex::column()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Center)
        .with_child(
            Align::centered(Label::new("apri l'immagine etc etc"))     
        )
}


