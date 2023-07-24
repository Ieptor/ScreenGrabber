use druid::piet::{Color, RenderContext};
use druid::widget::{Widget};
use druid::{Data, Env, EventCtx, Point, Rect, Selector, Lens, Event, LifeCycle, LifeCycleCtx, UpdateCtx, LayoutCtx, BoxConstraints, Size, Application};
use screenshots::Screen;

use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;


#[derive(PartialEq)]
pub enum OverlayState {
    Selecting,
    ButtonsShown,
}

pub struct ScreenshotOverlay {
    start_point: Option<Point>,
    end_point: Option<Point>,
    screen: Option<Screen>,
    overlay_state: OverlayState,
}

impl ScreenshotOverlay {
    pub fn new() -> Self {
        ScreenshotOverlay {
            start_point: None,
            end_point: None,
            screen: None,
            overlay_state: OverlayState::Selecting,
        }
    }
    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = Some(screen);
    }

    pub fn is_point_in_screen(&self, point: Point, screen: &Screen) -> bool {
        let screen_right = screen.display_info.x as u32 + screen.display_info.width;
        let screen_bottom = screen.display_info.y as u32 + screen.display_info.height;
        point.x >= screen.display_info.x as f64
            && point.x <= screen_right as f64
            && point.y >= screen.display_info.y as f64
            && point.y <= screen_bottom as f64
    }

    pub fn show_buttons(&mut self) {
        self.overlay_state = OverlayState::ButtonsShown;
    }

}

const SELECT_AREA: Selector<()> = Selector::new("select-area");


#[derive(Clone, Data, Lens)]
pub struct AppState {
    selection: Rect,
    screens: Arc<Vec<Screen>>,
    capture_channel: Arc<Mutex<Option<mpsc::Sender<(Rect, Screen)>>>>
}

impl AppState {
    pub fn new(screens: Arc<Vec<Screen>>, capture_channel: Arc<Mutex<Option<mpsc::Sender<(Rect, Screen)>>>>) -> Self {
        AppState {
            selection: Rect::ZERO,
            screens,
            capture_channel
        }
    }
}

trait IsInsideRect {
    fn is_inside_rect(&self, origin: Point, size: Size) -> bool;
}

impl IsInsideRect for Point {
    fn is_inside_rect(&self, origin: Point, size: Size) -> bool {
        self.x >= origin.x && self.x <= origin.x + size.width && self.y >= origin.y && self.y <= origin.y + size.height
    }
}

const BUTTON_A_CLICKED: Selector<()> = Selector::new("button-a-clicked");
const BUTTON_B_CLICKED: Selector<()> = Selector::new("button-b-clicked");
const BUTTON_C_CLICKED: Selector<()> = Selector::new("button-c-clicked");

fn get_clicked_button(mouse_pos: Point, data: &AppState) -> Option<Selector> {
    let button_size = Size::new(30.0, 30.0); // Adjust the button size as needed
    let button_spacing = 50.0; // Adjust the spacing between buttons as needed

    let center = data.selection.center();
    let left_button_origin = Point::new(center.x - button_size.width - button_spacing, data.selection.y1 + button_spacing);
    let middle_button_origin = Point::new(center.x - button_size.width / 2.0, data.selection.y1 + button_spacing);
    let right_button_origin = Point::new(center.x + button_spacing, data.selection.y1 + button_spacing);

    if mouse_pos.is_inside_rect(left_button_origin, button_size) {
        Some(BUTTON_A_CLICKED)
    } else if mouse_pos.is_inside_rect(middle_button_origin, button_size) {
        Some(BUTTON_B_CLICKED)
    } else if mouse_pos.is_inside_rect(right_button_origin, button_size) {
        Some(BUTTON_C_CLICKED)
    } else {
        None
    }
}

impl Widget<AppState> for ScreenshotOverlay {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::MouseDown(mouse_event) => {
                self.start_point = Some(mouse_event.pos);

                //which screen?

