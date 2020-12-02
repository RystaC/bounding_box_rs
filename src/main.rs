use std::f64;

extern crate glfw;

use glfw::{Action, Context, Key};

mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use ::gl::types::*;

extern {
    fn gluPerspective(fovy: GLdouble, aspect: GLdouble, zNear: GLdouble, zFar: GLdouble);
    fn gluLookAt(eyeX: GLdouble, eyeY: GLdouble, eyeZ: GLdouble, centerX: GLdouble, centerY: GLdouble, centerZ: GLdouble, upX: GLdouble, upY: GLdouble, upZ: GLdouble);
}

type Triangle = [[f32; 3]; 3];
type Color = Triangle;
type AABB = [[f32; 3]; 2];

fn main() {
    let width = 512;
    let height = 512;

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(width, height, "bounding box", glfw::WindowMode::Windowed).unwrap();

    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s));

    unsafe {
        gl::MatrixMode(gl::PROJECTION);
        gl::LoadIdentity();
        gluPerspective(45.0, (width / height) as GLdouble, 0.1, 20.0);

        gl::MatrixMode(gl::MODELVIEW);
        gl::LoadIdentity();
        gluLookAt(3.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    }

    let mut angle = 0.0;

    let triangle1 = [[1.0, 1.0, 1.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let triangle2 = [[-1.0, 0.0, 0.5], [-0.5, -1.0, 0.2], [0.0, -0.2, 0.0]];
    let triangles = vec![triangle1, triangle2];
    let color = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let aabb = create_aabb(triangles.as_slice());
    let aabb1 = create_aabb(&[triangles[0]]);
    let aabb2 = create_aabb(&[triangles[1]]);

    while !window.should_close() {
        glfw.poll_events();

        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::MatrixMode(gl::MODELVIEW);
            gl::LoadIdentity();
            gluLookAt(5.0 * f64::cos(angle * f64::consts::PI), 0.0, 5.0 * f64::sin(angle * f64::consts::PI), 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);

            draw_triangle(triangle1, color);
            draw_triangle(triangle2, color);

            draw_aabb(aabb);
            draw_aabb(aabb1);
            draw_aabb(aabb2);
        }

        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut angle);
        }

        window.swap_buffers();
    }
}

fn draw_triangle(triangle: Triangle, color: Color) {
    unsafe {
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);

        gl::Begin(gl::TRIANGLES);

        gl::Color3fv(color[0].as_ptr());
        gl::Vertex3fv(triangle[0].as_ptr());
        gl::Color3fv(color[1].as_ptr());
        gl::Vertex3fv(triangle[1].as_ptr());
        gl::Color3fv(color[2].as_ptr());
        gl::Vertex3fv(triangle[2].as_ptr());

        gl::End();
    }
}

fn create_aabb(triangles: &[Triangle]) -> AABB {
    let mut x = vec![];
    let mut y = vec![];
    let mut z = vec![];

    for triangle in triangles {
        x.append(&mut vec![triangle[0][0], triangle[1][0], triangle[2][0]]);
        y.append(&mut vec![triangle[0][1], triangle[1][1], triangle[2][1]]);
        z.append(&mut vec![triangle[0][2], triangle[1][2], triangle[2][2]]);
    }

    [[x.iter().fold(f32::MAX, |m, v| v.min(m)), y.iter().fold(f32::MAX, |m, v| v.min(m)), z.iter().fold(f32::MAX, |m, v| v.min(m))],
     [x.iter().fold(f32::MIN, |m, v| v.max(m)), y.iter().fold(f32::MIN, |m, v| v.max(m)), z.iter().fold(f32::MIN, |m, v| v.max(m))]]
}

fn draw_aabb(aabb: AABB) {
    let pos = [[aabb[0][0], aabb[0][1], aabb[0][2]], [aabb[1][0], aabb[0][1], aabb[0][2]], [aabb[1][0], aabb[1][1], aabb[0][2]], [aabb[0][0], aabb[1][1], aabb[0][2]],
        [aabb[0][0], aabb[0][1], aabb[1][2]], [aabb[1][0], aabb[0][1], aabb[1][2]], [aabb[1][0], aabb[1][1], aabb[1][2]], [aabb[0][0], aabb[1][1], aabb[1][2]]];

    unsafe {
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        gl::Begin(gl::QUADS);

        gl::Color3f(1.0, 0.0, 0.0);
        gl::Vertex3fv(pos[0].as_ptr());
        gl::Vertex3fv(pos[1].as_ptr());
        gl::Vertex3fv(pos[2].as_ptr());
        gl::Vertex3fv(pos[3].as_ptr());

        gl::End();

        gl::Begin(gl::QUADS);

        gl::Color3f(1.0, 0.0, 0.0);
        gl::Vertex3fv(pos[4].as_ptr());
        gl::Vertex3fv(pos[5].as_ptr());
        gl::Vertex3fv(pos[6].as_ptr());
        gl::Vertex3fv(pos[7].as_ptr());

        gl::End();

        gl::Begin(gl::QUADS);

        gl::Color3f(1.0, 0.0, 0.0);
        gl::Vertex3fv(pos[0].as_ptr());
        gl::Vertex3fv(pos[1].as_ptr());
        gl::Vertex3fv(pos[5].as_ptr());
        gl::Vertex3fv(pos[4].as_ptr());

        gl::End();

        gl::Begin(gl::QUADS);

        gl::Color3f(1.0, 0.0, 0.0);
        gl::Vertex3fv(pos[3].as_ptr());
        gl::Vertex3fv(pos[2].as_ptr());
        gl::Vertex3fv(pos[6].as_ptr());
        gl::Vertex3fv(pos[7].as_ptr());

        gl::End();
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, angle: &mut f64) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        },

        glfw::WindowEvent::Key(Key::D, _, Action::Press, _) => {
            *angle += 0.1;
        },
        glfw::WindowEvent::Key(Key::D, _, Action::Repeat, _) => {
            *angle += 0.1;
        },

        glfw::WindowEvent::Key(Key::A, _, Action::Press, _) => {
            *angle -= 0.1;
        },
        glfw::WindowEvent::Key(Key::A, _, Action::Repeat, _) => {
            *angle -= 0.1;
        },

        _ => {},
    }
}