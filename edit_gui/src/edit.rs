use druid::{Data, Widget, Event, EventCtx, PaintCtx, Size, Env, Rect, Point, FontWeight};
use druid::piet::{Text, TextLayoutBuilder, RenderContext, ImageFormat, InterpolationMode, Color};
use image::{GenericImageView};
use image::{DynamicImage, Rgba};

use crate::ImageData;
use crate::utils::*;
use crate::drawing_tools::{Stroke};

pub struct Edit {
    images: ImageData,
    dest_rect: Option<Rect>,
    initial_point: Option<Point>,
    movement_points: Vec<Point>,//Vec<BezPath>,
    color: (Rgba<u8>, u32, u8),
    scaling_factors: (f64, f64),
    drawing: bool,
    adding_shapes: bool,
    highlighting: bool,
    disable_event: bool,
    pencil_selected: bool,
    resizing: u8,
    list_of_edits: (Vec<DynamicImage>, usize), //list of images with operations to undo
    temp_edit: Option<DynamicImage>,
    choosen_shape: u8,
    bottom_pos: Point,
    selection: Option<Rect>,
}

impl Edit {
    pub fn new(images: ImageData) -> Self {
        let screenshot_clone = images.screenshot.clone();
        Edit {
            images: images,
            dest_rect: None,
            initial_point: None,
            movement_points: Vec::new(),
            color: (Rgba([0, 0, 0, 255]), 10, 0),
            scaling_factors: (0.0, 0.0),
            drawing: false,
            adding_shapes: false,
            highlighting: false,
            disable_event: false,
            pencil_selected: false,
            resizing: 1,
            list_of_edits: (vec![screenshot_clone], 1),
            temp_edit: None,
            choosen_shape: 0,
            bottom_pos: Point::new(0.0, 0.0),
            selection: None,
        }
    }

    fn get_buttons_positions(&self, widget_size: Size) -> (Point, Point, Point, Point, Point, Point, Point, Point) {
        let icon_width = 64.0;
        let _icon_height = 64.0;
    
        // Calculate the total width needed for three icons and the spacing between them
        let total_width = 4.0 * icon_width;
        let spacing = (widget_size.width - total_width) / 5.0;
        let icon_top_position = Point::new(spacing, 15.0);
    
        // Calculate positions for each icon AT THE TOP
        let icon0_position = Point::new(icon_top_position.x, icon_top_position.y);
        let icon1_position = Point::new(icon_top_position.x + (icon_width + spacing) * 1.0, icon_top_position.y);
        let icon2_position = Point::new(icon_top_position.x + (icon_width + spacing) * 2.0, icon_top_position.y);
        let icon3_position = Point::new(icon_top_position.x + (icon_width + spacing) * 3.0, icon_top_position.y);
        
        //calculate positions for each icon at the bottom
        let below_image_position = self.bottom_pos;
        let bottom_total_width = 3.0 * icon_width;
        let bottom_spacing = (widget_size.width as f64 - bottom_total_width) / 7.0;
    
        let back_pos = Point::new(below_image_position.x + (icon_width + bottom_spacing) as f64, below_image_position.y);
        let forward_pos = Point::new(below_image_position.x + 32.0 + (icon_width + bottom_spacing) as f64, below_image_position.y);
        let check_pos = Point::new(below_image_position.x + (icon_width + bottom_spacing) * 2 as f64, below_image_position.y);
        let save_pos =  Point::new(below_image_position.x + (icon_width + bottom_spacing) * 3 as f64, below_image_position.y);
    
        (icon0_position, icon1_position, icon2_position, icon3_position, back_pos, forward_pos, check_pos, save_pos)
    }

    fn get_clicked_button(&self, mouse_pos: Point, _data: &AppState, ctx: &EventCtx) -> Option<u8> {
        let size = ctx.size();
        let icon_size = Size::new(64.0, 64.0);
        let arrows_size = Size::new(32.0, 64.0);
        let (first, second, third, fourth, back, forward, check, save) = self.get_buttons_positions(size);
    
        if mouse_pos.is_inside_rect(first, icon_size) {
            Some(5) //Either resize or back to main functionalities
        }
        else if mouse_pos.is_inside_rect(second, icon_size) {
            Some(0) //Either text or circle
        } else if mouse_pos.is_inside_rect(third, icon_size) {
            Some(1) //Either highlighter or rectangle
        } else if mouse_pos.is_inside_rect(fourth, icon_size) {
            Some(2) //Either shapes or triangle
        } else if mouse_pos.is_inside_rect(back, arrows_size) {
            Some(3) //back
        } else if mouse_pos.is_inside_rect(forward, arrows_size) {
            Some(4) //forward
        } else if mouse_pos.is_inside_rect(save, icon_size){
            Some(9) //save
        } else if mouse_pos.is_inside_rect(check, icon_size) && self.resizing == 2 {
            Some(6) //save resize
        } else {
            None
        }
    }
}

