// shaders.rs
use raylib::prelude::*;
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::matrix::multiply_matrix_vector4;
use crate::fragment::Fragment;
use crate::framebuffer::Framebuffer;
use crate::triangle;
use crate::light::Light;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    // Convert vertex position to homogeneous coordinates (Vec4) by adding a w-component of 1.0
    let mut position_vec4 = Vector4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    // --- Distorsión de la estrella basada en ruido y tiempo ---
    // Solo aplica si estamos renderizando la estrella
    if uniforms.render_type == 0 { // render_type 0 = star
        let pos = vertex.position; // Posición original del vértice
        let time = uniforms.time;

        // Calcular ruido basado en la posición original del vértice
        // Usamos una frecuencia más baja para distorsiones suaves
        let noise_input = Vector3::new(pos.x * 1.5, pos.y * 1.5, pos.z * 1.5 + time * 0.1); // Ajusta la frecuencia y velocidad
        let noise_factor = fractal_noise(&noise_input, 2); // Usar FBM para suavidad

        // Calcular una pequeña perturbación radial basada en ruido y tiempo
        let amplitude = 0.05; // Ajusta la magnitud de la distorsión
        let time_factor = (time * 0.8).sin(); // O cos, o una combinación
        let displacement = noise_factor * amplitude * time_factor;

        // Aplicar la perturbación en la dirección del vértice (radialmente)
        position_vec4.x += displacement * pos.x;
        position_vec4.y += displacement * pos.y;
        position_vec4.z += displacement * pos.z;
    }

    // Aplicar transformaciones estándar (Model, View, Projection, Viewport)
    // Apply Model transformation
    let world_position = multiply_matrix_vector4(&uniforms.model_matrix, &position_vec4);
    // Apply View transformation (camera)
    let view_position = multiply_matrix_vector4(&uniforms.view_matrix, &world_position);
    // Apply Projection transformation (perspective)
    let clip_position = multiply_matrix_vector4(&uniforms.projection_matrix, &view_position);
    // Perform perspective division to get NDC (Normalized Device Coordinates)
    let ndc = if clip_position.w != 0.0 {
        Vector3::new(
            clip_position.x / clip_position.w,
            clip_position.y / clip_position.w,
            clip_position.z / clip_position.w,
        )
    } else {
        Vector3::new(clip_position.x, clip_position.y, clip_position.z)
    };
    // Apply Viewport transformation to get screen coordinates
    let ndc_vec4 = Vector4::new(ndc.x, ndc.y, ndc.z, 1.0);
    let screen_position = multiply_matrix_vector4(&uniforms.viewport_matrix, &ndc_vec4);
    let transformed_position = Vector3::new(
        screen_position.x,
        screen_position.y,
        screen_position.z,
    );

    // Create a new Vertex with the potentially distorted position and other attributes
    Vertex {
        position: vertex.position, // Mantener la posición original para cálculos en el fragment shader
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position,
        transformed_normal: transform_normal(&vertex.normal, &uniforms.model_matrix),
    }
}

fn transform_normal(normal: &Vector3, model_matrix: &Matrix) -> Vector3 {
    // Convierte el normal a coordenadas homogéneas (añade coordenada w = 0.0)
    let normal_vec4 = Vector4::new(normal.x, normal.y, normal.z, 0.0);
    let transformed_normal_vec4 = multiply_matrix_vector4(model_matrix, &normal_vec4);
    // Convierte de vuelta a Vector3 y normaliza
    let mut transformed_normal = Vector3::new(
        transformed_normal_vec4.x,
        transformed_normal_vec4.y,
        transformed_normal_vec4.z,
    );
    transformed_normal.normalize();
    transformed_normal
}

// Función auxiliar para calcular ruido simple (ya existente)
fn noise(pos: &Vector3) -> f32 {
    let x = pos.x as i32;
    let y = pos.y as i32;
    let z = pos.z as i32;
    let n = (x.wrapping_add(y.wrapping_mul(57)).wrapping_add(z.wrapping_mul(113))) as f32;
    ((n * n * 41597.5453).sin() * 43758.5453) % 1.0
}

// Función para generar ruido fractal (más suave) (ya existente)
fn fractal_noise(pos: &Vector3, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    for _ in 0..octaves {
        value += noise(&Vector3::new(pos.x * frequency, pos.y * frequency, pos.z * frequency)) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    value
}

// Función para renderizar la estrella
fn star_color(pos: &Vector3, time: f32) -> Vector3 {
    // --- PASO 1: Calcular Ruido Dinámico ---
    let pos_scaled = Vector3::new(pos.x * 2.0, pos.y * 2.0, pos.z * 2.0);
    let base_noise = fractal_noise(&Vector3::new(pos_scaled.x + time * 0.2, pos_scaled.y + time * 0.2, pos_scaled.z + time * 0.2), 3);
    let detail_noise = fractal_noise(&Vector3::new(pos_scaled.x * 6.0 + time * 0.5, pos_scaled.y * 6.0 + time * 0.5, pos_scaled.z * 6.0 + time * 0.5), 2);
    let animated_noise = base_noise + detail_noise * 0.3;

    // --- PASO 2: Calcular Intensidad/Luminosidad ---
    let intensity_base = animated_noise * 0.5 + 0.5; // Escalar a [0, 1]
    let pulsation = (time * 0.8).sin().abs() * 0.1; // Pulsación global suave
    let local_intensity = (intensity_base + pulsation).min(1.0);

    // --- PASO 3: Determinar Color Basado en Intensidad (Temperatura) ---
    // Gradiente: Rojo (frío) -> Naranja -> Amarillo -> Blanco (caliente)
    let color = if local_intensity < 0.3 {
        Vector3::new(0.8, 0.2, 0.1) // Rojo intenso (manchas calientes)
    } else if local_intensity < 0.6 {
        Vector3::new(1.0, 0.6, 0.1) // Naranja
    } else if local_intensity < 0.85 {
        Vector3::new(1.0, 0.9, 0.3) // Amarillo
    } else {
        Vector3::new(1.0, 1.0, 0.8) // Blanco amarillento
    };

    // --- PASO 4: Aplicar Emisión Variable ---
    let final_color = color * local_intensity;

    // --- PASO 5: Ajustar para que luzca como emisión ---
    let distance_from_center = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt();
    if distance_from_center > 0.95 { // Ajusta el umbral para la corona
        let corona_intensity = (1.0 - distance_from_center).max(0.0) * 5.0; // Borde brillante
        return final_color + Vector3::new(1.0, 0.8, 0.4) * corona_intensity;
    }

    // Asegurar rango [0, 1]
    Vector3::new(
        final_color.x.max(0.0).min(1.0),
        final_color.y.max(0.0).min(1.0),
        final_color.z.max(0.0).min(1.0),
    )
}

// Fragment Shader principal - ahora solo para la estrella
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position; // Usamos la posición original del vértice
    let time = uniforms.time;

    // Llama a la función específica de la estrella
    let color = star_color(&pos, time);

    // Asegurar rango [0, 1]
    Vector3::new(
        color.x.max(0.0).min(1.0),
        color.y.max(0.0).min(1.0),
        color.z.max(0.0).min(1.0),
    )
}
