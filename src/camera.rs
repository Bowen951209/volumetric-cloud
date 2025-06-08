use cgmath::{InnerSpace, Matrix3, Rad, SquareMatrix, Vector3};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub direction: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_to_rh(self.eye, self.direction, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[derive(Default)]
pub struct CameraController {
    move_speed: f32,
    rotation_speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_mouse_pressed: bool,
    delta_vertical_angle: f64,
    delta_horizontal_angle: f64,
    last_mouse_position: Option<PhysicalPosition<f64>>,
}

impl CameraController {
    pub fn new(move_speed: f32, rotation_speed: f32) -> Self {
        Self {
            move_speed,
            rotation_speed,
            ..Default::default()
        }
    }

    pub fn restore(&mut self) {
        let move_speed = self.move_speed;
        let rotation_speed = self.rotation_speed;

        *self = Self::default();
        self.move_speed = move_speed;
        self.rotation_speed = rotation_speed;
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    KeyCode::ShiftLeft => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::MouseInput { state, .. } => {
                self.is_mouse_pressed = state.is_pressed();
                true
            }
            WindowEvent::CursorMoved { position, .. } => match self.is_mouse_pressed {
                true => {
                    if let Some(last_mouse_position) = self.last_mouse_position {
                        let delta_x = position.x - last_mouse_position.x;
                        let delta_y = position.y - last_mouse_position.y;

                        self.delta_horizontal_angle = -delta_x;
                        self.delta_vertical_angle = -delta_y;
                    }

                    self.last_mouse_position = Some(*position);
                    true
                }
                false => {
                    self.last_mouse_position = None;
                    false
                }
            },
            WindowEvent::CursorLeft { .. } => {
                self.last_mouse_position = None;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        let camera_rotation_horizontal = Matrix3::from_axis_angle(
            camera.up,
            Rad(self.delta_horizontal_angle as f32 * self.rotation_speed),
        );
        camera.direction = camera_rotation_horizontal * camera.direction;

        let right = camera.direction.cross(camera.up).normalize();

        let dir_dot_y = camera.direction.dot(Vector3::unit_y());
        if dir_dot_y.abs() < 0.99
            || (dir_dot_y > 0.0 && self.delta_vertical_angle < 0.0)
            || (dir_dot_y < 0.0 && self.delta_vertical_angle > 0.0)
        {
            let camera_rotation_vertical = Matrix3::from_axis_angle(
                right,
                Rad(self.delta_vertical_angle as f32 * self.rotation_speed),
            );
            camera.direction = camera_rotation_vertical * camera.direction;
        }

        self.delta_horizontal_angle = 0.0;
        self.delta_vertical_angle = 0.0;

        let forward_xz = Vector3::new(camera.direction.x, 0.0, camera.direction.z);
        let forward_xz = forward_xz.normalize();

        if self.is_forward_pressed {
            camera.eye += forward_xz * self.move_speed;
        }

        if self.is_backward_pressed {
            camera.eye -= forward_xz * self.move_speed;
        }

        if self.is_right_pressed {
            camera.eye += right * self.move_speed;
        }

        if self.is_left_pressed {
            camera.eye -= right * self.move_speed;
        }

        if self.is_up_pressed {
            camera.eye += camera.up * self.move_speed;
        }

        if self.is_down_pressed {
            camera.eye -= camera.up * self.move_speed;
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj_inv: [[f32; 4]; 4],
    cam_pos: [f32; 3],
    _padding: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj_inv: cgmath::Matrix4::identity().into(),
            cam_pos: [0.0, 0.0, 0.0],
            ..Default::default()
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_proj_inv = camera
            .build_view_projection_matrix()
            .invert()
            .unwrap()
            .into();
        self.cam_pos = camera.eye.into();
    }
}
