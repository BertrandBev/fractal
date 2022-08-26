use crate::fractal::*;
use crate::image_utils::{IPoint, RGB};
use crate::time::Instant;
use druid::kurbo::Circle;
use druid::{Point, Size};
use num_cpus;
use std::sync::{Arc, Mutex};
use std::thread;

const STAGES: usize = 4;
const BATCH: usize = 100;

#[derive(PartialEq, Clone, Copy)]
struct Input {
    size: Size,
    focus: Circle,
    stage: usize,
    quit: bool,
}

impl Input {
    fn new() -> Self {
        Input {
            size: Size::ZERO,
            focus: Circle::new(Point::ZERO, 0.),
            stage: 0,
            quit: false,
        }
    }

    fn resize(size: Size, focus: Circle) -> Self {
        Input {
            size: size,
            focus: focus,
            stage: 0,
            quit: false,
        }
    }
}

struct ThreadData {
    id: usize,
    batch_idx: usize,
    complete: bool,
    input: Input,
    buffers: [Vec<RGB>; STAGES],
}

struct RendererThread {
    // State
    thread_count: usize,
    thread: Option<thread::JoinHandle<()>>,
    data: Arc<Mutex<ThreadData>>,
}

impl RendererThread {
    fn new(id: usize, thread_count: usize) -> Self {
        let buf: [Vec<RGB>; STAGES] = Default::default();
        let data = Arc::new(Mutex::new(ThreadData {
            id: id,
            batch_idx: 0,
            complete: false,
            input: Input::new(),
            buffers: buf,
        }));
        RendererThread {
            thread_count: thread_count,
            thread: None,
            data: data,
        }
    }

    fn stage_size(size: &Size, stage: usize) -> IPoint {
        let factor = 2_f64.powi((STAGES - stage - 1) as i32);
        let w = (size.width / factor).floor() as usize;
        let h = (size.height / factor).floor() as usize;
        IPoint::new(w, h)
    }

    fn current_size(data: &ThreadData) -> IPoint {
        Self::stage_size(&data.input.size, data.input.stage)
    }

    fn buffer_length(data: &ThreadData, thread_count: usize) -> usize {
        let size = Self::current_size(data);
        size.x * size.y / thread_count + BATCH
    }

    fn init_buffer(data: &mut ThreadData, thread_count: usize) {
        let size = Self::buffer_length(data, thread_count);
        data.buffers[data.input.stage].resize(size, RGB::TRANSPARENT);
        data.buffers[data.input.stage].fill(RGB::TRANSPARENT);
    }

    fn start(&mut self) {
        let thread_count = self.thread_count;
        if thread_count > 1 {
            let data = Arc::clone(&self.data);
            let thread = thread::spawn(move || loop {
                if Self::thread_loop(&data, thread_count) {
                    break;
                }
            });
            self.thread = Some(thread);
        }
    }

    fn thread_loop(data: &Arc<Mutex<ThreadData>>, thread_count: usize) -> bool {
        // Simulate sleep
        // thread::sleep(Duration::from_millis(50));
        // Retreive data in a scope
        let (size, batch_idx, id): (IPoint, usize, usize);
        let input: Input;
        let mut complete: bool;
        {
            let data = data.lock().unwrap();
            if data.input.quit {
                return true;
            }
            (batch_idx, id, complete) = (data.batch_idx, data.id, data.complete);
            input = data.input;
            size = Self::current_size(&data);
        }
        // Skip if complete
        if complete {
            return false;
        }
        // Process a batch of items
        let idx = (thread_count * batch_idx + id) * BATCH;
        complete = idx >= size.x * size.y;
        let mut buf = [RGB::TRANSPARENT; BATCH];
        if !complete {
            // Process buffer
            for k in 0..BATCH {
                let idx = idx + k;
                let (x, y) = (idx % size.x, idx / size.x);

                // TODO: tune
                let f = (0.001 + 2.0 * input.focus.radius).sqrt();
                let max_iter = (223.0 / f).floor() as usize;

                let size = Size::new(size.x as f64, size.y as f64);
                let world = px_to_world(&input.focus, &size, &IPoint { x: x, y: y });
                let res = mandelbrot(
                    Complex {
                        r: world.x,
                        i: world.y,
                    },
                    100.,
                    max_iter,
                );
                let rgb = color_scheme(&res);
                buf[k] = rgb;

                // Benchmark
                // thread::sleep(Duration::from_micros(4 * (4 - input.stage as u64).pow(2)));
                // let x = x - size.width as usize / 2;
                // let y = y - size.height as usize / 2;
                // buf[k] = RGB::BLACK;
                // if (x * x + y * y) < (size.width * size.height) as usize / 6 {
                //     buf[k] = RGB::rand();
                // }
            }
        }
        // Now append that batch
        {
            let mut data = data.lock().unwrap();
            data.complete = complete;
            if !complete && Self::current_size(&data) == size {
                let stage = data.input.stage;
                let buffer = &mut data.buffers[stage];
                let slice = &mut buffer[batch_idx * BATCH..(batch_idx + 1) * BATCH];
                slice.copy_from_slice(&buf);
                data.batch_idx += 1;
            }
        }
        false
    }

