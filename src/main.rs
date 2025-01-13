use std::time;

use nannou::prelude::*;
use nannou::noise::*;
use nannou::draw::*;
use nannou::rand::*;
use nannou::lyon::{geom::euclid::Point2D, path::Path};
use nannou_egui::{self, egui, Egui};

// Window size in Points, coordinates from (-WIN_d / 2, WIN_d / 2)
const WIN_WIDTH: u32 = 1000;
const WIN_HEIGHT: u32 = 1000;

fn main() {
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::wait())
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

    let draw: Draw = app.draw();

    draw.background().color(nannou::color::BLACK);

    // How many lines to draw
    const NUM_LINES: usize = 32;
    // How many drawn points in a line
    const NUM_PTS: usize = 256;
    // "Start" radius
    const START_RAD: f32 = 400.0;
    // Number of segments to split the start to
    const NUM_SEG: usize = 4;
    // Corresponding arclen
    const SEG_AL: f32 = 2.0 * PI / NUM_SEG as f32;
    // Proportion of segment to start points in
    const SEG_START_PRO: f32 = 0.75;
    // Corresponding arclen
    const SEG_START_AL: f32 = SEG_AL * SEG_START_PRO;
    // Global offset for segment start
    let SEG_OFF: f32 = random_range(0.0, 2.0 * PI);
    // "Stop" radius as percentage of START_RAD
    const STOP_RAD_PERC: f32 = 0.125;
    // "Wind-in" speed -> Line will 0..WIND_AL
    const WIND_AL: f32 = 4.0 * PI;

    // Draw some number of lines
    let mut lines: Vec<Path> = Vec::with_capacity(NUM_LINES);

    for ldx in 0..NUM_LINES {
        let mut b = Path::builder();

        // Get "segment" to start in
        let ss: usize = random_range(0, NUM_SEG);
        // Some dist into that segment
        let t0: f32 = random_range(0.0, SEG_START_AL) + ss as f32 * SEG_AL + SEG_OFF;

        // "Step" start point some number of times
        // r = sin(x) + sigmoid(x)
        // t = x

        let wav_off: f32 = random_range(0.0, 2.0 * PI);
        let wav_mag: f32 = random_range(0.05, 0.1);
        let wav_spd: f32 = random_range(48.0, 80.0);
        let line_start: f32 = random_range(0.0, 0.5);
        let line_end: f32 = random_range(0.5, 1.0);

        for pdx in 0..NUM_PTS {
            // We will need pdx as a percentage
            let pdx_perc = pdx as f32 / ( NUM_PTS - 1 ) as f32;
            // Clip percentage 0..1 -> line_start..line_end
            let pdx_perc = pdx_perc * (line_end - line_start) + line_start;

            // Theta is a function of dist along the line
            let t = t0 + WIND_AL * pdx_perc;

            // Sigmoid coord is rescaled 0..NUM_PTS -> -4..10
            let sig_coord = pdx_perc * 14.0 - 4.0;
            // Sigmoid is a logistic func of the sig_coord
            let sig = 1.0 / ( 1.0 + (-1.0 * sig_coord).exp() );
            // Flip around y=0.5
            let sig = (sig - 0.5) * -1.0 + 0.5;

            // Wave the line in and out, but real smol
            // TODO: Wave perpendicular to velocity
            let wav: f32 = (pdx_perc * wav_spd + wav_off).sin() * wav_mag;
            // Also scale the wave by the sigmoid, so it shrinks as we get closer to event horizon
            let wav = wav * sig;

            // Shift and squash sigmoid 0..1 -> STOP_RAD_PERC..1
            let sig = sig * ( 1.0 - STOP_RAD_PERC ) + STOP_RAD_PERC;


            let r = (sig + wav) * START_RAD;

            if pdx == 0 {
                b.begin(Point2D::new(r * t.cos(), r * t.sin()));
            } else {
                b.line_to(Point2D::new(r * t.cos(), r * t.sin()));
            }
        }

        b.end(false);

        let p = b.build();
        lines.push(p);
    }

    for line in lines {
        draw.path()
            .stroke()
            .color(nannou::color::WHITE)
            .events(line.iter())
            .finish();
    }

    draw.to_frame(app, &frame)
        .unwrap();
    model.egui.draw_to_frame(&frame)
        .unwrap();
}