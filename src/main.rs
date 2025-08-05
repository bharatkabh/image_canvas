use eframe::{egui, App};
use egui::{ColorImage, TextureHandle, TextureOptions, Vec2};
use image::{DynamicImage, GenericImageView, RgbaImage};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use rfd::FileDialog;
use std::path::PathBuf;



pub struct ImageLayer {
    pub prev_opacity: f32,
    pub prev_brightness: i32,
    pub prev_contrast: f32,
    pub image: DynamicImage,
	 pub original_rgba: RgbaImage,
    pub texture: Option<TextureHandle>,
    pub visible: bool,
    pub opacity: f32,
    pub dirty: bool,
    pub cached_rgba: Option<RgbaImage>,
    pub brightness: i32,
    pub contrast: f32,
}

pub struct ImageApp {
    layers: Vec<ImageLayer>,
    rotation_angle_deg: f32,
    zoom_factor: f32,
}

impl Default for ImageApp {
    fn default() -> Self {
        Self {
            layers: Vec::new(),
            rotation_angle_deg: 0.0,
            zoom_factor: 1.0,
                        }
    }
}


impl App for ImageApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📂 Load Layer").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        if let Ok(img) = image::open(&path) {
                            let padded = pad_to_square(&img);
                            self.layers.push(ImageLayer {
    prev_opacity: 1.0,
	 prev_brightness: 0,
	prev_contrast: 0.0,
	image: padded.clone(),
	 original_rgba: padded.to_rgba8(),
    texture: None,
    visible: true,
    opacity: 1.0,
    dirty: true,
    cached_rgba: Some(padded.to_rgba8()),
	brightness: 0,
    contrast: 0.0,
});
                            self.update_all_textures(ctx); // only updates dirty layers
                        }
                    }
                }

                if ui.button("💾 Save Merged Image").clicked() {
    if let Some(path) = FileDialog::new()
    .add_filter("PNG Image", &["png"])
    .set_file_name("untitled.png")
    .save_file() {
        if let Some((w, h)) = self
            .layers
            .iter()
            .find(|l| l.visible)
            .map(|l| l.image.dimensions())
        {
            use image::{Rgba, ImageBuffer};
            let mut canvas = ImageBuffer::from_pixel(w, h, Rgba([0, 0, 0, 0]));
            for layer in self.layers.iter().filter(|l| l.visible) {
                let mut rgba = layer.image.to_rgba8();
                if (layer.opacity - 1.0).abs() > f32::EPSILON {
                    for pixel in rgba.pixels_mut() {
                        pixel.0[3] = ((pixel.0[3] as f32) * layer.opacity).round().clamp(0.0, 255.0) as u8;
                    }
                }
                image::imageops::overlay(&mut canvas, &rgba, 0, 0);
            }
            std::thread::spawn(move || {
    let _ = DynamicImage::ImageRgba8(canvas).save(&path);
});
        }
    }
}
              

                if ui.button("🔁 Rotate Top").clicked() {
    if let Some(layer) = self.layers.iter_mut().rev().find(|l| l.visible) {
       let rotated = rotate_with_padding(&layer.image, self.rotation_angle_deg);
                        layer.image = rotated.clone();
                        layer.cached_rgba = Some(rotated.to_rgba8());
                        layer.dirty = true;
                        ctx.request_repaint();
                        self.rotation_angle_deg = 0.0;
                        self.update_all_textures(ctx);
    }
}

                

                ui.add(egui::Slider::new(&mut self.rotation_angle_deg, -180.0..=180.0).text("Rotation Angle"));
                ui.add(egui::Slider::new(&mut self.zoom_factor, 0.1..=5.0).text("Zoom"));
            });

            ui.separator();
            ui.collapsing("🎨 Layer Visibility", |ui| {
    let mut changed = false;
    for (i, layer) in self.layers.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            changed |= ui.checkbox(&mut layer.visible, format!("Layer {}", i + 1)).changed();
            let opacity_changed = ui.add(egui::Slider::new(&mut layer.opacity, 0.0..=1.0).text("Opacity")).changed();
            if opacity_changed { layer.dirty = true; }
            changed |= opacity_changed;
            let brightness_changed = ui.add(egui::Slider::new(&mut layer.brightness, -100..=100).text("Brightness")).changed();
            if brightness_changed { layer.dirty = true; }
            changed |= brightness_changed;
            let contrast_changed = ui.add(egui::Slider::new(&mut layer.contrast, -100.0..=100.0).text("Contrast")).changed();
            if contrast_changed { layer.dirty = true; }
            changed |= contrast_changed;
           if ui.button("Reset").clicked() {
    layer.opacity = 1.0;
    layer.brightness = 0;
    layer.contrast = 0.0;
    layer.prev_opacity = 1.0;
    layer.prev_brightness = 0;
    layer.prev_contrast = 0.0;
	layer.cached_rgba = Some(layer.original_rgba.clone());
    layer.dirty = true;
	  ctx.request_repaint();
}
		   /*  if ui.button("Reset").clicked() {
                layer.opacity = 1.0;
                layer.brightness = 0;
                layer.contrast = 0.0;
                layer.dirty = true;
            } */
           /* changed |= ui
                .add(egui::Slider::new(&mut layer.opacity, 0.0..=1.0).text("Opacity"))
                .changed(); */
        });
    }
    if changed {
        self.update_all_textures(ctx);
    }
});


            ui.separator();
let base_size = self
    .layers
    .iter()
    .find(|l| l.visible)
    .and_then(|l| l.texture.as_ref())
    .map(|tex| tex.size_vec2())
    .unwrap_or(egui::vec2(512.0, 512.0));
