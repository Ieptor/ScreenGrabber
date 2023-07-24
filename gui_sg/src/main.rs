use druid::{Lens, Application, AppLauncher, LocalizedString, WindowDesc, Data, Widget, Selector, Handled, DelegateCtx, Env};
use druid::widget::Either;
use std::process::Command;

mod custom_widgets;
use custom_widgets::{initial_layout, save_path_layout, shortcut_layout};

mod utils;
use utils::read_config_file;


pub const HOME: Selector = Selector::new("my_app.home");
pub const LAUNCH_OVERLAY: Selector = Selector::new("my_app.launch_overlay");
pub const PATH_GUI: Selector = Selector::new("my_app.launch_pathgui");
pub const SHORTCUT_GUI: Selector = Selector::new("my_app.launch_shortcutgui");
pub const RUN_IN_BACKGROUND: Selector = Selector::new("my_app.launch_run_background");

#[derive(Clone, Data, Lens)]
pub struct MainState {
    // Add a flag to indicate whether the overlay should be launched
    launch_overlay: bool,
    path_gui: bool,
    shortcut_gui: bool,
    path: String,
    shortcut: String,
}

fn main() {
    let config_file_path = std::path::Path::new("../config/config.txt");
    let mut path = "target".to_string();
    let mut shortcut_string = "ctrl + k".to_string();

    match read_config_file(config_file_path) {
        Ok((savepath, shortcut)) => {
            path = savepath;
            shortcut_string = shortcut;
        },
        Err(_) => {
            eprintln!("Error reading config file");
        }
    }

    // Create the main window
    let main_window = WindowDesc::new(build_ui())
        .title(LocalizedString::new("Simple GUI"))
        .window_size((600.0, 400.0));

    // Launch the application
    let initial_state = MainState {
        launch_overlay: false,
        path_gui: false,
        shortcut_gui: false,
        path,
        shortcut: shortcut_string
    };

    AppLauncher::with_window(main_window)
        .delegate(Delegate {})
        .launch(initial_state)
        .expect("Failed to launch application");

}


fn build_ui() -> impl Widget<MainState> {
    
    let initial_layout = initial_layout();
    let save_path_layout = save_path_layout();
    let shortcut_layout = shortcut_layout();

    Either::new(
        // If both path_gui and shortcut_gui are false, show the initial layout
        |data: &MainState, _env: &Env| !data.path_gui && !data.shortcut_gui,
        initial_layout,
        // If path_gui is true and shortcut_gui is false, show the save_path_layout
        Either::new(
            |data: &MainState, _env: &Env| data.path_gui && !data.shortcut_gui,
            save_path_layout,
            // If shortcut_gui is true and path_gui is false, show the shortcut_layout
            shortcut_layout,
        ),
    )
}

struct Delegate;

impl druid::AppDelegate<MainState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut MainState,
        _env: &druid::Env,
    ) -> druid::Handled {
        if cmd.is(LAUNCH_OVERLAY) {
            // Set the flag to launch the overlay
            data.launch_overlay = true;
            //create a process that runs run_overlay() and then quit this gui
            Application::global().quit();

            // Launch the overlay binary as a new process
            let _ = Command::new(r"..\overlay_process\release\overlay_process.exe")
                .spawn()
                .expect("Failed to start overlay process");

            Handled::Yes
        } else if cmd.is(PATH_GUI) {
            data.launch_overlay = false;
            data.shortcut_gui = false;
            data.path_gui = true;
            Handled::Yes
        } else if cmd.is(SHORTCUT_GUI) {
            data.launch_overlay = false;
            data.path_gui = false;
            data.shortcut_gui = true;
            Handled::Yes
        } else if cmd.is(HOME){ 
            data.launch_overlay = false;
            data.shortcut_gui = false;
            data.path_gui = false;
            Handled::Yes
        }  else if cmd.is(RUN_IN_BACKGROUND){
            Application::global().quit();
            //run the background shortcut listener
            let _ = Command::new(r"..\background_listener\release\background_listener.exe")
            .spawn()
            .expect("Failed to start background listener");
            Handled::Yes
        }
        else {Handled::No}
    }

}


