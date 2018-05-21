#![feature(vec_remove_item, proc_macro, wasm_custom_section, wasm_import_module)]
#![allow(unused)]

extern crate wasm_bindgen;

use std::f64::consts::PI;
use std::fmt;
use std::mem;
use std::ops::{Add, Mul};

use wasm_bindgen::describe::WasmDescribe;
use wasm_bindgen::prelude::*;

const CELL_SIZE: usize = 20;

mod piano;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Cell {
    row: usize,
    col: usize,
}

#[derive(PartialEq, Copy, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Point {
    fn origin() -> Point {
        Point { x: 0.0, y: 0.0 }
    }

    fn distance_to(self, o: Point) -> f64 {
        ((self.x - o.x).powi(2) + (self.y - o.y).powi(2)).sqrt()
    }

    fn cross(self, o: Point) -> f64 {
        self.x * o.y - o.x * self.y
    }

    fn rotate(self, angle: f64) -> Point {
        let Point { x, y } = self;
        Point {
            x: x * angle.cos() - y * angle.sin(),
            y: x * angle.sin() + y * angle.cos(),
        }
    }
}

impl Add<Vector2> for Point {
    type Output = Point;

    fn add(self, other: Vector2) -> Point {
        Point {
            x: (self.x + other.x),
            y: (self.y + other.y),
        }
    }
}

impl Mul<f64> for Vector2 {
    type Output = Vector2;

