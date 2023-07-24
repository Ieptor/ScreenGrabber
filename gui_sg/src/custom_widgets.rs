//internal dependencies

use crate::{MainState, RUN_IN_BACKGROUND, LAUNCH_OVERLAY, PATH_GUI, SHORTCUT_GUI, HOME};
use crate::utils::{save_to_config_file, ShortcutValidation, validate_shortcut};

//external dependencies

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

    let snip = Button::new("Snappa lo schermo")
        .on_click(|ctx, _, _| {
            ctx.submit_command(LAUNCH_OVERLAY);
        })
        .fix_height(50.0)
        .center();

    let background = Button::new("Vai in background")
        .on_click(|ctx, _, _| {
            ctx.submit_command(RUN_IN_BACKGROUND);
        })
        .fix_height(50.0)
        .center();


    let welcome_and_button = Flex::row()
        .with_flex_child(welcome_phrase, 1.0)
        .with_spacer(20.0)
        .with_flex_child(snip, 1.0)
        .with_flex_child(background, 1.0)
        .center();

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_flex_child(button_row, 1.0) 
        .with_spacer(80.0)
        .with_flex_child(welcome_and_button, 1.0)
        .padding(30.0)
}


pub fn save_path_layout() -> impl Widget<MainState> {
    let button_row = create_button_row();

    let text_input_widget = TextBox::new()
        .with_line_wrapping(false)
        .lens(MainState::path) 
        .fix_width(500.0)
        .fix_height(40.0)
        .padding(10.0);

    let browse_button = Button::new("Browse")
        .on_click(|_ctx, data: &mut MainState, _env| {
            if let Ok(Some(selected_directory)) = FileDialog::new().show_open_single_dir() {
                data.path = selected_directory.to_string_lossy().to_string(); 
                if let Some(dir) = selected_directory.to_str() {
                    let config_path =  std::path::Path::new("../config/config.txt");
                    match save_to_config_file(config_path, dir, "save_path") {
                        Ok(_) => {},
                        Err(_) => {show_message_box("Error", "An error occured in saving the path, retry.");}
                    }
                }
                                
            }
        })
        .padding(10.0);

    let text_and_button_row = Flex::row()
        .with_flex_child(text_input_widget, 1.0)
        .with_spacer(20.0)
        .with_flex_child(browse_button, 0.8)
        .center();

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
        format!("Shortcut: (ctrl + 'key')")
    });

    let shortcut_textbox = TextBox::new()
        .controller(ShortcutController)
        .lens(MainState::shortcut);
        

    let save_button = Button::new("Save")
        .on_click(|_ctx, data: &mut MainState, _env| {
            let config_path =  std::path::Path::new("../config/config.txt");
                let validation = validate_shortcut(&data.shortcut);
                match validation {
                    ShortcutValidation::Valid => {
                        match save_to_config_file(config_path, &data.shortcut, "shortcut") {
                            Ok(_) => {show_message_box("Saved", "Shortcut saved correctly.")},
                            Err(_) => {show_message_box("Error", "An error occured in saving the configuration, retry.")}
                        }
                    },
                    ShortcutValidation::Incomplete => {
                        show_message_box("Incomplete shortcut", "Shortcut chosen is incomplete, try ctrl + 'key'")
                    },
                    ShortcutValidation::Invalid => {
                        show_message_box("Invalid shortcut", "Shortcut chosen is invalid, try ctrl + 'key'")
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
                        if data == "ctrl +" && c.chars().all(|ch| ch.is_ascii_alphanumeric()){
                            *data = format!("ctrl + {}", c);
                        } 
                    },
                    KbKey::Control => {
                        data.clear();
                        *data = "ctrl +".to_string();
                        ctx.request_update();
                    },
                    _ => {}
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