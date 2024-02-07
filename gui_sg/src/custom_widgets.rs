//internal dependencies

use crate::{MainState, RUN_IN_BACKGROUND, LAUNCH_OVERLAY, PATH_GUI, SHORTCUT_GUI, HOME, FULLSCREEN, DELAY};
use crate::utils::{save_to_config_file, ShortcutValidation, validate_shortcuts};
use overlay_process::utils::{get_config_file_path};


//external dependencies

use druid::widget::{Button, Flex, WidgetExt, Label, CrossAxisAlignment, TextBox, Controller, SvgData, Svg};
use native_dialog::{FileDialog, MessageDialog};
use druid::{EventCtx, Event, KbKey, Widget, Color, Selector, Menu, LocalizedString, MenuItem, Target};
use druid::widget::prelude::*;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use druid::Point;
use druid::Rect;
use druid::piet::{Text,TextLayoutBuilder};
use druid::MouseButton;

const HOME_ICON_SVG_STR: &str = include_str!("./icons/home-icon.svg");
const SAVEPATH_ICON_SVG: &str = include_str!("./icons/save-icon.svg");
const SHORTCUT_ICON_SVG: &str = include_str!("./icons/keyboard-icon.svg");
const SCREENSHOT_ICON_SVG: &str = include_str!("./icons/snip-icon.svg");
const BACKGROUND_ICON_SVG: &str = include_str!("./icons/background-icon.svg");
const FULLSCREEN_ICON_SVG: &str = include_str!("./icons/fullscreen-icon.svg");
const DELAY_ICON_SVG: &str = include_str!("./icons/delay-icon.svg");


fn show_message_box(title: &str, message: &str) {
    MessageDialog::new()
        .set_title(title)
        .set_text(message)
        .show_alert()
        .expect("Failed to show the message box.");
}

struct IconButton {
    icon: SvgData,
    label: String,
    command: Selector,
    main_button: bool,
}



impl IconButton {
    fn new(icon: SvgData, label: String, command: Selector, main_button: bool) -> Self {
        Self {
            icon,
            label,
            command,
            main_button,
        }
    }
    fn layout_bounds(&self, origin: Point, size: Size) -> Rect {
        Rect::from_origin_size(origin, size)
    }

    fn show_delay_menu(&self, ctx: &mut EventCtx, mouse_pos: Point, data:  &mut MainState) {
        // Create a channel and store the sender in menu_channel
        let (tx, rx) = mpsc::channel();
        let tx1 = tx.clone();
        let tx2 = tx.clone();
        let tx3 = tx.clone();

        let mut s0 = "no delay";
        let mut s1 = "1s";
        let mut s3 = "3s";
        let mut s5 = "5s";

        match data.delay_state {
            0 => {s0 = "● no delay"}
            1 => {s1 = "● 1s"}
            3 => {s3 = "● 3s"}
            5 => {s5 = "● 5s"}
            _ => {}
        }

        let base: Menu<MainState> = Menu::new(LocalizedString::new("Delay"));
        let delay_menu = base
                .entry(MenuItem::new(LocalizedString::new(s0)).on_activate(move |_, _, _| {
                    tx3.send(0 as u32).unwrap();
                }))
                .entry(MenuItem::new(LocalizedString::new(s1)).on_activate(move |_, _, _| {
                    tx.send(1 as u32).unwrap();
                }))
                .entry(MenuItem::new(LocalizedString::new(s3)).on_activate(move |_, _, _| {
                    tx1.send(3 as u32).unwrap();
                }))
                .entry(MenuItem::new(LocalizedString::new(s5)).on_activate(move |_, _, _| {
                    tx2.send(5 as u32).unwrap();
                }));
        

        // Show the menu at the mouse position
        let adjusted_pos = Point::new(mouse_pos.x + 350.0, mouse_pos.y);
        ctx.show_context_menu(delay_menu, adjusted_pos);

        // Spawn a thread to wait for messages and handle them
        let ext_event_sink = ctx.get_external_handle();

        thread::spawn(move || {
            // This will block until a message is received
            let received_command = match rx.recv() {
                Ok(cmd) => cmd,
                Err(_err) => {//gestire meglio questo errore TODO
                    10
                }
            };
            match received_command {
                0 | 1 | 3 | 5 => {
                    ext_event_sink
                        .submit_command(DELAY, received_command, Target::Global)
                        .expect("Failed to submit DELAY command");
                }
                _ => {
                    // Handle unknown command
                }
            }
        });      
    }

}

