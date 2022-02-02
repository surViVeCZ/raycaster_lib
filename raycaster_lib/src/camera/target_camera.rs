use nalgebra::{matrix, point, vector, Matrix4, Point3, Vector3};
use sdl2::event::Event;

use super::Camera;

pub struct TargetCamera {
    position: Point3<f32>,
    target: Point3<f32>,
    spherical: (f32, f32, f32),
    resolution: (usize, usize),
    mouse_down: bool,
}

impl TargetCamera {
    pub fn new(width: usize, height: usize) -> TargetCamera {
        let mut t = TargetCamera {
            position: point![300.0, 300.0, 300.0],
            target: point![34.0, 128.0, 128.0],
            spherical: (0.0, 0.0, 0.0),
            resolution: (width, height),
            mouse_down: false,
        };
        t.recalc_spherical();
        t
    }

    pub fn change_pos(&mut self, delta: Vector3<f32>) {
        self.position += delta;
        self.recalc_spherical();
    }

    pub fn set_pos(&mut self, pos: Point3<f32>) {
        self.position = pos;
        self.recalc_spherical();
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
        self.recalc_spherical();
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    // get spherical coordinates, relative to target
    // return r, theta, phi
    pub fn get_spherical(&self) -> (f32, f32, f32) {
        self.spherical
    }

    fn set_spherical(&mut self, r: f32, theta: f32, phi: f32) {
        self.spherical = (r, theta, phi);
        self.recalc_carthesian();
    }

    fn recalc_spherical(&mut self) {
        let dif = self.position - self.target;
        let r = dif.magnitude() as f32;
        let theta = (dif.z / r).acos();
        let phi = dif.y.atan2(dif.x);

        self.spherical = (r, theta, phi);
    }

    fn recalc_carthesian(&mut self) {
        let (r, theta, phi) = self.spherical;
        let sphere_offset = vector![
            r * theta.sin() * phi.cos(),
            r * theta.sin() * phi.sin(),
            r * theta.cos()
        ];

        self.position = self.target + sphere_offset;
    }
}

impl Camera for TargetCamera {
    fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    fn view_matrix(&self) -> Matrix4<f32> {
        // calculate camera coord system
        let camera_forward = (self.position - self.target).normalize();
        let up_vec = vector![0.0, 1.0, 0.0];
        let right = Vector3::cross(&up_vec, &camera_forward);
        let up = Vector3::cross(&camera_forward, &right);

        // cam to world matrix
        matrix![right.x, up.x, camera_forward.x, self.position.x;
                right.y, up.y, camera_forward.y, self.position.y;
                right.z, up.z, camera_forward.z, self.position.z;
                0.0, 0.0, 0.0, 1.0]
    }

    fn get_position(&self) -> Point3<f32> {
        self.position
    }

    fn get_user_input(&mut self, event: &sdl2::event::Event) {
        match event {
            Event::MouseMotion { xrel, yrel, .. } => {
                // When mouse button is down, drag camera around
                if !self.mouse_down {
                    return;
                }

                let drag_diff = (*xrel as f32, *yrel as f32);

                let (r, mut theta, mut phi) = self.spherical;

                println!("Current: > r {} theta {} phi {}", r, theta, phi);

                let drag_speed = 0.04;

                phi += drag_speed * drag_diff.1; // drag to left (negative drag) increases phi (rotation clockwise)

                theta -= drag_speed * drag_diff.0;

                println!("New T {}", theta);

                self.set_spherical(r, theta, phi);

                println!("New pos > {:?}", self.position);
            }
            Event::MouseButtonDown { .. } => self.mouse_down = true,
            Event::MouseButtonUp { .. } => self.mouse_down = false,
            Event::MouseWheel { y, .. } => {
                // y        ... vertical scroll
                // +1 unit  ... 1 step of wheel down (negative -> scroll up)

                let (mut r, theta, phi) = self.get_spherical();

                let zoom_speed = 10.0;
                r += zoom_speed * (-y as f32);

                self.set_spherical(r, theta, phi);
            }
            _ => {}
        }
    }
}