                let screens = &data.screens;
                for screen in screens.iter() {
                    if self.is_point_in_screen(mouse_event.pos, screen) {
                        self.set_screen(screen.clone());
                        break;
                    }
                }

                
                if self.overlay_state == OverlayState::ButtonsShown {
                    let mouse_pos = mouse_event.pos;
                    if let Some(button_clicked) = get_clicked_button(mouse_pos, data) {
                        match button_clicked {
                            BUTTON_A_CLICKED => {
                                println!("Button A clicked, sending message...");
                                if let Ok(mut tx) = data.capture_channel.lock() {
                                    if let Some(tx) = tx.take() {
                                        // Notify the main thread to capture the screenshot
                                        if let Some(screen) = self.screen {
                                            tx.send((data.selection, screen)).expect("Failed to send message to main thread");
                                            drop(tx);
                                            Application::global().quit();
                                        }
                                    }
                                }
                            },
                            BUTTON_B_CLICKED => {println!("Button B clicked. Color: Green");
                            Application::global().quit();
                            },
                            BUTTON_C_CLICKED => {Application::global().quit();}
                            _ => {}
                        }
                        ctx.set_handled();
                    }
                }
                
                ctx.set_active(true);
                ctx.set_handled();
                
            }

            Event::MouseUp(mouse_event) => {
                self.end_point = Some(mouse_event.pos);
                ctx.set_active(false);
                ctx.submit_command(SELECT_AREA.to_owned());
                ctx.set_handled();

                self.show_buttons();
            }
            Event::MouseMove(mouse_event) => {
                if ctx.is_active() {
                    if let Some(start) = self.start_point {
                        self.end_point = Some(mouse_event.pos);
                        let selection = Rect::from_points(start, mouse_event.pos);
                        data.selection = selection;
                        ctx.request_paint();
                    }
                }
            }
        
            _ => (),
        }
    }
    

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppState, _env: &Env) {

        let size = ctx.size();
        let bg_color = Color::rgba(0.0, 0.0, 0.0, 0.4);
        ctx.fill(size.to_rect(), &bg_color);

        let selection_color = Color::rgba(0.0, 0.0, 0.8, 0.3);
        ctx.fill(data.selection, &selection_color);

        if self.overlay_state == OverlayState::ButtonsShown {
            // Draw buttons below the selected area
            let button_size = Size::new(30.0, 30.0); // Adjust the button size as needed
            let button_spacing = 50.0; // Adjust the spacing between buttons as needed

            let center = data.selection.center();
            let left_button_origin = Point::new(center.x - button_size.width - button_spacing, data.selection.y1 + button_spacing);
            let middle_button_origin = Point::new(center.x - button_size.width / 2.0, data.selection.y1 + button_spacing);
            let right_button_origin = Point::new(center.x + button_spacing, data.selection.y1 + button_spacing);

            // Define colors for the buttons
            let button_color_a = Color::rgba(0.8, 0.0, 0.0, 0.8);
            let button_color_b = Color::rgba(0.0, 0.8, 0.0, 0.8);
            let button_color_c = Color::rgba(0.0, 0.0, 0.8, 0.8);

            // Draw the buttons as rectangles
            let button_a_rect = Rect::from_origin_size(left_button_origin, button_size);
            ctx.fill(button_a_rect, &button_color_a);

            let button_b_rect = Rect::from_origin_size(middle_button_origin, button_size);
            ctx.fill(button_b_rect, &button_color_b);

            let button_c_rect = Rect::from_origin_size(right_button_origin, button_size);
            ctx.fill(button_c_rect, &button_color_c);
       }
}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
        // Update the layout to account for the buttons below the selected area
        let mut size = bc.max();
        if self.overlay_state == OverlayState::ButtonsShown {
            let button_height = 30.0; // Height of the buttons
            size.height += button_height + 10.0; // 10.0 for spacing between buttons and selected area
        }
        size
    }
    

    
    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {}

   

}

