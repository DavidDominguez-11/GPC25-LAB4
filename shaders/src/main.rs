// main.rs
mod framebuffer;
mod triangle;
mod obj;
mod matrix;
mod fragment;
mod vertex;
mod camera;
mod shaders;
mod light;
use triangle::triangle;
use obj::Obj;
use framebuffer::Framebuffer;
use raylib::prelude::*;
use std::thread;
use std::time::Duration;
use std::f32::consts::PI;
use matrix::{create_model_matrix, create_projection_matrix, create_viewport_matrix};
use vertex::Vertex;
use camera::Camera;
use shaders::{vertex_shader, fragment_shader}; // Ya no necesitamos render_rings/render_moon
use light::Light;
#[derive(Clone)]
pub struct Uniforms {
    pub model_matrix: Matrix,
    pub view_matrix: Matrix,
    pub projection_matrix: Matrix,
    pub viewport_matrix: Matrix,
    pub time: f32, // elapsed time in seconds
    pub dt: f32, // delta time in seconds
    pub planet_type: i32, // Ahora dedicado a la estrella (e.g., 5)
    pub render_type: i32, // 0: star
}

fn render_star(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    let mut star_uniforms = uniforms.clone();
    star_uniforms.render_type = 0; // star
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, &star_uniforms);
        transformed_vertices.push(transformed);
    }
    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }
    // Rasterization Stage
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], light));
    }
    // Fragment Processing Stage
    for fragment in fragments {
        let final_color = fragment_shader(&fragment, uniforms);
        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            final_color,
            fragment.depth,
        );
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Procedural Star - Vertex Distortion & Noise")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    let mut framebuffer = Framebuffer::new(window_width, window_height);

    // Inicializar cámara para enfocar la estrella
    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0), // eye - más cerca para la estrella
        Vector3::new(0.0, 0.0, 0.0), // target
        Vector3::new(0.0, 1.0, 0.0), // up
    );

    // Parámetros de transformación del modelo (fijos para la estrella)
    let translation = Vector3::new(0.0, 0.0, 0.0);
    let scale = 1.0; // Puedes ajustar el tamaño de la estrella
    let rotation = Vector3::new(0.0, 0.0, 0.0);

    // Light (La luz puede actuar como el centro de la estrella o una fuente externa)
    let light = Light::new(Vector3::new(0.0, 0.0, 0.0)); // Colocada en el centro
    let obj = Obj::load("./models/sphere.obj").expect("Failed to load obj");
    let vertex_array = obj.get_vertex_array();

    framebuffer.set_background_color(Color::new(135, 206, 235, 255)); // Fondo
    let mut time = 0.0;
    // No necesitamos cambiar tipos, es solo una estrella
    let star_type = 5; // Usamos 5 para representar la estrella

    while !window.window_should_close() {
        let dt = window.get_frame_time();
        time += dt;

        // Opcional: Control de cámara
        camera.process_input(&window);

        framebuffer.clear();
        framebuffer.set_current_color(Color::new(200, 200, 255, 255));

        // Crear matrices de transformación
        let model_matrix = create_model_matrix(translation, scale, rotation);
        let view_matrix = camera.get_view_matrix();
        let projection_matrix = create_projection_matrix(PI / 3.0, window_width as f32 / window_height as f32, 0.1, 100.0);
        let viewport_matrix = create_viewport_matrix(0.0, 0.0, window_width as f32, window_height as f32);

        // Renderizar la estrella
        let star_uniforms = Uniforms {
            model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            dt,
            planet_type: star_type, // Siempre 5
            render_type: 0,         // Siempre estrella
        };
        render_star(&mut framebuffer, &star_uniforms, &vertex_array, &light);

        framebuffer.swap_buffers(&mut window, &raylib_thread);
        thread::sleep(Duration::from_millis(16));
    }
}