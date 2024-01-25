use druid::{Data, Widget, Event, EventCtx, PaintCtx, Size, Env, Rect, Point};
use druid::piet::{Piet, RenderContext, ImageFormat, InterpolationMode, Color, LinearGradient, UnitPoint};
use crate::ImageData;
use image::{GenericImageView};
use druid::theme;
use crate::utils::{save_edited_image, Stroke, blend_images};
use druid::kurbo::{PathEl, BezPath};
use image::{DynamicImage, Rgba, RgbaImage};
use druid::ImageBuf;
use image::ColorType;
use image::{Rgb, ImageBuffer};


pub struct Edit {
    images: ImageData,
    dest_rect: Option<Rect>,
    initial_point: Option<Point>,
    movement_points: Vec<Point>,//Vec<BezPath>,
    color: (Rgba<u8>, u32),
    scaling_factors: (f64, f64),
    drawing: bool,
    list_of_edits: (Vec<DynamicImage>, usize), //list of images with operations to undo
    temp_edit: Option<DynamicImage>,
}

impl Edit {
    pub fn new(images: ImageData) -> Self {
        let screenshot_clone = images.screenshot.clone();
        Edit {
            images: images,
            dest_rect: None,
            initial_point: None,
            movement_points: Vec::new(),
            color: (Rgba([0, 0, 0, 255]), 10),
            scaling_factors: (0.0, 0.0),
            drawing: false,
            list_of_edits: (vec![screenshot_clone], 1),
            temp_edit: None,
        }
    }

}

#[derive(Clone, Data)]
pub struct AppState;

impl AppState {
    pub fn new() -> Self {
        AppState 
    }
}

trait IsInsideRect {
    fn is_inside_rect(&self, origin: Point, size: Size) -> bool;
    fn is_inside_direct_rect(&self, rect: Rect) -> bool;
}

impl IsInsideRect for Point {
    fn is_inside_rect(&self, origin: Point, size: Size) -> bool {
        self.x >= origin.x && self.x <= origin.x + size.width && self.y >= origin.y && self.y <= origin.y + size.height
    }
    fn is_inside_direct_rect(&self, rect: Rect) -> bool {
        self.x >= rect.x0 && self.x <= rect.x1 && self.y >= rect.y0 && self.y <= rect.y1
    }
}

fn get_clicked_button(mouse_pos: Point, data: &AppState, ctx: &EventCtx) -> Option<u8> {
    let size = ctx.size();
    let icon_size = Size::new(64.0, 64.0);
    let arrows_size = Size::new(32.0, 64.0);
    let (save, text, highlight, shapes, back, forward) = get_buttons_positions(size);

    if mouse_pos.is_inside_rect(save, icon_size) {
        Some(3)
    }
    else if mouse_pos.is_inside_rect(text, icon_size) {
        Some(0)
    } else if mouse_pos.is_inside_rect(highlight, icon_size) {
        Some(1)
    } else if mouse_pos.is_inside_rect(shapes, icon_size) {
        Some(2)
    } else if mouse_pos.is_inside_rect(back, arrows_size) {
        Some(4)
    } else if mouse_pos.is_inside_rect(forward, arrows_size) {
        Some(5)
    } else {
        None
    }
}

impl Widget<AppState> for Edit {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::MouseDown(mouse_event) => {
                let mouse_pos = mouse_event.pos;
                let dest_rect = self.dest_rect;
                