let desired_size = base_size * self.zoom_factor;
let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
let painter = ui.painter_at(rect);

for layer in self.layers.iter().filter(|l| l.visible) {
    if let Some(tex) = &layer.texture {
        let size = tex.size_vec2() * self.zoom_factor;
        let top_left = rect.min;
        let image_rect = egui::Rect::from_min_size(top_left, size);
        painter.image(
            tex.id(),
            image_rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), // UV
            egui::Color32::WHITE,
        );
    }
}
            
        });
    }
}

 impl ImageApp {
fn update_all_textures(&mut self, ctx: &egui::Context) {
    use std::thread;

    let ctx = ctx.clone();
    let layers_in = std::mem::take(&mut self.layers);

    // Spawn thread and move processed layers back
    let handle = thread::spawn(move || {
        let mut result_layers = Vec::with_capacity(layers_in.len());
        for (i, mut layer) in layers_in.into_iter().enumerate() {
            if !layer.dirty {
                result_layers.push(layer);
                continue;
            }
            if let Some(rgba) = &layer.cached_rgba {
                let mut adjusted = image::imageops::brighten(rgba, layer.brightness);
                adjusted = image::imageops::contrast(&adjusted, layer.contrast);
                let size = [adjusted.width() as usize, adjusted.height() as usize];
				let mut pixels = adjusted.into_raw();
                for j in (0..pixels.len()).step_by(4) {
                    pixels[j + 3] = ((pixels[j + 3] as f32) * layer.opacity).round().clamp(0.0, 255.0) as u8;
                }
				
//let mut pixels = adjusted.into_raw(); // consumes `adjusted` after we've gotten the size

               // let size = [adjusted.width() as usize, adjusted.height() as usize];
                let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
                let texture = ctx.load_texture(&format!("image_{}", i), color_image, TextureOptions::LINEAR);
                layer.texture = Some(texture);
                layer.dirty = false;
            }
            result_layers.push(layer);
        }
        result_layers
    });

    // Wait for thread and retrieve processed layers
    if let Ok(updated_layers) = handle.join() {
        self.layers = updated_layers;
    }
}

 }


/* impl ImageApp {
    fn update_all_textures(&mut self, ctx: &egui::Context) {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let ctx = ctx.clone();
    let layers = Arc::new(Mutex::new(std::mem::take(&mut self.layers)));
    let layers_clone = Arc::clone(&layers);

    thread::spawn(move || {
        let mut layers = layers_clone.lock().unwrap();
        for (i, layer) in layers.iter_mut().enumerate() {
            if !layer.dirty { continue; }
            if let Some(rgba) = &layer.cached_rgba {
                let mut adjusted = image::imageops::brighten(rgba, layer.brightness);
                adjusted = image::imageops::contrast(&adjusted, layer.contrast);
                let mut pixels = adjusted.into_raw();
                for i in (0..pixels.len()).step_by(4) {
                    pixels[i + 3] = ((pixels[i + 3] as f32) * layer.opacity).round().clamp(0.0, 255.0) as u8;
                }
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

                let texture = ctx.load_texture(&format!("image_{}", i), color_image, TextureOptions::LINEAR);
                layer.texture = Some(texture);
                layer.dirty = false;
            }
        }
    });

    self.layers = Arc::try_unwrap(layers).unwrap().into_inner().unwrap();
}
} */

fn pad_to_square(img: &DynamicImage) -> DynamicImage {
    use image::{Rgba, ImageBuffer, imageops::overlay};

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let size = w.max(h);

    let offset_x = ((size - w) / 2) as i64;
    let offset_y = ((size - h) / 2) as i64;

    let mut canvas = ImageBuffer::from_pixel(size, size, Rgba([0, 0, 0, 0]));
    overlay(&mut canvas, &rgba, offset_x, offset_y);

    DynamicImage::ImageRgba8(canvas)
}

fn rotate_with_padding(img: &DynamicImage, angle_deg: f32) -> DynamicImage {
    use image::{Rgba, ImageBuffer};
    use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let angle = angle_deg.to_radians();

    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;

    let corners = [
        (-cx, -cy),
        (cx, -cy),
        (-cx, cy),
        (cx, cy),
    ];

    let rotated = corners.iter().map(|(x, y)| {
        let new_x = x * angle.cos() - y * angle.sin();
        let new_y = x * angle.sin() + y * angle.cos();
        (new_x, new_y)
    });

    let (min_x, max_x, min_y, max_y) = rotated.clone().fold(
        (f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY, f32::NEG_INFINITY),
        |(min_x, max_x, min_y, max_y), (x, y)| {
            (
                min_x.min(x),
                max_x.max(x),
                min_y.min(y),
                max_y.max(y),
            )
        },
    );

    let new_w = (max_x - min_x).ceil() as u32;
    let new_h = (max_y - min_y).ceil() as u32;

    let offset_x = ((new_w as f32) / 2.0 - cx).round() as i64;
    let offset_y = ((new_h as f32) / 2.0 - cy).round() as i64;

    let mut canvas = ImageBuffer::from_pixel(new_w, new_h, Rgba([0, 0, 0, 0]));
    image::imageops::overlay(&mut canvas, &rgba, offset_x, offset_y);

    let rotated = rotate_about_center(
        &canvas,
        angle,
        Interpolation::Bilinear,
        Rgba([0, 0, 0, 0]),
    );

    DynamicImage::ImageRgba8(rotated)
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust Image Canvas",
        options,
        Box::new(|_cc| Box::new(ImageApp::default())),
    )
}