    fn mul(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Cell {
    fn new(row: usize, col: usize) -> Cell {
        Cell { row, col }
    }

    fn to_mid_point(self) -> Point {
        Point {
            x: (CELL_SIZE * self.col + CELL_SIZE / 2) as f64,
            y: (CELL_SIZE * self.row + CELL_SIZE / 2) as f64,
        }
    }

    fn to_corner(self) -> Point {
        Point {
            x: (CELL_SIZE * self.col) as f64 + CELL_SIZE as f64 / 2.0,
            y: (CELL_SIZE * self.row) as f64 + CELL_SIZE as f64,
        }
    }

    fn to_top_mid(self) -> Point {
        Point {
            x: (CELL_SIZE * self.col) as f64 + CELL_SIZE as f64 / 2.0,
            y: (CELL_SIZE * self.row) as f64,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Rectangle {
    base_a: Point,
    base_b: Point,
    height: f64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Segment {
    from: Point,
    to: Point,
}

impl Segment {
    fn new(a: Point, b: Point) -> Self {
        Segment { from: a, to: b }
    }

    fn midpoint(self) -> Point {
        Point {
            x: (self.from.x + self.to.x) / 2.0,
            y: (self.from.y + self.to.y) / 2.0,
        }
    }

    fn draw(self, ctx: &mut CanvasRenderingContext2D, color: &str) {
        Circle::new(self.from, 4.0).draw(ctx, "grey");
        Circle::new(self.to, 4.0).draw(ctx, "purple");
        ctx.begin_path();
        ctx.move_to(self.from.x as usize, self.from.y as usize);
        ctx.line_to(self.to.x as usize, self.to.y as usize);
        ctx.set_stroke_style(color);
        ctx.set_line_width(3.0);
        ctx.stroke();
    }

    fn to_rectangle(self, width: usize) -> Rectangle {
        // XXX: unimplemented
        Rectangle {
            base_a: self.from,
            base_b: self.to,
            height: 20.0,
        }
    }

    fn length(self) -> f64 {
        self.from.distance_to(self.to)
    }

    fn contains(self, needle: Point) -> bool {
        let t = Point {
            x: self.to.x - self.from.x,
            y: self.to.y - self.to.x,
        };
        let b = Point {
            x: needle.x - self.from.x,
            y: needle.y - self.from.y,
        };
        t.cross(b).abs() < ::std::f64::EPSILON
    }

    fn is_right_of_line(self, needle: Point) -> bool {
        let t = Point {
            x: self.to.x - self.from.x,
            y: self.to.y - self.to.x,
        };
        let b = Point {
            x: needle.x - self.from.x,
            y: needle.y - self.from.y,
        };
        t.cross(b) < 0.0
    }

    fn is_parallel(self, other: Self) -> bool {
        self.from.y - self.to.y == other.from.y - other.to.y
            && self.from.x - self.to.x == other.from.x - other.to.x
    }

    fn intersects(self, other: Self) -> bool {
        let i = !self.is_parallel(other)
            && (self.contains(other.from) || self.contains(other.to)
                || (self.is_right_of_line(other.from) ^ self.is_right_of_line(other.to)));
        log(&format!("{:?} intersects {:?} = {:?}", self, other, i));
        i
    }
}

/// Is p1 between p2 and p3?
fn between(p1: f64, p2: f64, p3: f64) -> bool {
    (p2 < p1 && p1 < p3) || (p3 < p1 && p1 < p2) || p1 == p2 || p1 == p3
}

#[derive(Copy, Clone)]
struct Vector2 {
    x: f64,
    y: f64,
}

impl fmt::Debug for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}, {}>", self.x, self.y)
    }
}

impl Vector2 {
    fn to_point(pt: Point) -> Vector2 {
        Vector2::from_segment(Segment::new(pt, Point { x: 0.0, y: 0.0 }))
    }

    fn i() -> Vector2 {
        Vector2 { x: 1.0, y: 0.0 }
    }

    fn j() -> Vector2 {
        Vector2 { x: 0.0, y: 1.0 }
    }

    fn angle_to(self, v: Vector2) -> f64 {
        (self.dot(v) / (self.magnitude() * v.magnitude())).acos()
    }

    fn anchor_at(self, p: Point) -> Segment {
        Segment {
            from: p,
            to: p + self,
        }
    }

    fn dot(self, o: Vector2) -> f64 {
        self.x * o.x + self.y * o.y
    }

    fn project_onto(self, b: Vector2) -> Vector2 {
        let a = self;
        //log(&format!(
        //    "{:?} onto {:?} = {:?}",
        //    a,
        //    b,
        //    b * (a.dot(b) as f64 / b.dot(b) as f64) as f64
        //));
        b * (a.dot(b) as f64 / b.dot(b) as f64) as f64
    }

    fn from_segment(s: Segment) -> Vector2 {
        Vector2 {
            x: (s.from.x as f64) - (s.to.x as f64),
            y: (s.from.y as f64) - (s.to.y as f64),
        }
    }

    fn orthogonal_cw(self) -> Vector2 {
        Vector2 {
            x: self.y,
            y: -self.x,
        }
    }

    fn orthogonal_ccw(self) -> Vector2 {
        Vector2 {
            x: -self.y,
            y: self.x,
        }
    }

    fn magnitude(self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    fn with_length(self, length: f64) -> Vector2 {
        let scale = length as f64 / self.magnitude();
        let x = self.x as f64;
        let y = self.y as f64;
        Vector2 {
            x: (x * scale),
            y: (y * scale),
        }
    }
}

impl Rectangle {
    fn contains(&self, needle: Point) -> bool {
        let [p1, p2, p3, p4] = {
            let v = Vector2::from_segment(Segment {
                from: self.base_a,
                to: self.base_b,
            });
            let v1 = v.orthogonal_ccw().with_length(self.height / 2.0);
            let v2 = v.orthogonal_cw().with_length(self.height / 2.0);
            let p1 = self.base_a + v1;
            let p2 = self.base_a + v2;
            let p3 = self.base_b + v2;
            let p4 = self.base_b + v1;
            [p1, p2, p3, p4]
        };

        let side = if p1.y < p2.y {
            Segment::new(p1, p2)
        } else {
            Segment::new(p2, p1)
        };
        let off_x = Vector2::i().angle_to(Vector2::from_segment(side));
        let p1 = p1.rotate(off_x);
        let p2 = p2.rotate(off_x);
        let p3 = p3.rotate(off_x);
        let p4 = p4.rotate(off_x);
        let np = needle.rotate(off_x);

        if between(np.x, p1.x, p3.x) && between(np.y, p1.y, p3.y) {
            return true;
        }

        false
    }

    fn center(&self) -> Point {
        let v = Vector2::from_segment(Segment {
            from: self.base_a,
            to: self.base_b,
        });
        let v1 = v.orthogonal_ccw().with_length(self.height / 2.0);
        let v2 = v.orthogonal_cw().with_length(self.height / 2.0);
        let p1 = self.base_a + v1;
        let p3 = self.base_b + v2;
        Segment::new(p1, p3).midpoint()
    }

    fn draw(&self, ctx: &mut CanvasRenderingContext2D, color: &str) {
        let v = Vector2::from_segment(Segment {
            from: self.base_a,
            to: self.base_b,
        });
        let v1 = v.orthogonal_ccw().with_length(self.height / 2.0);
        let v2 = v.orthogonal_cw().with_length(self.height / 2.0);
        let p1 = self.base_a + v1;
        let p2 = self.base_a + v2;
        let p3 = self.base_b + v2;
        let p4 = self.base_b + v1;

        ctx.set_line_width(4.0);
        ctx.begin_path();
        ctx.move_to(p1.x as usize, p1.y as usize);
        ctx.line_to(p2.x as usize, p2.y as usize);
        ctx.line_to(p3.x as usize, p3.y as usize);
        ctx.move_to(p4.x as usize, p4.y as usize);
        ctx.line_to(p1.x as usize, p1.y as usize);
        ctx.set_stroke_style(color);
        ctx.stroke();
    }
}

struct Circle {
    x: f64,
    y: f64,
    radius: f64,
}

impl Circle {
    fn new(Point { x, y }: Point, radius: f64) -> Circle {
        Circle { x, y, radius }
    }

    fn draw(&self, ctx: &mut CanvasRenderingContext2D, color: &str) {
        ctx.begin_path();
        ctx.move_to(self.x as usize, self.y as usize);
        ctx.arc(
            self.x as usize,
            self.y as usize,
            self.radius,
            0.0,
            PI * 2.0,
            false,
        );
        ctx.set_fill_style(color);
        ctx.fill();
    }
}

#[wasm_bindgen]
pub struct Universe {
    pub width: usize,
    pub height: usize,
    points: Vec<Cell>,
    ctx: CanvasRenderingContext2D,

    active: Option<Cell>,
    active_pt: Option<Point>,

    tubes: Vec<Tube>,

    audio: Audio,
    stop_selected: bool,
}

struct Tube {
    length: f64,
    from: Point,
    selected: bool,
    source: AudioSource,
    open: bool,
}

const SPEED: f64 = 343.0;
const PIXELS_PER_METER: f64 = 1300.0;

impl Tube {
    fn length(frequency: f64) -> f64 {
        SPEED / (4.0 * frequency) * (PIXELS_PER_METER)
    }

    fn segment(&self) -> Segment {
        Segment::new(
            self.from,
            Point {
                x: self.from.x,
                y: self.from.y - self.length,
            },
        )
    }

    fn contains(&self, p: Point) -> bool {
        self.segment().to_rectangle(0).contains(p)
    }

    fn new(audio: &Audio, from: Point, length: f64) -> Tube {
        // v = 343 m/s
        // L = length
        // Fundamental:
        // wavelength = 4 * length
        // v = wavelength * freq
        // freq = 343 / wavelength
        let mut source = audio.get_source(SPEED / (4.0 * length / PIXELS_PER_METER));
        Tube {
            length: length,
            from,
            selected: false,
            source: source,
            open: false,
        }
    }

    fn set_selected(&mut self, audio: &Audio, stop_selected: bool, s: bool) {
        if s {
            self.selected = true;
            if stop_selected {
                self.source.start();
            }
        } else {
            self.selected = false;
            if stop_selected {
                self.source.pause(audio);
            }
        }
    }

    fn adjust_frequency(&mut self) {
        if self.open {
            self.source
                .set_frequency(SPEED / (2.0 * self.length / PIXELS_PER_METER));
        } else {
            self.source
                .set_frequency(SPEED / (4.0 * self.length / PIXELS_PER_METER));
        }
    }

    fn draw(&self, ctx: &mut CanvasRenderingContext2D) {
        let v = Vector2::from_segment(self.segment());
        let Segment {
            from: base_a,
            to: base_b,
        } = self.segment();
        const WIDTH: f64 = 20.0;
        let v1 = v.orthogonal_ccw().with_length(WIDTH / 2.0);
        let v2 = v.orthogonal_cw().with_length(WIDTH / 2.0);
        let p1 = base_a + v1;
        let p2 = base_a + v2;
        let p3 = base_b + v2;
        let p4 = base_b + v1;

        ctx.set_line_width(4.0);
        ctx.begin_path();
        ctx.move_to(p1.x as usize, p1.y as usize);
        if self.open {
            ctx.move_to(p2.x as usize, p2.y as usize);
        } else {
            ctx.line_to(p2.x as usize, p2.y as usize);
        }
        ctx.line_to(p3.x as usize, p3.y as usize);
        // skip side because open on top always
        ctx.move_to(p4.x as usize, p4.y as usize);
        ctx.line_to(p1.x as usize, p1.y as usize);
        ctx.set_stroke_style(if self.selected {
            "#f0f"
        } else if self.source.playing {
            "#0fc"
        } else {
            "#0ff"
        });
        ctx.stroke();

        for &dir in [-1.0, 1.0].iter() {
            ctx.begin_path();
            ctx.move_to(base_a.x as usize, base_a.y as usize);
            let mut y = base_a.y;
            let divisor = if self.open { 2.0 } else { 4.0 };
            while y > base_b.y {
                y -= 0.5;
                let i = y - base_a.y;
                let x = base_b.x
                    + dir * (WIDTH / 2.0 - 4.0) * (i * 2.0 * PI / (divisor * v.magnitude())).sin();
                ctx.line_to_float(x, y);
            }
            ctx.set_line_width(2.0);
            ctx.set_stroke_style("#94b4dd");
            ctx.stroke();
        }
    }
}

struct Audio {
    ac: AudioContext,
    master: GainNode,
}

impl Audio {
    fn new(ac: AudioContext) -> Self {
        let master = ac.create_gain();
        master.connect(&ac.destination());
        Audio { ac, master }
    }

    fn get_source(&self, frequency: f64) -> AudioSource {
        let intermediate = self.ac.create_gain();
        intermediate.connect_to_gain(&self.master);
        let osc = self.ac.create_oscillator();
        osc.connect(&intermediate);
        osc.frequency().set_value(frequency);
        AudioSource {
            c: AS_COUNT.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst),
            source: osc,
            playing: false,
            frequency: frequency,
            intermediate,
            played: false,
        }
    }
}

use std::sync::atomic::AtomicUsize;
static AS_COUNT: AtomicUsize = AtomicUsize::new(0);

struct AudioSource {
    c: usize,
    frequency: f64,
    source: OscillatorNode,
    intermediate: GainNode,
    playing: bool,
    played: bool,
}

impl AudioSource {
    fn start(&mut self) {
        if !self.playing {
            self.playing = true;
            if !self.played {
                self.played = true;
                self.source.start();
            } else {
                self.intermediate.gain().set_value(1.0);
            }
        }
    }

    fn stop(mut self) {
        if self.playing {
            self.playing = false;
            self.source.stop();
        }
    }

    fn set_frequency(&mut self, f: f64) {
        self.frequency = f;
        self.source.frequency().set_value(f);
    }

    fn pause(&mut self, audio: &Audio) {
        if self.playing {
            self.playing = false;
            for i in 0..=10000 {
                self.intermediate
                    .gain()
                    .set_value(1.0 - 1.0 / 10000.0 * i as f64);
            }
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn new(ac: AudioContext, ctx: CanvasRenderingContext2D) -> Self {
        let audio = Audio::new(ac);
        let frequencies = [
            261.626, 293.665, 329.628, 349.228, 391.995, 440.000, 493.883, 523.251,
        ];

        Universe {
            height: 60,
            width: 60,
            points: Vec::new(),
            ctx,
            active: None,
            active_pt: None,
            tubes: frequencies
                .iter()
                .enumerate()
                .map(|(i, &f)| {
                    let x = (i as f64 + 1.0) * CELL_SIZE as f64 * 2.5;
                    let length = Tube::length(f);
                    Tube::new(&audio, Point { x, y: 750.0 }, length)
                })
                .collect::<Vec<_>>(),
            audio: audio,
            stop_selected: true,
        }
    }

    pub fn stop_selected(&mut self, checked: bool) {
        log(&format!("stop_Selected = {}", checked));
        self.stop_selected = checked;
    }

    pub fn keypress(&mut self, key: &str, shift_key: bool) -> bool {
        if let Some(idx) = self.tubes.iter().position(|t| t.selected) {
            match key {
                "d" => {
                    let tube = self.tubes.remove(idx);
                    tube.source.stop();
                }
                "o" => {
                    self.tubes[idx].open ^= true;
                    self.tubes[idx].adjust_frequency();
                }
                "p" => {
                    let tube = &mut self.tubes[idx];
                    if tube.source.playing {
                        tube.source.pause(&self.audio);
                    } else {
                        tube.source.start();
                    }
                }
                "ArrowUp" => {
                    self.tubes[idx].length += if shift_key { 20.0 } else { 1.0 };
                    self.tubes[idx].adjust_frequency();
                }
                "ArrowDown" => {
                    self.tubes[idx].length -= if shift_key { 20.0 } else { 1.0 };
                    if self.tubes[idx].length < 10.0 {
                        self.tubes[idx].length = 10.0;
                    }
                    self.tubes[idx].adjust_frequency();
                }
                "ArrowRight" => {
                    let l = self.tubes.len();
                    self.select(idx, false);
                    self.select((idx + 1) % l, true);
                }
                "ArrowLeft" => {
                    self.select(idx, false);
                    if idx == 0 {
                        let l = self.tubes.len();
                        self.select(l - 1, true);
                    } else {
                        self.select(idx - 1, true);
                    }
                }
                _ => {
                    return false;
                }
            }
            return true;
        }
        false
    }

    fn select(&mut self, idx: usize, v: bool) {
        self.tubes[idx].set_selected(&self.audio, self.stop_selected, v);
    }

    pub fn clicked(&mut self, row: usize, col: usize, x: usize, y: usize) -> bool {
        let clicked = Point {
            x: x as f64,
            y: y as f64,
        };
        let selected = self.tubes.iter().find(|t| t.contains(clicked)).is_some();
        if selected {
            for tube in &mut self.tubes {
                tube.set_selected(&self.audio, self.stop_selected, false);
            }
            let tube = self.tubes.iter_mut().find(|t| t.contains(clicked)).unwrap();
            tube.set_selected(&self.audio, self.stop_selected, true);
            return true;
        } else {
            let selected = Cell::new(row, col);
            if let Some(active) = self.active.take() {
                let from = self.active_pt.take().unwrap();
                let to = clicked;
                if active != selected {
                    let mut side = Segment { from: from, to: to };
                    let base = if from.y > to.y { from } else { to };
                    self.tubes.push(Tube::new(&self.audio, base, side.length()));
                }
            } else {
                self.active = Some(selected);
                self.active_pt = Some(clicked);
            }

            return true;
        }

        false
    }

    fn draw_rect(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        self.ctx.begin_path();
        self.ctx.move_to(x1, y1);
        self.ctx.line_to(x2, y1);
        self.ctx.line_to(x2, y2);
        self.ctx.line_to(x1, y2);
        self.ctx.line_to(x1, y1);
        self.ctx.stroke();
    }

    pub fn draw_grid(&mut self) {
        self.ctx.begin_path();
        self.ctx
        // todo: 1.0 / window.devicePixelRatio
            .set_line_width(0.5);
        self.ctx.set_stroke_style("#CCCCCC");

        // vertical lines
        for i in 0..=self.width {
            self.ctx.move_to(i * CELL_SIZE, 0);
            self.ctx.line_to(i * CELL_SIZE, CELL_SIZE * self.height);
        }

        // horizontal lines
        for i in 0..=self.height {
            self.ctx.move_to(0, i * CELL_SIZE);
            self.ctx.line_to(CELL_SIZE * self.width, i * CELL_SIZE);
        }

        self.ctx.stroke();

        if let Some(Cell { row: y, col: x }) = self.active {
            self.ctx.set_stroke_style("#f00");
            self.ctx.set_line_width(3.0);
            self.draw_rect(
                x * CELL_SIZE,
                y * CELL_SIZE,
                (x + 1) * CELL_SIZE,
                (y + 1) * CELL_SIZE,
            );
        }
    }

    pub fn draw_points(&mut self) {
        self.ctx.set_stroke_style("#00000");
        for &Cell { col: x, row: y } in &self.points {
            self.ctx
                .fill_rect(x * CELL_SIZE, y * CELL_SIZE, CELL_SIZE, CELL_SIZE);
        }
        document
            .body()
            .query_selector("#rust-out")
            .set_inner_html("");
        for tube in &self.tubes {
            tube.draw(&mut self.ctx);
            if tube.selected {
                document
                    .body()
                    .query_selector("#rust-out")
                    .set_inner_html(&format!(
                        "Selected {}tube:<br>Length: {:.3}m<br>Frequency: {:.2}{}",
                        if tube.open { "open " } else { "" },
                        tube.length / PIXELS_PER_METER,
                        tube.source.frequency,
                        if let Some((_, s)) = piano::NOTES
                            .iter()
                            .find(|(f, _)| *f == tube.source.frequency as usize)
                        {
                            format!("<br>Note: {}", s)
                        } else {
                            "".to_string()
                        }
                    ));
            }
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    pub type CanvasRenderingContext2D;
    #[wasm_bindgen(method, js_name = beginPath)]
    fn begin_path(this: &CanvasRenderingContext2D);
    #[wasm_bindgen(method, setter = lineWidth)]
    fn set_line_width(this: &CanvasRenderingContext2D, width: f64);
    #[wasm_bindgen(method, setter = strokeStyle)]
    fn set_stroke_style(this: &CanvasRenderingContext2D, style: &str);
    #[wasm_bindgen(method, setter = fillStyle)]
    fn set_fill_style(this: &CanvasRenderingContext2D, style: &str);
    #[wasm_bindgen(method, js_name = moveTo)]
    fn move_to(this: &CanvasRenderingContext2D, x: usize, y: usize);
    #[wasm_bindgen(method, js_name = lineTo)]
    fn line_to(this: &CanvasRenderingContext2D, x: usize, y: usize);
    #[wasm_bindgen(method, js_name = lineTo)]
    fn line_to_float(this: &CanvasRenderingContext2D, x: f64, y: f64);
    #[wasm_bindgen(method)]
    fn arc(
        this: &CanvasRenderingContext2D,
        x: usize,
        y: usize,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        anticlockwise: bool,
    );
    #[wasm_bindgen(method, js_name = fillRect)]
    fn fill_rect(this: &CanvasRenderingContext2D, x: usize, y: usize, width: usize, height: usize);
    #[wasm_bindgen(method)]
    fn stroke(this: &CanvasRenderingContext2D);
    #[wasm_bindgen(method)]
    fn fill(this: &CanvasRenderingContext2D);
    #[wasm_bindgen(method, js_name = fillText)]
    fn fill_text(this: &CanvasRenderingContext2D, text: &str, x: f64, y: f64);

    type HTMLDocument;
    static document: HTMLDocument;
    #[wasm_bindgen(method, js_name = createElement)]
    fn create_element(this: &HTMLDocument, tagName: &str) -> Element;
    #[wasm_bindgen(method, getter)]
    fn body(this: &HTMLDocument) -> Element;

    type Element;
    #[wasm_bindgen(method, setter = innerHTML)]
    fn set_inner_html(this: &Element, html: &str);
    #[wasm_bindgen(method, js_name = appendChild)]
    fn append_child(this: &Element, other: Element);
    #[wasm_bindgen(method, js_name = querySelector)]
    fn query_selector(this: &Element, selector: &str) -> Element;

    pub type AudioContext;
    #[wasm_bindgen(method, js_name = createGain)]
    fn create_gain(this: &AudioContext) -> GainNode;
    #[wasm_bindgen(method, js_name = createOscillator)]
    fn create_oscillator(this: &AudioContext) -> OscillatorNode;
    #[wasm_bindgen(method, getter)]
    fn destination(this: &AudioContext) -> AudioNode;

    type OscillatorNode;
    #[wasm_bindgen(method)]
    fn start(this: &OscillatorNode);
    #[wasm_bindgen(method)]
    fn stop(this: &OscillatorNode);
    #[wasm_bindgen(method)]
    fn connect(this: &OscillatorNode, node: &GainNode);
    #[wasm_bindgen(method)]
    fn disconnect(this: &OscillatorNode);
    #[wasm_bindgen(method, getter)]
    fn frequency(this: &OscillatorNode) -> AudioParam;

    type AudioParam;
    #[wasm_bindgen(method, setter)]
    fn set_value(this: &AudioParam, v: f64);
    #[wasm_bindgen(method, js_name = linearRampToValueAtTime)]
    fn linear_ramp_to(this: &AudioParam, v: f64, t: f64);

    type GainNode;
    #[wasm_bindgen(method, getter)]
    fn gain(this: &GainNode) -> AudioParam;
    type AudioNode;
    #[wasm_bindgen(method)]
    fn connect(this: &GainNode, node: &AudioNode);
    #[wasm_bindgen(method, js_name = connect)]
    fn connect_to_gain(this: &GainNode, node: &GainNode);

    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(cb: &Closure<FnMut()>, delay: u32) -> f64;
}