#[derive(Clone, Data)]
pub struct AppState;

impl AppState {}

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

impl Widget<AppState> for Edit {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::MouseDown(mouse_event) => {
                let mouse_pos = mouse_event.pos;
                
                if let Some(button_clicked) = self.get_clicked_button(mouse_pos, data, ctx) {
                    match button_clicked {
                        0 => {
                            //writing or circle or orange
                            if self.adding_shapes {
                                self.choosen_shape = 0; //0 -> circle
                            } else if self.highlighting { 
                                self.drawing = true;
                                //set orange highlighter
                                self.color = (Rgba([255, 165, 0, 128]), 18, 1);
                            } else if self.pencil_selected  {
                                //writing, set small
                                self.color = (Rgba([0, 0, 0, 255]), 3, 1);
                            } else {
                                //writing normally
                                self.drawing = true;
                                self.pencil_selected = true;
                                self.color = (Rgba([0, 0, 0, 255]), 3, 1);
                            }
                            self.resizing = 0;
                            self.selection = None;
                            ctx.request_paint();
                        },
                        1 => {
                            //highlighter or rectangle or yellow highlighter or small size pencil
                            if self.adding_shapes {
                                self.choosen_shape = 1; //1 -> rectangle
                            } else if self.highlighting {
                                self.drawing = true;
                                //set yellow highlighter
                                self.color = (Rgba([255, 255, 0, 128]), 18, 2);
                            } else if self.pencil_selected  {
                                //medium pencil
                                self.color = (Rgba([0, 0, 0, 255]), 6, 2);
                            }  else {
                                //initialize highlighting process, default value = orange
                                self.drawing = true;
                                self.highlighting = true;
                                self.color = (Rgba([255, 165, 0, 128]), 18, 1);
                            }
                            self.resizing = 0;
                            self.selection = None;
                            ctx.request_paint();
                        },
                        2 => {
                            //adding shapes or triangle or green highlighter
                            if self.adding_shapes {
                                self.choosen_shape = 2; //2-> triangle
                            } else if self.highlighting {
                                self.drawing = true;
                                self.color = (Rgba([0, 255, 0, 128]), 18, 3);
                            } else if self.pencil_selected {
                                //large pencil
                                self.color = (Rgba([0, 0, 0, 255]), 9, 3);
                            } else {
                                self.adding_shapes = true;
                                self.drawing = false;
                            }
                            self.resizing = 0;
                            self.selection = None;
                            ctx.request_paint();
                        },
                        3 => {
                            //Undo
                            let (edits, index) = &self.list_of_edits;
                            if *index > 1 {
                                let undo_index = *index - 1;
                                self.images.screenshot = edits[undo_index-1].clone();
                                self.list_of_edits = (edits.clone(), undo_index);
                                ctx.request_paint();
                            }
                        },
                        4 => {
                            //Redo
                            let (edits, index) = &self.list_of_edits;
                            if *index < 5 && (edits.len() != *index) {
                                let redo_index = *index + 1;
                                self.images.screenshot = edits[redo_index-1].clone();
                                self.list_of_edits = (edits.clone(), redo_index);
                                ctx.request_paint();
                            }
                        },
                        5 => {
                            //resize or back
                            if self.adding_shapes || self.highlighting || self.pencil_selected {
                                self.adding_shapes = false;
                                self.highlighting = false;
                                self.pencil_selected = false;
                                self.resizing = 1;
                                ctx.request_paint();
                            } else {
                                //resize functionality
                                self.resizing = 1;
                                self.drawing = false;
                                //dont want to catch mouse up now:
                                self.disable_event = true;
                                ctx.request_paint();
                            }
                        }, 
                        6 => {
                            //resizing SAVING functionality
                            if let Some(selection) = self.selection {
                                let point = apply_scaling_to_point(self.scaling_factors, Point::new(selection.x0, selection.y0));
                                let width = selection.width().round() as u32;
                                let height = selection.height().round() as u32;
                                let mut screenshot = self.images.screenshot.clone();

                                let sub_image = screenshot.crop(point.x as u32, point.y as u32, width, height);

                                let resized_image = resize_image(sub_image.clone(), (1200, 500));
                                self.images.screenshot = resized_image.clone();

                                //update edit list for undo/redo
                                let (edits, index) = &self.list_of_edits;
                                let mut edits_clone = edits.clone(); 

                                if *index == 5 {
                                    edits_clone.remove(0);
                                    edits_clone.push(resized_image.clone());
                                    self.list_of_edits = (edits_clone, *index);
                                } else {
                                    let truncate_index = *index;
                                    edits_clone.truncate(truncate_index);
                                    edits_clone.push(resized_image.clone());
                                    let new_index = edits_clone.len();
                                    self.list_of_edits = (edits_clone, new_index);
                                }
                            }
                            self.resizing = 1;
                            self.selection = None;
                            ctx.request_paint();                        
                        },
                        9 => {
                            //save
                            let _ = save_edited_image(self.images.screenshot.clone(), self.images.directory.as_str());
                            self.resizing = 0;
                            
                        }
                        _ => {},
                        }
                } 
                
