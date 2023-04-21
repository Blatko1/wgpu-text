use nalgebra::{Matrix4, Point3, Vector3};

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    up: Vector3<f32>,
    pub aspect: f32,
    pub fov: f32,
    near: f32,
    far: f32,
    pub controller: CameraController,
    pub global_matrix: Matrix4<f32>,
}

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    pub fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        let controller = CameraController::new();
        Self {
            eye: Point3::new(0., 0., 1.),
            target: Point3::new(0., 0., -1.),
            up: Vector3::y(),
            aspect: config.width as f32 / config.height as f32,
            fov: 60.,
            near: 0.01,
            far: 100.0,
            controller,
            global_matrix: OPENGL_TO_WGPU_MATRIX,
        }
    }

    fn update_global_matrix(&mut self) {
        let target = Point3::new(
            self.eye.x + self.target.x,
            self.eye.y + self.target.y,
            self.eye.z + self.target.z,
        );
        let projection = Matrix4::new_perspective(
            self.aspect,
            self.fov.to_degrees(),
            self.near,
            self.far,
        );
        let view = Matrix4::look_at_rh(&self.eye, &target, &self.up);
        self.global_matrix = OPENGL_TO_WGPU_MATRIX * projection * view;
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.aspect = config.width as f32 / config.height as f32;
    }

    pub fn update(&mut self) {
        self.controller.update();
        self.fov += self.controller.fov_delta;
        self.controller.fov_delta = 0.;
        self.target = Point3::new(
            self.controller.yaw.to_radians().cos()
                * self.controller.pitch.to_radians().cos(),
            self.controller.pitch.to_radians().sin(),
            self.controller.yaw.to_radians().sin()
                * self.controller.pitch.to_radians().cos(),
        );
        let target = Vector3::new(self.target.x, 0.0, self.target.z).normalize();
        self.eye += target
            * self.controller.speed
            * (self.controller.forward - self.controller.backward);
        self.eye += target.cross(&self.up)
            * self.controller.speed
            * (self.controller.right - self.controller.left);
        self.eye += Vector3::new(0.0, 1.0, 0.0)
            * self.controller.speed
            * (self.controller.up - self.controller.down);
        self.update_global_matrix();
    }

    pub fn input(&mut self, event: &winit::event::DeviceEvent) {
        self.controller.process_input(event);
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f64,
    forward: f32,
    backward: f32,
    left: f32,
    right: f32,
    up: f32,
    down: f32,
    pub yaw: f32,
    pub pitch: f32,
    fov_delta: f32,
    time: std::time::Instant,
}

impl CameraController {
    pub fn new() -> Self {
        CameraController {
            speed: 0.4,
            sensitivity: 0.1,
            forward: 0.,
            backward: 0.,
            left: 0.,
            right: 0.,
            up: 0.,
            down: 0.,
            yaw: 0.,
            pitch: 0.0,
            fov_delta: 0.,
            time: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let time = self.time.elapsed().as_millis() as f64 * 0.01;
        self.yaw = 270.0 + 20.0 * time.to_radians().sin() as f32;
    }

    pub fn process_input(&mut self, event: &winit::event::DeviceEvent) {
        /*match event {
            DeviceEvent::MouseMotion { delta } => {
                self.yaw += (delta.0 * self.sensitivity) as f32;
                self.pitch -= (delta.1 * self.sensitivity) as f32;

                if self.pitch > 89.0 {
                    self.pitch = 89.0;
                } else if self.pitch < -89.0 {
                    self.pitch = -89.0;
                }

                if self.yaw > 360.0 {
                    self.yaw = 0.0;
                } else if self.yaw < 0.0 {
                    self.yaw = 360.0;
                }
            }
            DeviceEvent::Motion { .. } => {}
            DeviceEvent::Button { .. } => {}
            DeviceEvent::Key(KeyboardInput {
                state,
                virtual_keycode,
                ..
            }) => {
                let value: f32;
                if *state == winit::event::ElementState::Pressed {
                    value = 1.
                } else {
                    value = 0.;
                }
                match virtual_keycode.unwrap() {
                    VirtualKeyCode::Space => {
                        self.up = value;
                    }
                    VirtualKeyCode::LShift => {
                        self.down = value;
                    }
                    VirtualKeyCode::W => {
                        self.forward = value;
                    }
                    VirtualKeyCode::S => {
                        self.backward = value;
                    }
                    VirtualKeyCode::A => {
                        self.left = value;
                    }
                    VirtualKeyCode::D => {
                        self.right = value;
                    }
                    _ => (),
                }
            }
            _ => (),
        }*/
    }
}
