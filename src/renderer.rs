use crate::fractal::*;
use crate::image_utils::RGB;
use druid::{Rect, Size};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const STAGES: usize = 4;
const BATCH: usize = 100;

#[derive(PartialEq, Clone, Copy)]
struct Input {
    size: Size,
    region: Rect,
    stage: usize,
    quit: bool,
}

impl Input {
    fn new() -> Self {
        Input {
            size: Size::ZERO,
            region: Rect::ZERO,
            stage: 0,
            quit: false,
        }
    }

    fn resize(size: Size, region: Rect) -> Self {
        Input {
            size: size,
            region: region,
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
    thread: Option<thread::JoinHandle<()>>,
    data: Arc<Mutex<ThreadData>>,
}

impl RendererThread {
    fn new(id: usize) -> Self {
        let buf: [Vec<RGB>; STAGES] = Default::default();
        let data = Arc::new(Mutex::new(ThreadData {
            id: id,
            batch_idx: 0,
            complete: false,
            input: Input::new(),
            buffers: buf,
        }));
        RendererThread {
            thread: None,
            data: data,
        }
    }

    fn stage_size(size: &Size, stage: usize) -> (usize, usize) {
        let factor = 2_f64.powi((STAGES - stage - 1) as i32);
        let w = (size.width / factor).floor() as usize;
        let h = (size.height / factor).floor() as usize;
        (w, h)
    }

    fn current_size(data: &ThreadData) -> (usize, usize) {
        Self::stage_size(&data.input.size, data.input.stage)
    }

    fn init_buffer(data: &mut ThreadData) {
        let (w, h) = Self::current_size(data);
        data.buffers[data.input.stage].resize(w * h / STAGES + BATCH, RGB::TRANSPARENT);
        data.buffers[data.input.stage].fill(RGB::TRANSPARENT);
    }

    fn start(&mut self) {
        let data = Arc::clone(&self.data);
        let thread = thread::spawn(move || loop {
            // Simulate sleep
            thread::sleep(Duration::from_millis(100));
            // Retreive data in a scope
            let (w, h, batch_idx, id): (usize, usize, usize, usize);
            let mut complete: bool;
            {
                let data = data.lock().unwrap();
                if data.input.quit {
                    break;
                }
                (batch_idx, id, complete) = (data.batch_idx, data.id, data.complete);
                (w, h) = Self::current_size(&data);
            }
            // Skip if complete
            if complete {
                continue;
            }
            // Process a batch of items
            let idx = (STAGES * batch_idx + id) * BATCH;
            complete = idx >= w * h;
            let mut buf = [RGB::TRANSPARENT; BATCH];
            if !complete {
                // Process buffer
                for k in 0..BATCH {
                    buf[k] = RGB::rand()
                }
            }
            // Now append that batch
            {
                let mut data = data.lock().unwrap();
                data.complete = complete;
                if !complete {
                    // Copy batch
                    let stage = data.input.stage;
                    let buffer = &mut data.buffers[stage];
                    let slice = &mut buffer[batch_idx * BATCH..(batch_idx + 1) * BATCH];
                    slice.copy_from_slice(&buf);
                    data.batch_idx += 1;
                }
            }
        });
        self.thread = Some(thread);
    }

    fn stop(&mut self) {
        self.data.lock().unwrap().input.quit = true;
        self.thread.take().map(|thread| thread.join());
    }

    fn resize(&self, size: Size, region: Rect) {
        self.data.lock().unwrap().input = Input::resize(size, region);
        self.set_stage(0);
    }

    fn set_stage(&self, stage: usize) {
        let mut data = self.data.lock().unwrap();
        data.input.stage = stage;
        data.complete = false;
        data.batch_idx = 0;
        Self::init_buffer(&mut data);
    }

    fn stage_complete(&self, stage: usize) -> bool {
        let data = self.data.lock().unwrap();
        data.input.stage == stage && data.complete
    }

    fn populate_image(&self, image: &mut [RGB]) {
        let mut data = self.data.lock().unwrap();
        let (w, h) = Self::current_size(&data);
        // println!("w {} h {}", w, h);
        if image.len() != w * h {
            // Mismatched length
            return;
        }
        let mut batch_idx = 0;
        loop {
            let stage = data.input.stage;
            let idx = (STAGES * batch_idx + data.id) * BATCH;
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

pub struct Renderer {
    stage: usize,
    size: Size,
    region: Rect,
    threads: Vec<RendererThread>,
}

impl Renderer {
    pub fn new(thread_count: usize) -> Self {
        // Create threads
        let mut threads: Vec<RendererThread> = Vec::new();
        for id in 0..thread_count {
            let mut thread = RendererThread::new(id);
            thread.start();
            threads.push(thread);
        }
        Renderer {
            stage: 0,
            threads: threads,
            region: Rect::ZERO,
            size: Size::ZERO,
        }
    }

    pub fn stop(&mut self) {
        for thread in self.threads.iter_mut() {
            thread.stop();
        }
    }

    pub fn resize(&mut self, size: Size, region: Rect) {
        if size != self.size || region != self.region {
            self.stage = 0;
            self.size = size;
            self.region = region;
            for thread in self.threads.iter_mut() {
                thread.resize(size, region);
            }
        }
    }

    pub fn get_image(&mut self, image: &mut Vec<RGB>) -> (usize, usize) {
        // Resize image if needed
        let (w, h) = RendererThread::stage_size(&self.size, self.stage);
        image.resize(w * h, RGB::TRANSPARENT);
        // Advance the stage if needed
        if self.stage < STAGES - 1 {
            let mut complete_count = 0;
            for thread in self.threads.iter_mut() {
                if thread.stage_complete(self.stage) {
                    complete_count += 1;
                }
            }
            if complete_count == self.threads.len() {
                self.stage += 1;
                for thread in self.threads.iter_mut() {
                    thread.set_stage(self.stage);
                }
            }
        }
        // Retrieve images
        for thread in self.threads.iter_mut() {
            thread.populate_image(image);
        }
        (w, h)
    }
}
