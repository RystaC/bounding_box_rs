use std::f64;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

type Coord = [f32; 3];
type Triangle = [Coord; 3];
type Color = Triangle;
type AABB = [Coord; 2];

fn main() {
    let width = 512;
    let height = 512;

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(width, height, "bounding box", glfw::WindowMode::Windowed).unwrap();

    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s));

    unsafe {
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Enable(gl::DEPTH_TEST);

        gl::MatrixMode(gl::PROJECTION);
        gl::LoadIdentity();
        gluPerspective(45.0, (width / height) as GLdouble, 0.1, 20.0);

        gl::MatrixMode(gl::MODELVIEW);
        gl::LoadIdentity();
        gluLookAt(3.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    }

    let mut angle = 0.0;

    let mut triangles = read_off_to_triangles("resources/bunny.off");
    triangles.sort_by(|a, b| a.partial_cmp(&b).unwrap());

    let color = [[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];

    let aabbs = construct_aabbs(triangles.as_slice(), 0, 5);

    while !window.should_close() {
        glfw.poll_events();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::MatrixMode(gl::MODELVIEW);
            gl::LoadIdentity();
            gluLookAt(3.0 * f64::cos(angle * f64::consts::PI), 0.0, 3.0 * f64::sin(angle * f64::consts::PI), 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);

            gl::PushMatrix();
            gl::Translatef(0.0, -1.0, 0.0);
            gl::Scalef(10.0, 10.0, 10.0);

            for i in 0..triangles.len() { draw_triangle(triangles[i], color); };
            for i in 0..aabbs.len() { draw_aabb(aabbs[i]); };

            gl::PopMatrix();
        }

        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut angle);
        }

        window.swap_buffers();
    }
}

fn read_off_to_triangles(path: &str) -> Vec<Triangle> {
    let mut reader = BufReader::new(File::open(path).unwrap());
    
    let mut header_buf = String::new();
    reader.read_line(&mut header_buf).unwrap();
    header_buf.pop();
    if header_buf != "OFF" { panic!("input file format is invalid."); };

    let mut num_data_buf = String::new();
    reader.read_line(&mut num_data_buf).unwrap();
    num_data_buf.pop();
    let token_buffer: Vec<&str> = num_data_buf.as_mut_str().split(' ').collect();

    let num_vertices = token_buffer[0].parse::<usize>().unwrap();
    let num_faces = token_buffer[1].parse::<usize>().unwrap();

    let mut vertices: Vec<Coord> = Vec::new();
    for _ in 0..num_vertices {
        let mut vertices_buf = String::new();
        reader.read_line(&mut vertices_buf).unwrap();
        vertices_buf.pop();
        let token_buffer: Vec<&str> = vertices_buf.split(' ').collect();

        vertices.push([token_buffer[0].parse::<f32>().unwrap(), token_buffer[1].parse::<f32>().unwrap(), token_buffer[2].parse::<f32>().unwrap()]);
    };

    let mut triangles: Vec<Triangle> = Vec::new();
    for _ in 0..num_faces {
        let mut indices_buf = String::new();
        reader.read_line(&mut indices_buf).unwrap();
        indices_buf.pop();
        let token_buffer: Vec<&str> = indices_buf.split(' ').collect();

        if token_buffer[0] != "3" { panic!("there are non-triangle faces."); };

        let indices = [token_buffer[1].parse::<usize>().unwrap(), token_buffer[2].parse::<usize>().unwrap(), token_buffer[3].parse::<usize>().unwrap()];
        triangles.push([vertices[indices[0]], vertices[indices[1]], vertices[indices[2]]]);
    };

    triangles
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

fn construct_aabbs(triangles: &[Triangle], times: usize, limit: usize) -> Vec<AABB> {
    if times > limit || times > (triangles.len() as f64).log2() as usize { vec![] }
    else {
        let mut aabbs = vec![create_aabb(triangles)];
        aabbs.append(construct_aabbs(&triangles[0..triangles.len()/2], times + 1, limit).as_mut());
        aabbs.append(construct_aabbs(&triangles[triangles.len()/2..triangles.len()], times + 1, limit).as_mut());
        aabbs
    }
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