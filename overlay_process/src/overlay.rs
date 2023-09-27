use druid::piet::{Color, RenderContext, ImageFormat, InterpolationMode};
use druid::widget::Widget;
use druid::{Data, Env, EventCtx, Point, Rect, Selector, Lens, Event, LifeCycle, LifeCycleCtx, UpdateCtx, LayoutCtx, BoxConstraints, Size, Application};
use screenshots::Screen;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;
use druid::platform_menus::mac::file::print;

use crate::{IconData};


#[derive(PartialEq, Debug)]
pub enum OverlayState {
    Selecting,
    ButtonsShown,
}

pub struct ScreenshotOverlay {
    start_point: Option<Point>,
    end_point: Option<Point>,
    screen: Option<Screen>,
    overlay_state: OverlayState,
    icon_data: IconData,
}

impl ScreenshotOverlay {
    pub fn new(icon_data: IconData) -> Self {
        ScreenshotOverlay {
            start_point: None,
            end_point: None,
            screen: None,
            overlay_state: OverlayState::Selecting,
            icon_data,
        }
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = Some(screen);
    }

    pub fn is_point_in_screen(&self, point: Point, screen: &Screen, translation_factor: i32) -> bool {
        use druid::Value::Point;

        let screen_right = screen.display_info.x as i32 + screen.display_info.width as i32;
        let screen_left = screen.display_info.x as i32;

        //let screen_bottom = screen.display_info.y as i32 + screen.display_info.height as i32;

        let translated_point_x = point.x - translation_factor as f64;

        translated_point_x >= screen_left as f64 && translated_point_x <= screen_right as f64
            //&& point.y >= screen.display_info.y as f64
            //&& point.y <= screen_bottom as f64

    }

    pub fn show_buttons(&mut self) {
        self.overlay_state = OverlayState::ButtonsShown;
    }

    pub fn hide_buttons(&mut self) {
        self.overlay_state = OverlayState::Selecting;
    }

}

const SELECT_AREA: Selector<()> = Selector::new("select-area");


#[derive(Clone, Data, Lens)]
pub struct AppState {
    selection: Rect,
    screens: Arc<Vec<Screen>>,
    capture_channel: Arc<Mutex<Option<mpsc::Sender<(Rect, Screen, i32)>>>>,
}

