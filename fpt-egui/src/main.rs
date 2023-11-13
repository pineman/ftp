#![feature(lazy_cell)]

use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use egui::{Color32, Pos2, TextureOptions};
use log::info;

// use sha2::Digest;

const GB_FRAME_IN_SECONDS: f64 = 0.016666666667;

pub struct TemplateApp {
    egui_frame_count: u64,
    gb_frame_count: u64,
    last_time: f64,
    accum_time: f64,
    image: Arc<egui::ColorImage>,
    texture: Option<egui::TextureHandle>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            egui_frame_count: 0,
            gb_frame_count: 0,
            last_time: 0.0,
            accum_time: 0.0,
            image: Arc::new(egui::ColorImage::new(
                [160, 144],
                egui::Color32::TRANSPARENT,
            )),
            texture: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        Default::default()
    }
}

#[cfg(target_arch = "wasm32")]
fn now() -> f64 {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("performance"))
        .expect("failed to get performance from global object")
        .unchecked_into::<web_sys::Performance>()
        .now()
}

#[cfg(not(target_arch = "wasm32"))]
static APP_START: LazyLock<Instant> = LazyLock::new(Instant::now);

#[cfg(not(target_arch = "wasm32"))]
fn now() -> f64 {
    APP_START.elapsed().as_secs_f64() * 1000.0
}

// fn calc_sha256(input: &str) -> String {
//     let mut hasher = sha2::Sha256::new();
//     hasher.update(input);
//     let result = hasher.finalize();
//     format!("{:x}", result)
// }

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                ui.add_space(16.0);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.egui_frame_count += 1;
            ui.heading("fpt");
            ui.add(egui::Label::new(self.egui_frame_count.to_string()));
            ui.separator();

            let a = now();
            let time = ui.input(|i| i.time);
            let delta_time = ui.input(|i| i.unstable_dt) as f64;
            self.accum_time += delta_time;
            let before = self.gb_frame_count;

            while self.accum_time >= GB_FRAME_IN_SECONDS {
                // if self.accum_time >= GB_FRAME_IN_SECONDS {
                self.gb_frame_count += 1;
                // ... RENDER GAME BOY SCREEN ...
                // uncomment to make Tito's fans slightly noisier
                // for _ in 0..1000 {
                //     calc_sha256("hello world");
                // }
                // gb_frame = gb.get_frame();
                self.accum_time -= GB_FRAME_IN_SECONDS;

                if let Some(image) = Arc::get_mut(&mut self.image) {
                    // image.pixels.fill(Color32::TRANSPARENT);

                    // It all starts with this...
                    static mut CHAOS_GAME: Pos2 = Pos2::new(80., 143.9);
                    const STEPS: u64 = 5;
                    for i in 0..STEPS {
                        let t = (self.gb_frame_count * STEPS + i) as f64 / 60.
                            * 0.33
                            * 2.
                            * std::f64::consts::PI;
                        let r = (200. + (t * 1.01 + 0.).sin() * 40.) as u8;
                        let g = (180. + (t * 0.08 + 1.).sin() * 70.) as u8;
                        let b = (40.0 + (t * 0.57 + 2.).sin() * 20.) as u8;
                        let (x, y) = unsafe {
                            CHAOS_GAME = CHAOS_GAME.lerp(
                                match ((r as u32) + (g as u32) + (b as u32)) % 3 {
                                    0 => Pos2::new(0., 0.),
                                    1 => Pos2::new(0., 143.9),
                                    _ => Pos2::new(159.9, 143.9),
                                },
                                0.5,
                            );
                            (CHAOS_GAME.x.floor() as usize, CHAOS_GAME.y.floor() as usize)
                        };
                        image[(x, y)] = Color32::from_rgb(r, g, b);
                        let (x, y) = (159 - x, 143 - y);
                        image[(x, y)] = Color32::from_rgb(b, g, r);
                    }
                }
            }

            // self.texture = None; // batota
            let texture: &mut egui::TextureHandle = self.texture.get_or_insert_with(|| {
                // Load the texture only once.
                ui.ctx()
                    .load_texture("my-image", self.image.clone(), TextureOptions::NEAREST)
            });
            texture.set(self.image.clone(), TextureOptions::NEAREST); // TODO repeated work in 1st repaint
            ui.image((texture.id(), 3. * texture.size_vec2()));
            // ui.load_texture(gb_frame);

            self.last_time = time;

            let mut _ccc = false;
            if self.gb_frame_count - before > 1 {
                info!("more than one gb_frame");
                _ccc = true;
            }
            ui.separator();
            egui::Grid::new("my_grid").striped(true).show(ui, |ui| {
                macro_rules! stat {
                    ($label:literal : $fmt:literal, $value:expr) => {
                        ui.colored_label(Color32::LIGHT_GRAY, $label);
                        ui.monospace(format!($fmt, $value));
                        ui.code(stringify!($value));
                        ui.end_row();
                    };
                }
                stat!("time"        : "{:.8}", time);
                stat!("dt"          : "{:.8}", delta_time);
                stat!("accum. time" : "{:.8}", self.accum_time);
                stat!("last time"   : "{:.8}", self.last_time);
                stat!("Ideal count" : "{:.3}", time / GB_FRAME_IN_SECONDS);
                stat!("Frame count" : "{}"   , self.gb_frame_count);
                stat!("UI updates"  : "{}"   , self.egui_frame_count);
            });

            let b = now();
            info!("a {:.8}", a);
            info!("b {:.8}", b);
            let time_taken = (b - a) / 1000.0;
            info!("time_taken {:.8}", time_taken);

            // let time_taken = a.elapsed().as_secs_f64();
            // if ccc {
            //     info!("time_taken1 {:.8}", time_taken);
            // }
            // let time_taken = (time_taken * 1000.0).floor() / 1000.0;
            // if ccc {
            //     info!("time_taken2 {:.8}", time_taken);
            // }

            let sleep_time = GB_FRAME_IN_SECONDS - time_taken;
            info!("sleep_time {:.8}", sleep_time);
            if sleep_time < 0.0 {
                ctx.request_repaint();
            } else {
                ctx.request_repaint_after(Duration::from_secs_f64(sleep_time));
            }
        });
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        initial_window_size: Some([500.0, 700.0].into()),
        min_window_size: Some([500.0, 700.0].into()),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