    fn stop(&mut self) {
        self.data.lock().unwrap().input.quit = true;
        self.thread.take().map(|thread| thread.join());
    }

    fn resize(&self, size: Size, focus: Circle) {
        self.data.lock().unwrap().input = Input::resize(size, focus);
        self.set_stage(0);
    }

    fn set_stage(&self, stage: usize) {
        let mut data = self.data.lock().unwrap();
        data.input.stage = stage;
        data.complete = false;
        data.batch_idx = 0;
        Self::init_buffer(&mut data, self.thread_count);
    }

    fn status(&self, stage: usize) -> (bool, f64) {
        let data = self.data.lock().unwrap();
        let mut total_pixels = 0;
        let mut done = 0;
        for s in 0..STAGES {
            let size = Self::stage_size(&data.input.size, s);
            let px = size.x * size.y;
            total_pixels += px;
            if s < data.input.stage {
                done += px;
            } else if s == data.input.stage {
                done += (data.batch_idx * BATCH * self.thread_count).min(px);
            }
        }
        let progress = (done as f64 / total_pixels as f64).min(1.);
        (data.input.stage == stage && data.complete, progress)
    }

    fn populate_image(&self, image: &mut [RGB]) {
        // Run a fixed number of loops if thread hasn't started
        if self.thread.is_none() {
            let start = Instant::now();
            loop {
                Self::thread_loop(&self.data, self.thread_count);
                let elapsed = start.elapsed().as_millis();
                if elapsed > 30 {
                    break;
                }
            }
        }

        // Populate image
        let mut data = self.data.lock().unwrap();
        let size = Self::current_size(&data);
        if image.len() != size.x * size.y {
            // Mismatched length
            return;
        }
        let mut batch_idx = 0;
        while batch_idx < data.batch_idx {
            let stage = data.input.stage;
            let idx = (self.thread_count * batch_idx + data.id) * BATCH;
            let buffer = &mut data.buffers[stage];
            let idx_end = (idx + BATCH).min(image.len());
            if idx_end < idx {
                break;
            }
            let len = idx_end - idx;
            let batch_idx_start = batch_idx * BATCH;
            let batch_idx_end = batch_idx_start + len;
            if batch_idx_end >= buffer.len() {
                break;
            };
            let slice = &mut buffer[batch_idx_start..batch_idx_end];
            (&mut image[idx..idx_end]).copy_from_slice(slice);
            batch_idx += 1;
        }
    }
}

pub struct RendererResult {
    pub image_size: IPoint,
    pub progress: f64,
}

pub struct Renderer {
    stage: usize,
    size: Size,
    focus: Circle,
    threads: Vec<RendererThread>,
}

impl Renderer {
    pub fn new() -> Self {
        let thread_count = num_cpus::get();
        println!("Starting {} threads", thread_count);

        // Create threads
        let mut threads: Vec<RendererThread> = Vec::new();
        for id in 0..thread_count {
            let mut thread = RendererThread::new(id, thread_count);
            thread.start();
            threads.push(thread);
        }
        Renderer {
            stage: 0,
            threads: threads,
            focus: Circle::new(Point::ZERO, 0.),
            size: Size::ZERO,
        }
    }

    pub fn stop(&mut self) {
        for thread in self.threads.iter_mut() {
            thread.stop();
        }
    }

    pub fn resize(&mut self, size: Size, focus: Circle) {
        if size != self.size || focus != self.focus {
            self.stage = 0;
            self.size = size;
            self.focus = focus;
            for thread in self.threads.iter_mut() {
                thread.resize(size, focus);
            }
        }
    }

    pub fn update(&mut self, image: &mut Vec<RGB>) -> RendererResult {
        // Resize image if needed
        let mut size = RendererThread::stage_size(&self.size, self.stage);
        image.resize(size.x * size.y, RGB::TRANSPARENT);
        // Retrieve image if needed
        for thread in self.threads.iter_mut() {
            thread.populate_image(image);
        }
        // Update progress
        let thread_count = self.threads.len();
        let mut complete_count = 0;
        let mut mean_progress = 0.;
        if self.stage <= STAGES - 1 {
            for thread in self.threads.iter_mut() {
                let (complete, progress) = thread.status(self.stage);
                mean_progress += progress / thread_count as f64;
                if complete {
                    complete_count += 1;
                }
            }
        } else {
            // Rendering done!
            mean_progress = 1.;
        }
        // Advance the stage if needed
        if self.stage < STAGES - 1 {
            if complete_count == thread_count {
                self.stage += 1;
                for thread in self.threads.iter_mut() {
                    thread.set_stage(self.stage);
                }
                // Resize image
                let old_size = size;
                let old_image = image.clone();
                size = RendererThread::stage_size(&self.size, self.stage);
                RGB::resize_image(&old_image, &old_size, image, &size);
            }
        }

        RendererResult {
            image_size: size,
            progress: mean_progress,
        }
    }
}