impl Widget<MainState> for IconButton {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut MainState, _env: &Env) {
        match event {
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left && !ctx.is_handled() {
                    // Check if the mouse down event occurred inside the IconButton bounds.
                    let bounds = self.layout_bounds(Point::ORIGIN, ctx.size());
                    let hit_test_result = bounds.contains(mouse_event.pos);
                    if hit_test_result {
                        // Handle the onclick event here.
                        // For example, you can submit a command when the IconButton is clicked.
                        if self.label == "Delay" {
                            self.show_delay_menu(ctx, mouse_event.pos, data);
                        } else {
                            ctx.submit_command(self.command);

                        }
                        ctx.set_active(true);
                        ctx.request_paint();
                    }
                }
            }
            _ => {}
        }
    }

    // Use UpdateCtx instead of EventCtx for update method
    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &MainState, _data: &MainState, _env: &Env) {
        // Perform any updates for the button here if needed.
    }

    // Use LifeCycleCtx instead of EventCtx for lifecycle method
    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &MainState, _env: &Env) {
        // Handle lifecycle events for the button here if needed.
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &MainState,
        _env: &Env,
    ) -> Size {
        // Layout the child widgets (Svg and Label) and return the size.

        if self.main_button {
            let icon_size = Size::new(120.0, 45.0); 
            let _label_size = Size::new(140.0, 45.0); 
            Size::new(icon_size.width + 50.0, icon_size.height)

        } else {
            let icon_size = bc.constrain(Size::new(50.0, 50.0)); 
            let label_size = bc.constrain(Size::new(50.0, 10.0)); 
            let total_height = icon_size.height + label_size.height + 2.0; 
            Size::new(icon_size.width, total_height)
        }
        
       
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &MainState, env: &Env) {

        if !self.main_button{
            let  icon_size = Size::new(50.0, 50.0); // Adjust the size as needed.
            let  label_size = Size::new(50.0, 10.0); // Adjust the size as needed.        

            let total_height = icon_size.height + label_size.height + 1.0; // Add spacing between icon and label.
            // Paint the Svg icon.
            let icon_origin = Point::new(
                (ctx.size().width - icon_size.width) / 2.0,
                (ctx.size().height - total_height) / 2.0,
            );
        
            // Paint the Svg icon.
            let _icon_rect = Rect::from_origin_size(icon_origin, icon_size);
            ctx.with_save(|ctx| {
                Svg::new(self.icon.clone()).paint(ctx, data, env);
            });
            
            let label_text = self.label.clone();
            let offset_x: f64;
            let offset_y: f64;

            if label_text == "Home" {
                offset_x = 8.0;
                offset_y = 6.0;
            } else if label_text == "Path" {
                offset_x = 12.0;
                offset_y = 6.0;
            } else if label_text == "Shortcuts" {
                offset_x = 1.0;
                offset_y = 6.0;
            } else if label_text == "Delay" {
                offset_x = 8.0;
                offset_y = 6.0;
            }
            else {
                offset_x = 0.0;
                offset_y = 0.0;
            }

            let label_origin: Point = Point::new(
                offset_x + icon_origin.x + (icon_size.width - label_size.width)  / 2.0, 
                icon_origin.y + icon_size.height + offset_y, 
            );
            if let Ok(text_layout) = ctx
            .text()
            .new_text_layout(label_text)
            .max_width(label_size.width)
            .text_color(Color::BLACK)
            .build()
            {
                ctx.draw_text(&text_layout, label_origin);
            }
        } else {
            let  icon_size = Size::new(200.0, 50.0); // Adjust the size as needed.
                ctx.with_save(|ctx| {
                    Svg::new(self.icon.clone()).paint(ctx, data, env);
                });

            let label_text = self.label.clone();
            let label_origin: Point = Point::new(
                icon_size.width / 3.0 + 10.0, 
                icon_size.height / 3.0, 
            );

            if let Ok(text_layout) = ctx
            .text()
            .new_text_layout(label_text)
            .max_width(140.0)
            .text_color(Color::BLACK)
            .build()
            {
                ctx.draw_text(&text_layout, label_origin);
            }
        }    
    }
}

pub fn create_button_row() -> impl Widget<MainState> {

    let home_icon_svg: SvgData = SvgData::from_str(HOME_ICON_SVG_STR).expect("failed");
    let save_icon_svg: SvgData = SvgData::from_str(SAVEPATH_ICON_SVG).expect("failed");
    let shortcut_icon_svg: SvgData = SvgData::from_str(SHORTCUT_ICON_SVG).expect("failed"); 
    let delay_icon = SvgData::from_str(DELAY_ICON_SVG).expect("failed");


    Flex::row()
        .with_child(IconButton::new(home_icon_svg, "Home".to_string(), HOME, false))
        .with_spacer(60.0)
        .with_child(IconButton::new(save_icon_svg, "Path".to_string(), PATH_GUI, false))
        .with_spacer(60.0)
        .with_child(IconButton::new(shortcut_icon_svg, "Shortcuts".to_string(), SHORTCUT_GUI, false))
        .with_spacer(60.0)
        .with_child(IconButton::new(delay_icon, "Delay".to_string(), HOME, false))
        .padding(10.0)
        //.background(Color::BLACK)
}


