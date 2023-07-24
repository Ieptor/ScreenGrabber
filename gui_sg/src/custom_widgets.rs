use crate::{MainState, LAUNCH_OVERLAY, PATH_GUI, SHORTCUT_GUI, HOME};
use crate::utils::save_to_config_file;

use druid::widget::{Button, Flex, WidgetExt, Label, CrossAxisAlignment, TextBox, Controller};
use native_dialog::{FileDialog, MessageDialog};
use druid::{EventCtx, Event, KbKey, Widget};
use druid::widget::prelude::*;


fn show_message_box(title: &str, message: &str) {
    MessageDialog::new()
        .set_title(title)
        .set_text(message)
        .show_alert()
        .expect("Failed to show the message box.");
}

pub fn create_button_row() -> impl Widget<MainState> {
    Flex::row()
        .with_child(Button::new("Home").fix_width(100.0)
        .on_click(|ctx, _, _| {
            ctx.submit_command(HOME);
        }))
        .with_spacer(40.0)
        .with_child(Button::new("Savepath").fix_width(100.0)
        .on_click(|ctx, _, _| {
            ctx.submit_command(PATH_GUI);
        }))
        .with_spacer(40.0)
        .with_child(Button::new("Shortcut").fix_width(100.0)
        .on_click(|ctx, _, _| {
            ctx.submit_command(SHORTCUT_GUI);
        }))
}


pub fn initial_layout() -> impl Widget<MainState> {
    let button_row = create_button_row();
    let welcome_phrase = Label::new("Placeholder GUI")
        .with_text_size(24.0)
        .expand_width()
        .center();

    let main_button = Button::new("Snappa lo schermo")
        .on_click(|ctx, _, _| {
            ctx.submit_command(LAUNCH_OVERLAY);
        })
        .fix_height(50.0)
        .center();

    let welcome_and_button = Flex::row()
        .with_flex_child(welcome_phrase, 1.0)
        .with_spacer(20.0)
        .with_flex_child(main_button, 1.0)
        .center();

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_flex_child(button_row, 1.0) // Set to 0.0 to not take extra vertical space
        .with_spacer(80.0)
        .with_flex_child(welcome_and_button, 1.0)
        .padding(30.0)
}


pub fn save_path_layout() -> impl Widget<MainState> {
    let button_row = create_button_row();

    let text_input_widget = TextBox::new()
        .with_line_wrapping(false)
        .lens(MainState::path) // Connect the widget to the state using a lens
        .fix_width(500.0)
        .fix_height(40.0)
        .padding(10.0);

    let browse_button = Button::new("Browse")
        .on_click(|_ctx, data: &mut MainState, _env| {
            // Create options for the file dialog to select directories
            if let Ok(Some(selected_directory)) = FileDialog::new().show_open_single_dir() {
                data.path = selected_directory.to_string_lossy().to_string(); 
                if let Some(dir) = selected_directory.to_str() {
                    let config_path =  std::path::Path::new("../config/config.txt");
                    match save_to_config_file(config_path, dir, "save_path") {
                        Ok(_) => {},
                        Err(_) => {eprintln!("Error saving path to config file");
                            show_message_box("Error", "An error occured in saving the path, retry.");
                        }
                    }
                }
                                
            }
        })
        .padding(10.0);

    // Create a Row with the TextBox and Browse button
    let text_and_button_row = Flex::row()
        .with_flex_child(text_input_widget, 1.0)
        .with_spacer(20.0)
        .with_flex_child(browse_button, 0.8)
        .center();

    // Create the main Flex column layout
    Flex::column()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Center)
        .with_flex_child(button_row, 1.0)
        .with_spacer(80.0)
        .with_flex_child(text_and_button_row, 1.0)
        .padding(30.0)
}

pub fn shortcut_layout() -> impl Widget<MainState> {
    let button_row = create_button_row();

    let shortcut_label = Label::new(|_data: &MainState, _env: &_| {
        // Display the current chosen shortcut as a string
        format!("Shortcut:")
    });

    let shortcut_textbox = TextBox::new()
        .controller(ShortcutController)
        .lens(MainState::shortcut);
        

    let save_button = Button::new("Save")
        .on_click(|_ctx, data: &mut MainState, _env| {
            let config_path =  std::path::Path::new("../config/config.txt");
                println!{"{}", data.shortcut};
                match save_to_config_file(config_path, &data.shortcut, "shortcut") {
                        Ok(_) => {show_message_box("Saved", "Shortcut saved correctly.")},
                        Err(_) => {eprintln!("Error saving path to config file");
                            show_message_box("Error", "An error occured in saving the configuration, retry.");
                        }
                }
        })
        .padding(10.0);


    Flex::column()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Center)
        .with_flex_child(button_row, 1.0)
        .with_spacer(80.0)
        .with_flex_child(shortcut_label, 1.0)
        .with_spacer(20.0)
        .with_flex_child(shortcut_textbox, 1.0)
        .with_flex_child(save_button, 1.0)
        .padding(30.0)
}


struct ShortcutController;
impl<W: Widget<String>> Controller<String, W> for ShortcutController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut String,
        env: &Env,
    ) {
        match event {
            Event::KeyDown(key_event) => {
                match &key_event.key {
                    KbKey::Character(c) => {
                        if data == "ctrl +" {
                            *data = format!("ctrl + {}", c);
                        } 
                    },
                    KbKey::Control => {
                        data.clear();
                        *data = "ctrl +".to_string();
                        ctx.request_update();
                    },
                    _ => {
                        data.clear();
                    }
                }
                ctx.set_handled();  
            }
            _ => {}
        }
        // Delegate other events to the child
        match event {
            Event::KeyDown(_) => {},
            _ => {child.event(ctx, event, data, env);}
        }
    }
}