impl AppState {
    pub fn new(screens: Arc<Vec<Screen>>, capture_channel: Arc<Mutex<Option<mpsc::Sender<(Rect, Screen, i32)>>>>) -> Self {
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

fn get_clicked_button(mouse_pos: Point, screen: Screen, data: &AppState) -> Option<Selector> {
    let icon_size = Size::new(32.0, 32.0);
    let (left_button_origin, middle_button_origin, right_button_origin) = get_button_position(screen, data, icon_size);

    if mouse_pos.is_inside_rect(left_button_origin, icon_size) {
        Some(BUTTON_A_CLICKED)
    } else if mouse_pos.is_inside_rect(middle_button_origin, icon_size) {
        Some(BUTTON_B_CLICKED)
    } else if mouse_pos.is_inside_rect(right_button_origin, icon_size) {
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

                let screens = &data.screens;

                // find the translation factor corresponding to the minimum value of x
                let mut translation_factor = i32::MAX;
                for screen in screens.iter() {
                    if screen.display_info.x < translation_factor {
                        translation_factor = screen.display_info.x
                    }
                }

                let p = druid::Point::new(data.selection.x0, data.selection.y0);
                for screen in screens.iter() {
                    if self.is_point_in_screen(p, screen, translation_factor.abs()) {
                        self.set_screen(screen.clone());
                        break;
                    }
                }




                if self.overlay_state == OverlayState::ButtonsShown {
                    let mouse_pos = mouse_event.pos;
                    if let Some(screen) = self.screen {
                        if let Some(button_clicked) = get_clicked_button(mouse_pos, screen, data) {
                            match button_clicked {
                                BUTTON_A_CLICKED => {
                                    println!("Button A clicked, sending message...");
                                    if let Ok(mut tx) = data.capture_channel.lock() {
                                        if let Some(tx) = tx.take() {
                                            // Notify the main thread to capture the screenshot
                                            if let Some(screen) = self.screen {
                                                tx.send((data.selection, screen, translation_factor)).expect("Failed to send message to main thread");
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
                        } else {
                            self.hide_buttons();
                        }
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
                ctx.request_paint();

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
        let bg_color = Color::rgba(0.0, 0.0, 0.0, 0.7);
        let selection_color = Color::rgba(0.0, 0.0, 0.0, 0.1);
        let border_color = Color::rgba(1.0, 1.0, 1.0, 0.5);
        let edge_color = Color::rgba(1.0, 1.0, 1.0, 1.0);

        // border
        let borders = create_border_rectangles(data.selection, 3.0);
        for border in borders {
            ctx.fill(border, &border_color);
        }

        // selection
        let result = surrounding_rectangles(size.to_rect().clone(), data.selection.clone());
        for element in result {
            ctx.fill(element, &bg_color);
        }
        ctx.fill(data.selection, &selection_color);

        // edges
        let edges = create_l_shaped_rectangles(data.selection, 40.0, 7.0, 25.0);
        for edge in edges {
            ctx.fill(edge, &edge_color);
        }




        if self.overlay_state == OverlayState::ButtonsShown {


            if let Some(screen) = self.screen {


                let icon_size = Size::new(32.0, 32.0);

                let (left_button_origin, middle_button_origin, right_button_origin) = get_button_position(screen, data, icon_size);

                let left_button_rect = Rect::from_origin_size(left_button_origin, icon_size);
                let middle_button_rect = Rect::from_origin_size(middle_button_origin, icon_size);
                let right_button_rect = Rect::from_origin_size(right_button_origin, icon_size);

                let image = ctx
                    .make_image(32, 32, &self.icon_data.save_icon, ImageFormat::Rgb)
                    .unwrap();
                ctx.draw_image(&image, left_button_rect, InterpolationMode::Bilinear);

                let image = ctx
                    .make_image(32, 32, &self.icon_data.boh_icon, ImageFormat::Rgb)
                    .unwrap();
                ctx.draw_image(&image, middle_button_rect, InterpolationMode::Bilinear);

                let image = ctx
                    .make_image(32, 32, &self.icon_data.quit_icon, ImageFormat::Rgb)
                    .unwrap();
                ctx.draw_image(&image, right_button_rect, InterpolationMode::Bilinear);
            }
       }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
        // Update the layout to account for the buttons below the selected area
        let mut size = bc.max();
        if self.overlay_state == OverlayState::ButtonsShown {
            let button_height = 32.0; // Height of the buttons
            size.height += button_height + 2.0; // 10.0 for spacing between buttons and selected area
        }
        size
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {}

}


fn get_button_position(screen: Screen, data: &AppState, icon_size: Size) -> (Point, Point, Point){


    let center = data.selection.center();
    let button_spacing = 50.0;

    let space_below = screen.display_info.height as f64 - data.selection.y1;
    let mut vertical_offset = data.selection.y1 + button_spacing;

    let available_space_below = space_below >= icon_size.height + button_spacing;

    if !available_space_below {
        vertical_offset = data.selection.y0 - icon_size.height - button_spacing;
    }

    let left_button_origin = Point::new(center.x - icon_size.width - button_spacing, vertical_offset);
    let middle_button_origin = Point::new(center.x - icon_size.width / 2.0, vertical_offset);
    let right_button_origin = Point::new(center.x + button_spacing, vertical_offset);

    (left_button_origin, middle_button_origin, right_button_origin)
}

fn surrounding_rectangles(A: Rect, B: Rect) -> Vec<Rect> {
    let mut result = Vec::new();

    // Calcola il rettangolo sopra B
    if B.y1 < A.y1 {
        result.push(Rect {
            x0: B.x0,
            x1: B.x1,
            y0: B.y1,
            y1: A.y1,
        });
    }

    // Calcola il rettangolo sotto B
    if B.y0 > A.y0 {
        result.push(Rect {
            x0: B.x0,
            x1: B.x1,
            y0: A.y0,
            y1: B.y0,
        });
    }

    // Calcola il rettangolo a sinistra di B
    if B.x0 > A.x0 {
        result.push(Rect {
            x0: A.x0,
            x1: B.x0,
            y0: A.y0,
            y1: A.y1,
        });
    }

    // Calcola il rettangolo a destra di B
    if B.x1 < A.x1 {
        result.push(Rect {
            x0: B.x1,
            x1: A.x1,
            y0: A.y0,
            y1: A.y1,
        });
    }

    result
}

fn create_border_rectangles(rect: Rect, border_width: f64) -> Vec<Rect> {
    let mut result = Vec::new();

    // Calcola il rettangolo superiore
    result.push(Rect {
        x0: rect.x0 - border_width,
        x1: rect.x1 + border_width,
        y0: rect.y0 - border_width,
        y1: rect.y0,
    });

    // Calcola il rettangolo inferiore
    result.push(Rect {
        x0: rect.x0 - border_width,
        x1: rect.x1 + border_width,
        y0: rect.y1,
        y1: rect.y1 + border_width,
    });

    // Calcola il rettangolo sinistro
    result.push(Rect {
        x0: rect.x0 - border_width,
        x1: rect.x0,
        y0: rect.y0,
        y1: rect.y1,
    });

    // Calcola il rettangolo destro
    result.push(Rect {
        x0: rect.x1,
        x1: rect.x1 + border_width,
        y0: rect.y0,
        y1: rect.y1,
    });

    result
}

fn create_l_shaped_rectangles(rect: Rect, length: f64, width: f64, padding: f64) -> Vec<Rect> {

    let mut result = Vec::new();

    // Rettangoli orizzontali superiori
    let top_left = Rect {
        x0: rect.x0 - padding + width, // Inverti il segno di x e y
        x1: rect.x0 - padding, // Inverti il segno di x e y
        y0: rect.y1 + padding - length, // Inverti il segno di x e y
        y1: rect.y1 + padding, // Inverti il segno di x e y
    };
    let top_right = Rect {
        x0: rect.x1 + padding, // Inverti il segno di x e y
        x1: rect.x1 + padding - width, // Inverti il segno di x e y
        y0: rect.y1 + padding - length, // Inverti il segno di x e y
        y1: rect.y1 + padding, // Inverti il segno di x e y
    };

    // Rettangoli orizzontali inferiori
    let bottom_left = Rect {
        x0: rect.x0 - padding + width, // Inverti il segno di x e y
        x1: rect.x0 - padding, // Inverti il segno di x e y
        y0: rect.y0 - padding, // Inverti il segno di x e y
        y1: rect.y0 - padding + length, // Inverti il segno di x e y
    };
    let bottom_right = Rect {
        x0: rect.x1 + padding, // Inverti il segno di x e y
        x1: rect.x1 + padding - width, // Inverti il segno di x e y
        y0: rect.y0 - padding, // Inverti il segno di x e y
        y1: rect.y0 - padding + length, // Inverti il segno di x e y
    };

    // Rettangoli verticali sinistri
    let left_top = Rect {
        x0: rect.x0 - padding + length, // Inverti il segno di x e y
        x1: rect.x0 - padding, // Inverti il segno di x e y
        y0: rect.y1 + padding - width, // Inverti il segno di x e y
        y1: rect.y1 + padding, // Inverti il segno di x e y
    };
    let left_bottom = Rect {
        x0: rect.x0 - padding + length, // Inverti il segno di x e y
        x1: rect.x0 - padding, // Inverti il segno di x e y
        y0: rect.y0 - padding, // Inverti il segno di x e y
        y1: rect.y0 - padding + width, // Inverti il segno di x e y
    };

    // Rettangoli verticali destri
    let right_top = Rect {
        x0: rect.x1 + padding, // Inverti il segno di x e y
        x1: rect.x1 + padding - length, // Inverti il segno di x e y
        y0: rect.y1 + padding - width, // Inverti il segno di x e y
        y1: rect.y1 + padding, // Inverti il segno di x e y
    };
    let right_bottom = Rect {
        x0: rect.x1 + padding, // Inverti il segno di x e y
        x1: rect.x1 + padding - length, // Inverti il segno di x e y
        y0: rect.y0 - padding, // Inverti il segno di x e y
        y1: rect.y0 - padding + width, // Inverti il segno di x e y
    };

    result.push(top_left);
    result.push(top_right);
    result.push(bottom_left);
    result.push(bottom_right);
    result.push(left_top);
    result.push(left_bottom);
    result.push(right_top);
    result.push(right_bottom);

    result
}