pub fn initial_layout() -> impl Widget<MainState> {
    let button_row = create_button_row();

    let snip_icon: SvgData = SvgData::from_str(SCREENSHOT_ICON_SVG).expect("failed");
    let background_icon: SvgData = SvgData::from_str(BACKGROUND_ICON_SVG).expect("failed");
    let full_screenshot_icon: SvgData = SvgData::from_str(FULLSCREEN_ICON_SVG).expect("failed");

    let snip = Flex::row()
               .with_child(IconButton::new(snip_icon, "Take a screenshot".to_string(), LAUNCH_OVERLAY, true));
    let background = Flex::row()
                    .with_child(IconButton::new(background_icon, "Run in background".to_string(), RUN_IN_BACKGROUND, true));
    let fullscreen = Flex::row()
                .with_child(IconButton::new(full_screenshot_icon, "Capture fullscreen".to_string(), FULLSCREEN, true));
    
    let cop = Label::new(|_data: &MainState, _env: &_| {
                    format!("SnipGrab by: Pietro & Kevin")}).with_text_color(Color::BLACK);

    let function_column = Flex::column()
        .with_flex_child(snip, 1.0)
        .with_spacer(25.0)
        .with_flex_child(background, 1.0)
        .with_spacer(25.0)
        .with_flex_child(fullscreen, 1.0)
        .with_spacer(35.0)
        .with_flex_child(cop, 1.0);


    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_flex_child(button_row, 1.0) 
        .with_spacer(60.0)
        .with_flex_child(function_column, 3.0)
}

pub fn save_path_layout() -> impl Widget<MainState> {
    let button_row = create_button_row();

    let label = Label::new(|_data: &MainState, _env: &_| {
        format!("Choose save directory")
    }).with_text_color(Color::BLACK);

    let text_input_widget = TextBox::new()
        .with_text_color(Color::WHITE)
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

                    let config_path = get_config_file_path();
                    match save_to_config_file(&config_path, dir, "save_path") {
                        Ok(_) => {show_message_box("Saved", "Path saved correctly.");},
                        Err(_) => {show_message_box("Error", "An error occurred in saving the path, retry.");}
                    }
                }
                                
            }
        })
        .fix_width(80.0);

    let col = Flex::column()
        .with_flex_child(label, 1.0)
        .with_spacer(15.0)
        .with_flex_child(text_input_widget, 1.0)
        .with_spacer(5.0)
        .with_flex_child(browse_button, 1.0);
        
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_flex_child(button_row, 1.0)
        .with_spacer(60.0)
        .with_flex_child(col, 3.0)
}

pub fn shortcut_layout() -> impl Widget<MainState> {
    let button_row = create_button_row();

    let label = Label::new(|_data: &MainState, _env: &_| {
        format!("Shortcuts: (ctrl + 'key')")
    }).with_text_color(Color::BLACK);


    let shortcut_label = Label::new(|_data: &MainState, _env: &_| {
        format!("Background listener:")
    }).with_text_color(Color::BLACK);

    let shortcut_textbox = TextBox::new()
        .controller(ShortcutController)
        .lens(MainState::bg_shortcut);

    let full_screen_label = Label::new(|_data: &MainState, _env: &_| {
        format!("Fullscreen screenshot:")
    }).with_text_color(Color::BLACK);

    let full_screen_textbox = TextBox::new()
        .controller(ShortcutController)
        .lens(MainState::fs_shortcut);

    let apply_button = Button::new("Apply")
        .on_click(|_ctx, data: &mut MainState, _env| {
            let config_path = get_config_file_path();
                let validation = validate_shortcuts(&data.bg_shortcut, &data.fs_shortcut);
                let mut flag = false;
                match validation {
                    ShortcutValidation::Valid => {
                        match save_to_config_file(&config_path, &data.bg_shortcut, "bg_shortcut") {
                            Ok(_) => flag = true,
                            Err(_) => {show_message_box("Error", "An error occurred in saving the configuration, retry.")}
                        }
                        match save_to_config_file(&config_path, &data.fs_shortcut, "fs_shortcut") {
                            Ok(_) => flag = true,
                            Err(_) => {show_message_box("Error", "An error occurred in saving the configuration, retry.")}
                        }

                        if flag {
                            show_message_box("Saved", "Shortcut saved correctly.");
                        }
                    },
                    ShortcutValidation::Incomplete => {
                        show_message_box("Incomplete shortcut", "A shortcut chosen is incomplete, try ctrl + 'key'")
                    },
                    ShortcutValidation::Invalid => {
                        show_message_box("Invalid shortcut", "A shortcut chosen is invalid, try ctrl + 'key'")
                    },
                    ShortcutValidation::Same => {
                        show_message_box("Same shortcuts", "The two shortcuts must be different")
                    }
                }
        })
        .padding(10.0);

    let col = Flex::column()
        .with_flex_child(label, 1.0)
        .with_spacer(15.0)
        .with_flex_child(shortcut_label, 1.0)
        .with_spacer(10.0)
        .with_flex_child(shortcut_textbox, 1.0)
        .with_spacer(15.0)
        .with_flex_child(full_screen_label, 1.0)
        .with_spacer(10.0)
        .with_flex_child(full_screen_textbox, 1.0)
        .with_spacer(5.0)
        .with_flex_child(apply_button, 2.0);


    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_flex_child(button_row, 1.0)
        .with_spacer(30.0)
        .with_flex_child(col, 3.0)

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
        match event {
            Event::KeyDown(_) => {},
            _ => {child.event(ctx, event, data, env);}
        }
    }
}