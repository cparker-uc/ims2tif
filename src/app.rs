use egui::Context;
use eframe;
use std::path::PathBuf;
use image;

#[derive(Debug)]
pub struct Config {
    // Config will be called to read arguments from the command line
    pub file_name: PathBuf,
    pub output_dir: PathBuf,
    pub res_levels: usize,
    pub channels: usize,
}

impl Config {
    pub fn new() -> Self {
        Self {
            file_name: PathBuf::new(),
            output_dir: PathBuf::new(),
            res_levels: 0,
            channels: 0,
        }
    }
}

pub struct GuiApp { 
    pub conf: Config, 
}

impl GuiApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self { 
            conf: Config::new(),
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style())
                    .inner_margin(egui::Margin::same(16))
                    .inner_margin(egui::Margin::bottomf(32.0.into())))
            .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Imaris to TIFF Image Converter");
                        // Initialize the image loader so the icon will show up in the main window
                        egui_extras::install_image_loaders(ctx);
                        let logo = egui::Image::from_bytes("../assets/icon_big.png", &include_bytes!("../assets/icon_big.png")[..])
                            // .maintain_aspect_ratio(true)
                            .fit_to_original_size(0.1);
                        ui.add(logo);
                    });

        });
    }
}