                if let Some(button_clicked) = get_clicked_button(mouse_pos, data, ctx) {
                    match button_clicked {
                        0 => {
                            self.color = (Rgba([0, 0, 0, 255]), 10);
                        },
                        1 => {
                            self.color = (Rgba([255, 165, 0, 128]), 20);
                        },
                        2 => println!("shapes icon clicked"),
                        3 => {
                            let _ = save_edited_image(self.images.screenshot.clone(), r"C:\Users\pganc\Desktop");    
                        },
                        4 => {
                            //Undo functionality
                            let (edits, index) = &self.list_of_edits;
                            if *index > 1 {
                                let undo_index = *index - 1;
                                self.images.screenshot = edits[undo_index-1].clone();
                                self.list_of_edits = (edits.clone(), undo_index);
                                ctx.request_paint();
                            }
                        },
                        5 => {
                            //Redo functionality
                            let (edits, index) = &self.list_of_edits;
                            if *index < 5 && (edits.len() != *index) {
                                let redo_index = *index + 1;
                                self.images.screenshot = edits[redo_index-1].clone();
                                self.list_of_edits = (edits.clone(), redo_index);
                                ctx.request_paint();
                            }
                        },
                        _ => {},
                        }
                } 
                
                if let Some(dest_rect) = self.dest_rect {
                    if mouse_pos.is_inside_direct_rect(dest_rect) {
                        self.initial_point = Some(mouse_pos);
                        self.drawing = true;
                
                        self.movement_points.push(mouse_pos);
                        ctx.set_active(true);
                    }    
                }
                
                
            },

            Event::MouseUp(mouse_event) => {
                // Stop capturing mouse movement when the mouse is released
                if self.drawing {
                    if let Some(temp) = &self.temp_edit {
                        let blended = blend_images(self.images.screenshot.clone(), temp.clone());
                        self.images.screenshot = blended.clone(); //update blended image
                        self.temp_edit = None;
                        ctx.request_paint();

                        //update edit list for undo/redo
                        let (edits, index) = &self.list_of_edits;
                        let mut edits_clone = edits.clone(); 

                        if *index == 5 {
                            edits_clone.remove(0);
                            edits_clone.push(blended.clone());
                            self.list_of_edits = (edits_clone, *index);
                        } else {
                            let truncate_index = *index;
                            edits_clone.truncate(truncate_index);
                            edits_clone.push(blended.clone());
                            let new_index = edits_clone.len();
                            self.list_of_edits = (edits_clone, new_index);
                        }
                    }
                    
                    //reset
                    self.initial_point = None;
                    self.movement_points = Vec::new();
                    self.drawing = false;
                    
                    ctx.set_active(false);
                }
            },

            Event::MouseMove(mouse_event) => {
                if ctx.is_active() && self.drawing {
                    if let Some(dest_rect) = self.dest_rect {
                        let mouse_pos = mouse_event.pos;
                        self.movement_points.push(mouse_pos);
                    }

                    if self.movement_points.len() > 2 {
                            let (translation_x, translation_y) = self.scaling_factors;
                        
                            let converted_points: Vec<Point> = self.movement_points
                            .iter()
                            .map(|&point| Point::new(point.x + translation_x, point.y + translation_y))
                            .collect();
    
                            let (color, marker) = self.color;
                            let strokes = Stroke::new(converted_points, color, marker);
                            let (width, height) = &self.images.screenshot.dimensions();

                            self.temp_edit = Some(image::DynamicImage::ImageRgba8(strokes.draw((*width, *height))));
                            ctx.request_paint();
                    }
                }
            }

            _ => (),
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        let size = ctx.size();

        let icon_width = 64.0;
        let icon_height = 64.0;

        let backforward = 32.0;

        let total_width = 5.0 * icon_width;
        let spacing = (size.width - total_width) / 6.0;

        // Calculate the top position for the icons
        let icon_top_position = Point::new(spacing, 25.0);

        for i in 0..4 {
            // Create an image for each icon in the loop
            let icon = ctx.make_image(icon_width as usize, icon_height as usize, &self.images.icons[i].to_rgba(), ImageFormat::RgbaSeparate)
                        .unwrap();

            // Calculate the destination rectangle for each icon
            let icon_dest_rect = Rect::from_origin_size(
                Point::new(icon_top_position.x + (icon_width + spacing) * i as f64, icon_top_position.y),
                Size::new(icon_width, icon_height),
            );

            // Draw each icon
            ctx.draw_image(&icon, icon_dest_rect, InterpolationMode::Bilinear);
        }

