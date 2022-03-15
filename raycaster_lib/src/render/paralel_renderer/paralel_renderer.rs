use std::{
    sync::{Arc, Mutex, RwLock},
    thread::JoinHandle,
};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::vector;

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::ViewportBox,
    render::RenderOptions,
    volumetric::BlockVolume,
};

use super::messages::{RenderTask, SubFrameResult, ToCompositorMsg, ToMasterMsg, ToRendererMsg};
use super::workers::{CompositorWorker, RenderWorker};

pub struct ParalelRendererFront {
    renderer: ParalelRenderer,
    communication_in: (Sender<()>, Receiver<()>),
    communication_out: (Sender<()>, Receiver<()>),
}

impl ParalelRendererFront {
    pub fn new(
        volume: BlockVolume,
        camera: Arc<RwLock<PerspectiveCamera>>,
        render_options: RenderOptions,
    ) -> Self {
        let communication_in = crossbeam_channel::unbounded(); // main -> renderer
        let communication_out = crossbeam_channel::unbounded(); // renderer -> main

        let communication = (communication_out.0.clone(), communication_in.1.clone());

        let renderer = ParalelRenderer::new(volume, camera, render_options, communication);
        Self {
            renderer,
            communication_in,
            communication_out,
        }
    }

    pub fn get_sender_receiver(&self) -> (Sender<()>, Receiver<()>, Arc<Mutex<Vec<u8>>>) {
        (
            self.communication_in.0.clone(),
            self.communication_out.1.clone(),
            self.renderer.buffer.clone(),
        )
    }

    pub fn start_rendering(self) -> JoinHandle<()> {
        self.renderer.start_rendering()
    }
}

pub struct ParalelRenderer {
    volume: BlockVolume,
    camera: Arc<RwLock<PerspectiveCamera>>, // In read mode during the render, write inbetween renders
    render_options: RenderOptions,
    buffer: Arc<Mutex<Vec<u8>>>,
    communication: (Sender<()>, Receiver<()>),
}

impl ParalelRenderer {
    pub fn new(
        volume: BlockVolume,
        camera: Arc<RwLock<PerspectiveCamera>>,
        render_options: RenderOptions,
        communication: (Sender<()>, Receiver<()>),
    ) -> Self {
        let elements = render_options.resolution.0 * render_options.resolution.1;
        let buffer = Arc::new(Mutex::new(vec![0; elements]));
        Self {
            volume,
            camera,
            render_options,
            buffer,
            communication,
        }
    }

