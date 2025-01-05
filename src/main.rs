use std::time;

use nannou::prelude::*;
use nannou::noise::*;
use nannou_egui::{self, egui, Egui};

const WIN_WIDTH: u32 = 1000;
const WIN_HEIGHT: u32 = 1000;

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

struct Settings {
    nums: u32,

    nxs: f64,
    nxf: f64,
    nys: f64,
    nyf: f64,
    nss: f64,
}

struct Model {
    egui: Egui,

    noisex: Perlin,
    noisey: Perlin,
    noises: Perlin,
    noisec: Perlin,
    noiseoff: f64,

    settings: Settings,
}

fn model(app: &App) -> Model {
    let seed = time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let p = Perlin::new();
    let px = p.set_seed(seed);
    let py = p.set_seed(seed + 1);
    let ps = p.set_seed(seed + 2);
    let pc = p.set_seed(seed + 3);

    let window_id = app
        .new_window()
        .size(WIN_WIDTH, WIN_HEIGHT)
        .resizable(false)
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    Model {
        egui,
        noisex: px,
        noisey: py,
        noises: ps,
        noisec: pc,
        noiseoff: 0.0,
        settings: Settings{
            nums: 64,

            nxs: 10.0,
            nxf: 8.0,
            nys: 10.0,
            nyf: 8.0,
            nss: 1.0,
        },
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let settings = &mut model.settings;

    model.noiseoff = update.since_start.as_secs_f64();

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.label("Noise X Scale:");
        let nxss = egui::Slider::new(&mut settings.nxs, 0.0..=64.0);
        ui.add(nxss);

        ui.label("Noise X Factor:");
        let nxfs = egui::Slider::new(&mut settings.nxf, 1.0..=16.0);
        ui.add(nxfs);

        ui.label("Noise Y Scale:");
        let nyss = egui::Slider::new(&mut settings.nys, 0.0..=64.0);
        ui.add(nyss);

        ui.label("Noise Y Factor:");
        let nyfs = egui::Slider::new(&mut settings.nyf, 1.0..=16.0);
        ui.add(nyfs);

        ui.label("Noise Scale Scale:");
        let nsss = egui::Slider::new(&mut settings.nss, 0.1..=4.0);
        ui.add(nsss);
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let settings = &model.settings;

    let draw = app.draw();

    draw.background().color(nannou::color::BLACK);

    let gridsize_x = WIN_WIDTH as f64 / settings.nums as f64;
    let gridsize_y = WIN_HEIGHT as f64 / settings.nums as f64;

    for row in 0..(settings.nums) {
        let pos_y = gridsize_y * ( 0.5 + row as f64 ) - (WIN_HEIGHT as f64 / 2.0);

        let pos_y_norm = pos_y * 2.0 / WIN_HEIGHT as f64;

        for col in 0..(settings.nums) {
            let pos_x = gridsize_x * ( 0.5 + col as f64 ) - (WIN_WIDTH as f64 / 2.0);

            let pos_x_norm = pos_x * 2.0 / WIN_WIDTH as f64;

            let nxc = pos_x_norm * settings.nxf;
            let nyc = pos_y_norm * settings.nyf;

            let nx = model.noisex.get([nxc, nyc, model.noiseoff]) * settings.nxs;
            let ny = model.noisey.get([nxc, nyc, model.noiseoff]) * settings.nys;

            let ns = model.noises.get([nxc, nyc, model.noiseoff]);
            let ns = (ns + 1.0) / 2.0;
            let ns = gridsize_x.min(gridsize_y) * settings.nss * ns;

            let nc = model.noisec.get([nxc, nyc, model.noiseoff]);
            let nc = (nc + 1.0) / 2.0;


            draw.ellipse()
                .xy(Vec2::new(( pos_x + nx ) as f32, ( pos_y + ny ) as f32))
                .radius(ns as f32)
                .color(nannou::color::gray(nc as f32));
        }
    }

    /*
    let mut b = Path::builder();
    b.begin(Point2D::new(-225.0, 0.0));
    b.quadratic_bezier_to(Point2D::new(-75.0, 100.0), Point2D::new(0.0, 0.0));
    b.quadratic_bezier_to(Point2D::new(75.0, -100.0), Point2D::new(225.0, 0.0));
    b.end(false);
    let p = b.build();
    draw.path()
        .stroke()
        .color(nannou::color::WHITE)
        .events(p.iter());
    */

    draw.to_frame(app, &frame)
        .unwrap();
    model.egui.draw_to_frame(&frame)
        .unwrap();
}