                if let Some(dest_rect) = self.dest_rect{
                    if mouse_pos.is_inside_direct_rect(dest_rect) {
                        self.initial_point = Some(mouse_pos);

                        if self.adding_shapes {
                            // do nothing actually
                        } else if self.resizing == 0 {
                            self.drawing = true;
                            self.movement_points.push(mouse_pos);
                        } else if self.resizing == 2 {
                            //check if complete resizing area, in that case a click means restart resizing
                            self.selection = None;
                            self.resizing = 1;
                        }
                        ctx.set_active(true);
                    }    
                }
            },

            Event::MouseUp(_mouse_event) => {
                // Stop capturing mouse movement when the mouse is released
                if self.drawing || self.adding_shapes {
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

                if self.resizing == 1 {
                    if self.disable_event {
                        self.disable_event = false;
                    } else {
                        if let Some(_selection) = self.selection {
                            self.resizing = 2;
                            self.initial_point = None;
                            ctx.request_paint();
                        }
                    }
                }
              
            },

            Event::MouseMove(mouse_event) => {
                if ctx.is_active() && self.adding_shapes {
                    //handle shapes
                    if let Some(initial_point) = self.initial_point {
                        let new_point = apply_scaling_to_point(self.scaling_factors, mouse_event.pos);
                        let initial_scaled = apply_scaling_to_point(self.scaling_factors, initial_point);
                        let strokes = Stroke::new_empty();
                        let img_size = self.images.screenshot.dimensions();

                        //check which shape
                        let temp_edit = if self.choosen_shape == 0 {
                            let radius = calculate_radius(initial_scaled, new_point);
                            let new_image = strokes.draw_circle(initial_scaled, radius, img_size);
                            image::DynamicImage::ImageRgba8(new_image)
                        } else if self.choosen_shape == 1 {
                            let new_image = strokes.draw_enlarging_rectangle(initial_scaled, new_point, img_size);
                            image::DynamicImage::ImageRgba8(new_image)
                        } else {
                            let new_image = strokes.draw_enlarging_triangle(initial_scaled, new_point, img_size);
                            image::DynamicImage::ImageRgba8(new_image)
                        };
                        self.temp_edit = Some(temp_edit);
                        ctx.request_paint();
                    }
                } else if ctx.is_active() && self.drawing  {
                    //handle drawing
                    if let Some(_dest_rect) = self.dest_rect {
                        let mouse_pos = mouse_event.pos;
                        self.movement_points.push(mouse_pos);
                    }
                    if self.movement_points.len() > 2 {
                            let converted_points = apply_scaling(self.scaling_factors, self.movement_points.clone());
                            let (color, marker, _) = self.color;
                            let strokes = Stroke::new(converted_points, color, marker);
                            let (width, height) = &self.images.screenshot.dimensions();
                            self.temp_edit = Some(image::DynamicImage::ImageRgba8(strokes.draw((*width, *height))));
                            ctx.request_paint();
                    }
                } else if self.resizing == 1 {
                    if let Some(initial_point) = self.initial_point {
                        if let Some(dest_rect) = self.dest_rect {
                            let mut selection = Rect::from_points(initial_point, mouse_event.pos);
                    
                            // Ensure that the selection does not exceed the boundaries of dest_rect
                            if selection.x1 > dest_rect.x1 {
                                selection.x1 = dest_rect.x1;
                            }
                            if selection.y1 > dest_rect.y1 {
                                selection.y1 = dest_rect.y1;
                            }
                            if selection.x0 < dest_rect.x0 {
                                selection.x0 = dest_rect.x0;
                            }
                            if selection.y0 < dest_rect.y0 {
                                selection.y0 = dest_rect.y0;
                            }
                    
                            self.selection = Some(selection);
                            ctx.request_paint();
                        }
                     
                    }
                }
            }
            _ => (),
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &AppState, _env: &Env) {
        let size = ctx.size();

        let icon_width = 64.0;
        let icon_height = 64.0;

        let backforward = 32.0;

        let total_width = 4.0 * icon_width;
        let spacing = (size.width - total_width) / 5.0;

        // Calculate the top position for the icons
        let icon_top_position = Point::new(spacing, 15.0);

        /* 
        General icons
        */

        if !self.adding_shapes && !self.highlighting && !self.pencil_selected && self.resizing != 0 { 
            for i in 0..4 {
                let (text, offset_x, offset_y, bold) = match i {
                    0 => ("• Resize".to_string(), 11.0, 10.0, true),  
                    1 => ("Pencil".to_string(), 7.0, 10.0, false), 
                    2 => ("Highlighter".to_string(), 1.0, 10.0, false), 
                    3 => ("Shapes".to_string(), 15.0, 10.0, false), 
                    _ => unreachable!(),
                };

                let times = ctx.text().font_family("Times New Roman").unwrap();
                let text_layout = ctx.text().new_text_layout(text).font(times.clone(), 14.0);
                let text_layout_builder = if bold { text_layout.default_attribute(FontWeight::BOLD) } else { text_layout };
                let final_build = text_layout_builder.build().unwrap();

                // Define the position for the text
                let text_position = Point::new(
                    icon_top_position.x + offset_x + (icon_width + spacing) * i  as f64,
                    icon_top_position.y + icon_height + offset_y, 
                );

                // Draw text
                ctx.draw_text(&final_build, text_position);
        
                let icon = ctx.make_image(icon_width as usize, icon_height as usize, &self.images.icons[i].to_rgba8(), ImageFormat::RgbaSeparate)
                            .unwrap();

                let icon_dest_rect = Rect::from_origin_size(
                    Point::new(icon_top_position.x + (icon_width + spacing) * i as f64, icon_top_position.y),
                    Size::new(icon_width, icon_height),
                );

                ctx.draw_image(&icon, icon_dest_rect, InterpolationMode::Bilinear);
            }
        } else if self.pencil_selected {
            for i in 0..4 {
                let (text, offset_x, offset_y, bold) = match i {
                    1 => {
                        if self.color.2 == 1 {
                            ("• Small".to_string(), 11.0, 10.0, true)
                        } else {
                            ("Small".to_string(), 15.0, 10.0, false)
                        }
                    },
                    2 => {
                        if self.color.2 == 2 {
                            ("• Medium".to_string(), 3.0, 10.0, true)
                        } else {
                            ("Medium".to_string(), 7.0, 10.0, false)
                        }
                    }, 
                    3 => {
                        if self.color.2 == 3 {
                            ("• Large".to_string(), 11.0, 10.0, true)
                        } else {
                            ("Large".to_string(), 15.0, 10.0, false)
                        }
                    }, 
                    0 => ("".to_string(), 0.0, 0.0, false), 
                    _ => unreachable!(),
                };

                let times = ctx.text().font_family("Times New Roman").unwrap();
                let text_layout = ctx.text().new_text_layout(text).font(times.clone(), 14.0);
                let text_layout_builder = if bold { text_layout.default_attribute(FontWeight::BOLD) } else { text_layout };
                let final_build = text_layout_builder.build().unwrap();

                let text_position = Point::new(
                    icon_top_position.x + offset_x + (icon_width + spacing) * i  as f64,
                    icon_top_position.y + icon_height + offset_y, 
                );
                ctx.draw_text(&final_build, text_position);
                
                let reposition_icon = if i == 0 { 6 } else { 14 };
                let icon = ctx.make_image(icon_width as usize, icon_height as usize, &self.images.icons[i+reposition_icon].to_rgba8(), ImageFormat::RgbaSeparate)
                            .unwrap();

                let icon_dest_rect = Rect::from_origin_size(
                    Point::new(icon_top_position.x + (icon_width + spacing) * i as f64, icon_top_position.y),
                    Size::new(icon_width, icon_height),
                );

                ctx.draw_image(&icon, icon_dest_rect, InterpolationMode::Bilinear);
            }
        } else if self.adding_shapes {
            for i in 0..4 {
                let (text, offset_x, offset_y, bold) = match i {
                    1 => {
                        if self.choosen_shape == 0 {
                            ("• Circle".to_string(), 12.0, 10.0, true)
                        } else {
                            ("Circle".to_string(), 15.0, 10.0, false)
                        }
                    },  
                    2 => {
                        if self.choosen_shape == 1 {
                            ("• Rectangle".to_string(), -1.0, 10.0, true)
                        } else {
                            ("Rectangle".to_string(), 4.0, 10.0, false)
                        }
                    }, 
                    3 => {
                        if self.choosen_shape == 2 {
                            ("• Triangle".to_string(), 4.0, 10.0, true)
                        } else {
                            ("Triangle".to_string(), 7.0, 10.0, false)
                        }
                    }, 
                    0 => ("".to_string(), 0.0, 0.0, false), 
                    _ => unreachable!(),
                };

                let times = ctx.text().font_family("Times New Roman").unwrap();
                let text_layout = ctx.text().new_text_layout(text).font(times.clone(), 14.0);
                let text_layout_builder = if bold { text_layout.default_attribute(FontWeight::BOLD) } else { text_layout };
                let final_build = text_layout_builder.build().unwrap();

                let text_position = Point::new(
                    icon_top_position.x + offset_x + (icon_width + spacing) * i  as f64,
                    icon_top_position.y + icon_height + offset_y, 
                );
                ctx.draw_text(&final_build, text_position);
                
                let icon = ctx.make_image(icon_width as usize, icon_height as usize, &self.images.icons[i+6].to_rgba8(), ImageFormat::RgbaSeparate)
                            .unwrap();

                let icon_dest_rect = Rect::from_origin_size(
                    Point::new(icon_top_position.x + (icon_width + spacing) * i as f64, icon_top_position.y),
                    Size::new(icon_width, icon_height),
                );

                ctx.draw_image(&icon, icon_dest_rect, InterpolationMode::Bilinear);
            }
        } else if self.highlighting {
            for i in 0..4 {
                let (text, offset_x, offset_y, bold) = match i {
                    1 => {
                        if self.color.2 == 1 {
                            ("• Orange".to_string(), 8.0, 10.0, true)
                        } else {
                            ("Orange".to_string(), 12.0, 10.0, false)
                        }
                    },  
                    2 => {
                        if self.color.2 == 2 {
                            ("• Yellow".to_string(), 8.0, 10.0, true)
                        } else {
                            ("Yellow".to_string(), 12.0, 10.0, false)
                        }
                    }, 
                    3 => {
                        if self.color.2 == 3 {
                            ("• Green".to_string(), 10.0, 10.0, true)
                        } else {
                            ("Green".to_string(), 14.0, 10.0, false)
                        }
                    }, 
                    0 => ("".to_string(), 0.0, 0.0, false), 
                    _ => unreachable!(),
                };

                let times = ctx.text().font_family("Times New Roman").unwrap();
                let text_layout = ctx.text().new_text_layout(text).font(times.clone(), 14.0);
                let text_layout_builder = if bold { text_layout.default_attribute(FontWeight::BOLD) } else { text_layout };
                let final_build = text_layout_builder.build().unwrap();

                let text_position = Point::new(
                    icon_top_position.x + offset_x + (icon_width + spacing) * i  as f64,
                    icon_top_position.y + icon_height + offset_y, 
                );
                ctx.draw_text(&final_build, text_position);

                let reposition_icon = if i == 0 { 6 } else { 9 };
                let icon = ctx.make_image(icon_width as usize, icon_height as usize, &self.images.icons[i+reposition_icon].to_rgba8(), ImageFormat::RgbaSeparate)
                            .unwrap();

                let icon_dest_rect = Rect::from_origin_size(
                    Point::new(icon_top_position.x + (icon_width + spacing) * i as f64, icon_top_position.y),
                    Size::new(icon_width, icon_height),
                );

                ctx.draw_image(&icon, icon_dest_rect, InterpolationMode::Bilinear);
            }
        }

    
        /*
        Rendering the image
        */

        let (resize_width, resize_height) = &self.images.screenshot.dimensions();
        let center_position = Point::new(
            (size.width - *resize_width as f64) / 2.0,
            (size.height + 20.0 - *resize_height as f64)  / 2.0,
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
            .make_image(*resize_width as usize, *resize_height as usize, &temp.to_rgba8(), ImageFormat::RgbaSeparate)
            .unwrap();
            ctx.draw_image(&upper, dest_rect, InterpolationMode::Bilinear);
        }

        /*
        Save, UNDO-Redo icons and EVENTUALLY Check for resizing
        */

        let bottom_total_width = 3.0 * icon_width;
        let bottom_spacing = (size.width as f64 - bottom_total_width) / 7.0;
        let below_image_y = 650.0;
        let below_image_position = Point::new(bottom_spacing, below_image_y);
        self.bottom_pos = below_image_position;


        let left_half_rect = Rect::from_origin_size (
            Point::new(below_image_position.x + (icon_width + bottom_spacing) as f64, below_image_position.y),
            Size::new(backforward, icon_height),
        );
        
        // Position the right half rect to the right of the left half rect
        let right_half_rect = Rect::from_origin_size (
            Point::new(below_image_position.x + backforward + (icon_width + bottom_spacing) as f64, below_image_position.y),
            Size::new(backforward, icon_height),
        );

        let check_rect = Rect::from_origin_size(
            Point::new(below_image_position.x + (icon_width + bottom_spacing) * 2 as f64, below_image_position.y),
            Size::new(icon_width, icon_height),
        );

        let save_rect = Rect::from_origin_size(
            Point::new(below_image_position.x + (icon_width + bottom_spacing) * 3  as f64, below_image_position.y),
            Size::new(icon_width, icon_height),
        );
        
        // render the first two images, the first on top of the other
        let back_icon = ctx.make_image(
                backforward as usize,
                icon_height as usize,
                &self.images.icons[4].to_rgba8(),
                ImageFormat::RgbaSeparate,
            ).unwrap();
        ctx.draw_image(&back_icon, left_half_rect, InterpolationMode::Bilinear);

        let forward_icon = ctx.make_image(
            backforward as usize,
            icon_height as usize,
            &self.images.icons[5].to_rgba8(),
            ImageFormat::RgbaSeparate,
            ).unwrap();
        ctx.draw_image(&forward_icon, right_half_rect, InterpolationMode::Bilinear);

        let save_icon = ctx.make_image(
            icon_width as usize,
            icon_height as usize,
            &self.images.icons[13].to_rgba8(),
            ImageFormat::RgbaSeparate,
            ).unwrap();
        ctx.draw_image(&save_icon, save_rect, InterpolationMode::Bilinear);

         
        for i in 0..3 {
            let (text, offset_x, offset_y, offset_width, mut render) = match i {
                0 => ("Undo/Redo".to_string(), 0.0, 10.0, 1, true),  
                1 => ("Confirm resize".to_string(), -5.0, 10.0, 2, false), 
                2 => ("Save".to_string(), 17.0, 10.0, 3, true), 
                _ => unreachable!(),
            };

            if i == 1 && self.resizing == 2 {
                render = true;
            }

            if render {
                let times = ctx.text().font_family("Times New Roman").unwrap();
                let text_layout = ctx.text().new_text_layout(text).font(times.clone(), 14.0);
                let final_build = text_layout.build().unwrap();

                let text_position = Point::new(
                    below_image_position.x + offset_x + (icon_width + bottom_spacing) * offset_width as f64,
                    below_image_position.y + icon_height + offset_y, 
                );

                ctx.draw_text(&final_build, text_position);
            }
        }
      

        if self.resizing != 0 {
            if let Some(selection) = self.selection {
                let selection_color = Color::rgba(0.0, 0.0, 0.0, 0.4);
                ctx.fill(selection, &selection_color);
            }
        } 

        if self.resizing == 2 {
            let check_icon = ctx.make_image(
                icon_width as usize,
                icon_height as usize,
                &self.images.icons[14].to_rgba8(),
                ImageFormat::RgbaSeparate,
                ).unwrap();
            ctx.draw_image(&check_icon, check_rect, InterpolationMode::Bilinear);
        }

    }

    fn layout(&mut self, _ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, _data: &AppState, _env: &Env) -> druid::Size {
        // Return the size of the widget
        Size::new(bc.max().width, bc.max().height)
    }

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &AppState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {}
}