        //Rendering back and forward buttons
        let left_half_rect = Rect::from_origin_size (
            Point::new(icon_top_position.x + ((icon_width + spacing) * 4 as f64), icon_top_position.y),
            Size::new(backforward, icon_height),
        );
    
        let right_half_rect = Rect::from_origin_size (
            Point::new(icon_top_position.x + ((icon_width + spacing) * 4 as f64) + (icon_width / 2.0), icon_top_position.y),
            Size::new(backforward, icon_height),
        );
        
        // render the first two images, the first on top of the other
        let back_icon = ctx.make_image(
                backforward as usize,
                icon_height as usize,
                &self.images.icons[4].to_rgba(),
                ImageFormat::RgbaSeparate,
            ).unwrap();
        ctx.draw_image(&back_icon, left_half_rect, InterpolationMode::Bilinear);

        
        let forward_icon = ctx.make_image(
            backforward as usize,
            icon_height as usize,
            &self.images.icons[5].to_rgba(),
            ImageFormat::RgbaSeparate,
            ).unwrap();
        ctx.draw_image(&forward_icon, right_half_rect, InterpolationMode::Bilinear);


        let (resize_width, resize_height) = &self.images.screenshot.dimensions();
        let center_position = Point::new(
            (size.width - *resize_width as f64) / 2.0,
            (size.height + 75.0 - *resize_height as f64)  / 2.0,
        );
        self.scaling_factors = (-center_position.x, -center_position.y);


        // Create an image with the dimensions of the loaded image
        let image = ctx
            .make_image(*resize_width as usize, *resize_height as usize, &self.images.screenshot.to_rgb8(), ImageFormat::Rgb)
            .unwrap();
    
        // Create a destination rectangle centered in the middle of the screen
        let dest_rect = Rect::from_origin_size(center_position, Size::new(*resize_width as f64, *resize_height as f64));
        self.dest_rect = Some(dest_rect);
        ctx.draw_image(&image, dest_rect, InterpolationMode::Bilinear);

        if let Some(temp) = &self.temp_edit {
            let upper = ctx
            .make_image(*resize_width as usize, *resize_height as usize, &temp.to_rgba(), ImageFormat::RgbaSeparate)
            .unwrap();
            ctx.draw_image(&upper, dest_rect, InterpolationMode::Bilinear);
        }
    }

    fn layout(&mut self, _ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, _data: &AppState, _env: &Env) -> druid::Size {
        // Return the size of the widget
        Size::new(bc.max().width, bc.max().height)
    }

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &AppState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {}

}


fn get_buttons_positions(widget_size: Size) -> (Point, Point, Point, Point, Point, Point) {
    let icon_width = 64.0;
    let icon_height = 64.0;

    // Calculate the total width needed for three icons and the spacing between them
    let total_width = 5.0 * icon_width;
    let spacing = (widget_size.width - total_width) / 6.0;

    // Calculate the top position for the icons
    let icon_top_position = Point::new(spacing, 25.0);

    // Calculate positions for each icon
    let icon0_position = Point::new(icon_top_position.x, icon_top_position.y);
    let icon1_position = Point::new(icon_top_position.x + (icon_width + spacing) * 1.0, icon_top_position.y);
    let icon2_position = Point::new(icon_top_position.x + (icon_width + spacing) * 2.0, icon_top_position.y);
    let icon3_position = Point::new(icon_top_position.x + (icon_width + spacing) * 3.0, icon_top_position.y);
    let back_pos = Point::new(icon_top_position.x + ((icon_width + spacing) * 4 as f64), icon_top_position.y);
    let forward_pos = Point::new(icon_top_position.x + ((icon_width + spacing) * 4 as f64) + (icon_width / 2.0), icon_top_position.y);

    (icon0_position, icon1_position, icon2_position, icon3_position, back_pos, forward_pos)
}