    pub fn start_rendering(mut self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            // Scope assures threads will be joined before exiting the scope
            crossbeam::scope(|s| {
                let volume = &self.volume;

                // inlined function because borrow checker defeated me (scope cannot leave closure)
                let (render_handles, comp_handles, task_sender, result_receiver) = {
                    // Send to compositor, compositor recieves message
                    let ren_to_comp = [
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                    ];
                    let compositor_send: [Sender<ToCompositorMsg>; 4] = [
                        ren_to_comp[0].0.clone(),
                        ren_to_comp[1].0.clone(),
                        ren_to_comp[2].0.clone(),
                        ren_to_comp[3].0.clone(),
                    ];
                    // Send to renderer, renderer recieves message
                    let comp_to_ren = [
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                    ];
                    let renderer_send: [Sender<ToRendererMsg>; 4] = [
                        comp_to_ren[0].0.clone(),
                        comp_to_ren[1].0.clone(),
                        comp_to_ren[2].0.clone(),
                        comp_to_ren[3].0.clone(),
                    ];

                    let (task_sender, task_receiver) = crossbeam_channel::unbounded();

                    let mut renderers = Vec::with_capacity(4);
                    let mut compositors = Vec::with_capacity(4);

                    let resolution = vector![
                        self.render_options.resolution.0,
                        self.render_options.resolution.1
                    ];

                    for i in 0..4 {
                        // Create render thread
                        let receiver = comp_to_ren[i].1.clone(); // Receiver
                        let all_compositors = compositor_send.clone();
                        let task_receiver = task_receiver.clone();
                        let blocks_ref = &volume.data[..];
                        let camera_ref = self.camera.clone();
                        let handle = s.spawn(move |_| {
                            println!("Started renderer {i}");
                            // Force move into closure
                            let renderer_id = i;
                            let all_compositors = all_compositors; // Senders for all compositors
                            let task_receiver = task_receiver;
                            let message_receiver = receiver;
                            let blocks_ref = blocks_ref;
                            let camera_ref = camera_ref;

                            let render_worker = RenderWorker::new(
                                renderer_id,
                                camera_ref,
                                resolution,
                                all_compositors,
                                message_receiver,
                                task_receiver,
                                blocks_ref,
                            );

                            render_worker.run();
                        });

                        renderers.push(handle);
                    }

                    let compositor_areas = ParalelRenderer::generate_compositor_areas(4);

                    let (result_sender, result_receiver) = crossbeam_channel::unbounded();

                    for i in 0..4 {
                        // Create compositor thread

                        let receiver = ren_to_comp[i].1.clone(); // Receiver
                        let result_sender = result_sender.clone();
                        let all_renderers = renderer_send.clone();
                        let camera_ref = self.camera.clone();
                        let area = compositor_areas[i];
                        let blocks_ref = &volume.data[..];
                        let handle = s.spawn(move |_| {
                            println!("Started compositor {i}");
                            // Force move into closure
                            let compositor_id = i;
                            let all_renderers = all_renderers; // Senders for all compositors
                            let message_receiver = receiver;
                            let blocks_ref = blocks_ref;
                            let area = area;

                            let compositor = CompositorWorker::new(
                                compositor_id,
                                camera_ref,
                                area,
                                resolution,
                                all_renderers,
                                message_receiver,
                                result_sender,
                                blocks_ref,
                            );

                            compositor.run();
                        });

                        compositors.push(handle);
                    }

                    (renderers, compositors, task_sender, result_receiver)
                };

                let (width, height) = self.render_options.resolution;

                // Master loop
                loop {
                    // Gather input

                    // Lock buffer
                    let mut buffer = self.buffer.lock().unwrap();

                    // Render
                    self.render(
                        task_sender.clone(),
                        result_receiver.clone(),
                        &mut buffer[..],
                    );

                    // Send result
                }
            })
            .unwrap();
        })
    }

    fn render(
        &self,
        task_sender: Sender<RenderTask>,
        result_receiver: Receiver<ToMasterMsg>,
        buffer: &mut [u8],
    ) {
        let resolution = self.render_options.resolution;

        // Read lock until end of function
        let camera = self
            .camera
            .read()
            .expect("Cannot acquire read lock to camera");

        // get subvolume distances
        let mut block_order = Vec::with_capacity(self.volume.data.len());
        for (id, block) in self.volume.data.iter().enumerate() {
            let distance = camera.box_distance(&block.bound_box);
            block_order.push((id, distance));
        }
        block_order.sort_unstable_by(|i1, i2| i1.1.partial_cmp(&i2.1).unwrap());

        // Send rendering tasks
        //
        // Sent in order of camera distance (asc)
        // for Load balancing
        for (block_id, distance) in block_order {
            // Find out if block is empty, in which case dont send it
            let block = &self.volume.data[block_id];

            // Send task
            task_sender.send(RenderTask::new(block_id)).unwrap();
        }

        // Get subcanvases from compositors and save them to buffer
        for _ in 0..4 {
            let msg = result_receiver.recv().unwrap();
            match msg {
                ToMasterMsg::Subframe(res) => {
                    self.copy_subframe(buffer, res);
                }
            }
        }
    }

    fn generate_compositor_areas(arg: usize) -> Vec<ViewportBox> {
        todo!()
    }

    fn copy_subframe(&self, buffer: &mut [u8], res: SubFrameResult) {
        let SubFrameResult {
            data,
            offset,
            width,
        } = res;

        let data = &data[..];

        let mut ptr = offset;
        let mut ptr_data = 0;
        while ptr_data < buffer.len() {
            buffer[ptr..ptr + width].copy_from_slice(&data[ptr_data..ptr_data + width]);

            ptr += self.render_options.resolution.0;
            ptr_data += width;
        }
    }
}
