// shaders.rs
use raylib::prelude::*;
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::matrix::multiply_matrix_vector4;
use crate::fragment::Fragment;
use crate::framebuffer::Framebuffer; // Importamos Framebuffer desde su módulo
use crate::triangle; // Importamos la función triangle
use crate::light::Light; // Importamos Light desde su módulo

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    // Convert vertex position to homogeneous coordinates (Vec4) by adding a w-component of 1.0
    let mut position_vec4 = Vector4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    // Modificar la posición si estamos renderizando anillos o luna
    match uniforms.render_type {
        1 => { // rings
            // Generar posición para anillos - solo coordenadas X y Z, Y es cercano a 0
            let angle = (vertex.position.x + vertex.position.z) * 3.0; // Usar coordenadas para generar ángulo
            let radius = 1.5 + (vertex.position.y * 0.1); // Variar radio basado en Y
            position_vec4.x = radius * angle.cos();
            position_vec4.z = radius * angle.sin();
            position_vec4.y = vertex.position.y * 0.1; // Hacer anillo delgado
        }
        2 => { // moon
            // Calcular posición orbital de la luna
            let moon_orbit_time = uniforms.time * 0.5; // Luna orbita más lento
            let moon_distance = 3.0; // Distancia de la luna
            let moon_x = moon_distance * moon_orbit_time.cos();
            let moon_z = moon_distance * moon_orbit_time.sin();
            let moon_y = (moon_orbit_time * 2.0).sin() * 0.5; // Movimiento vertical
            
            // Posición base de la luna
            let moon_base = Vector3::new(moon_x, moon_y, moon_z);
            
            // Añadir posición relativa del vértice
            position_vec4.x = moon_base.x + vertex.position.x * 0.3; // Luna más pequeña
            position_vec4.y = moon_base.y + vertex.position.y * 0.3;
            position_vec4.z = moon_base.z + vertex.position.z * 0.3;
        }
        _ => {} // Planet - usar posición original
    }

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
    
    // Create a new Vertex with the transformed position
    Vertex {
        position: vertex.position,
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

// Función auxiliar para calcular ruido simple
fn noise(pos: &Vector3) -> f32 {
    let x = pos.x as i32;
    let y = pos.y as i32;
    let z = pos.z as i32;
    
    let n = (x.wrapping_add(y.wrapping_mul(57)).wrapping_add(z.wrapping_mul(113))) as f32;
    ((n * n * 41597.5453).sin() * 43758.5453) % 1.0
}

// Función para generar ruido fractal (más suave)
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

// Función para simular iluminación basada en el normal
fn simulate_lighting(normal: &Vector3, light_dir: &Vector3) -> f32 {
    let mut light_dir_normalized = *light_dir;
    let length = (light_dir_normalized.x * light_dir_normalized.x + 
                 light_dir_normalized.y * light_dir_normalized.y + 
                 light_dir_normalized.z * light_dir_normalized.z).sqrt();
    
    if length > 0.0 {
        light_dir_normalized.x /= length;
        light_dir_normalized.y /= length;
        light_dir_normalized.z /= length;
    }
    
    let intensity = normal.x * light_dir_normalized.x + 
                   normal.y * light_dir_normalized.y + 
                   normal.z * light_dir_normalized.z;
    
    intensity.max(0.0).min(1.0) * 0.8 + 0.2 // Agrega algo de luz ambiente
}

// Función para aplicar rotación al planeta
fn rotate_planet_position(pos: &Vector3, time: f32, rotation_speed: f32) -> Vector3 {
    let angle = time * rotation_speed;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    
    // Rotación alrededor del eje Y (rotación axial)
    Vector3::new(
        pos.x * cos_a - pos.z * sin_a,
        pos.y,
        pos.x * sin_a + pos.z * cos_a
    )
}

fn rocky_planet_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.25); // Rotación muy lenta
    
    // Ruido base para terreno
    let base_noise = fractal_noise(&rotated_pos, 3);
    let detail_noise = fractal_noise(&Vector3::new(rotated_pos.x * 6.0, rotated_pos.y * 6.0, rotated_pos.z * 6.0), 2);
    
    // Colores: estilo lunar/desierto
    let regolith = Vector3::new(0.65, 0.65, 0.6);   // Gris lunar
    let dust = Vector3::new(0.75, 0.72, 0.68);      // Polvo claro
    let cracks = Vector3::new(0.3, 0.3, 0.35);      // Grietas oscuras
    let iron_stain = Vector3::new(0.55, 0.45, 0.4); // Manchas de óxido suave
    
    let elevation = base_noise * 0.5 + 0.5;
    
    // Terreno base
    let mut color = if elevation > 0.6 {
        dust
    } else if elevation < 0.4 {
        cracks
    } else {
        regolith
    };
    
    // Añadir manchas de óxido en zonas medias
    if elevation > 0.45 && elevation < 0.55 {
        let stain_factor = (detail_noise * 3.0).sin().abs() * 0.3;
        color = color * (1.0 - stain_factor) + iron_stain * stain_factor;
    }
    
    // Simular grietas lineales (usando ruido direccional)
    let crack_noise_x = fractal_noise(&Vector3::new(rotated_pos.y * 8.0, rotated_pos.z * 8.0, time), 2);
    let crack_noise_y = fractal_noise(&Vector3::new(rotated_pos.x * 8.0, rotated_pos.z * 8.0, time * 1.3), 2);
    let crack_lines = (crack_noise_x + crack_noise_y) * 0.5;
    if crack_lines < -0.4 {
        color = color * 0.7 + cracks * 0.3;
    }
    
    // Polvo fino que se mueve con el tiempo (efecto sutil)
    let dust_layer = fractal_noise(&Vector3::new(rotated_pos.x * 4.0, rotated_pos.y * 4.0, time * 0.05), 2);
    if dust_layer > 0.6 {
        color = color * 0.95 + dust * 0.05;
    }
    
    // Iluminación con poca luz ambiente (mundo sin atmósfera)
    let light_dir = Vector3::new(1.0, 1.0, 1.0);
    let lighting = simulate_lighting(&Vector3::new(rotated_pos.x, rotated_pos.y, rotated_pos.z), &light_dir);
    color * lighting * 0.9 + Vector3::new(0.1, 0.1, 0.12) // Muy poca luz ambiente
}

fn gas_giant_1_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.8);
    let lat = rotated_pos.z.atan2((rotated_pos.x * rotated_pos.x + rotated_pos.y * rotated_pos.y).sqrt());
    
    // Bandas horizontales
    let band1 = (lat * 8.0 + time * 0.1).sin().abs();
    let band2 = ((lat * 6.0 + 1.0).cos() + 0.5).abs();
    
    let base = Vector3::new(0.85, 0.75, 0.65); // Crema
    let dark_band = Vector3::new(0.7, 0.4, 0.2); // Naranja oscuro
    let light_band = Vector3::new(0.95, 0.85, 0.7); // Crema claro
    
    let mut color = base 
        + dark_band * band1 * 0.4 
        + light_band * band2 * 0.2;
    
    // Gran tormenta (mancha roja)
    let storm_x = (rotated_pos.x + 0.4).abs() < 0.12;
    let storm_y = (rotated_pos.y - 0.25).abs() < 0.1;
    if storm_x && storm_y {
        color = color * 0.7 + Vector3::new(0.8, 0.2, 0.1) * 0.3;
    }
    
    // Nubes turbulentas
    let cloud_noise = fractal_noise(&Vector3::new(rotated_pos.x * 10.0, rotated_pos.y * 10.0, time * 0.05), 3);
    let clouds = Vector3::new(0.98, 0.95, 0.9) * (cloud_noise * 0.3).max(0.0);
    color += clouds;
    
    // Iluminación
    let light_dir = Vector3::new(1.0, 1.0, 1.0);
    let lighting = simulate_lighting(&Vector3::new(rotated_pos.x, rotated_pos.y, rotated_pos.z), &light_dir);
    color * lighting
}

fn gas_giant_2_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.6);
    let lat = rotated_pos.z.atan2((rotated_pos.x * rotated_pos.x + rotated_pos.y * rotated_pos.y).sqrt());
    
    let base = Vector3::new(0.2, 0.3, 0.8); // Azul profundo
    let swirl_noise = fractal_noise(&Vector3::new(rotated_pos.x * 5.0, rotated_pos.y * 5.0, rotated_pos.z * 5.0 + time), 4);
    
    // Remolinos verdes/azules
    let swirl_color = Vector3::new(0.1, 0.6, 0.5); // Verde azulado
    let swirl_factor = (swirl_noise * 2.0).sin().abs() * 0.5;
    
    let mut color = base * (1.0 - swirl_factor) + swirl_color * swirl_factor;
    
    // Bandas ecuatoriales más claras
    let equator = (lat * 4.0).cos() * 0.3;
    color += Vector3::new(0.3, 0.5, 0.9) * equator;
    
    // Nubes altas
    let high_clouds = fractal_noise(&Vector3::new(rotated_pos.x * 12.0, lat * 10.0, time * 0.03), 3);
    color += Vector3::new(0.8, 0.9, 1.0) * (high_clouds * 0.2).max(0.0);
    
    // Iluminación
    let light_dir = Vector3::new(1.0, 1.0, 1.0);
    let lighting = simulate_lighting(&Vector3::new(rotated_pos.x, rotated_pos.y, rotated_pos.z), &light_dir);
    color * lighting
}

fn sci_fi_green_planet_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.4);
    
    // Ruido para biomas
    let biome_noise = fractal_noise(&rotated_pos, 4);
    let detail_noise = fractal_noise(&Vector3::new(rotated_pos.x * 6.0, rotated_pos.y * 6.0, rotated_pos.z * 6.0), 3);
    
    // Colores base
    let jungle = Vector3::new(0.1, 0.7, 0.2);   // Verde jungla
    let crystal = Vector3::new(0.0, 0.9, 0.3);  // Verde brillante (cristales)
    let toxic = Vector3::new(0.0, 0.5, 0.1);    // Verde oscuro (tóxico)
    let biolum = Vector3::new(0.0, 1.0, 0.4);   // Verde neón (bioluminiscencia)
    
    let elevation = biome_noise * 0.5 + 0.5;
    let mut color = if elevation > 0.7 {
        crystal
    } else if elevation > 0.4 {
        jungle
    } else {
        toxic
    };
    
    // Patrones de venas/cristales
    let vein_noise = fractal_noise(&Vector3::new(rotated_pos.x * 10.0, rotated_pos.y * 10.0, rotated_pos.z * 10.0 + time * 0.2), 2);
    let veins = (vein_noise * 8.0 + time).sin().abs();
    if veins > 0.8 {
        color = color * 0.6 + biolum * 0.4;
    }
    
    // Brillo pulsante en zonas altas
    let pulse = ((time * 2.0).sin() * 0.5 + 0.5) * 0.3;
    if elevation > 0.65 {
        color += biolum * pulse;
    }
    
    // Iluminación (menos dependencia de la luz solar, más auto-emisión)
    let light_dir = Vector3::new(1.0, 1.0, 1.0);
    let lighting = simulate_lighting(&Vector3::new(rotated_pos.x, rotated_pos.y, rotated_pos.z), &light_dir);
    let ambient = 0.4; // Más luz ambiente
    color * (lighting * 0.6 + ambient) + biolum * 0.1
}

fn ocean_planet_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.5);
    
    // Profundidad del océano (basado en "altura")
    let ocean_noise = fractal_noise(&rotated_pos, 3);
    let depth = ocean_noise * 0.5 + 0.5; // 0 = profundo, 1 = superficie
    
    // Colores de agua
    let deep_ocean = Vector3::new(0.0, 0.1, 0.3);   // Azul profundo
    let shallow = Vector3::new(0.0, 0.4, 0.6);      // Azul turquesa
    let foam = Vector3::new(0.8, 0.9, 1.0);         // Espuma blanca
    
    let mut color = if depth < 0.3 {
        deep_ocean
    } else if depth < 0.6 {
        shallow
    } else {
        // Pequeñas islas (muy raras)
        Vector3::new(0.3, 0.25, 0.15) // Arena/marrón
    };
    
    // Olas y espuma
    let wave_noise = fractal_noise(&Vector3::new(rotated_pos.x * 15.0, rotated_pos.y * 15.0, time * 0.1), 2);
    let waves = (wave_noise * 4.0).sin().abs();
    if waves > 0.9 && depth > 0.2 {
        color = color * 0.8 + foam * 0.2;
    }
    
    // Nubes densas (cobertura del 60%)
    let cloud_noise = fractal_noise(&Vector3::new(rotated_pos.x * 8.0, rotated_pos.y * 8.0, time * 0.02), 4);
    let cloud_coverage = if cloud_noise > 0.4 { 0.7 } else { 0.0 };
    if cloud_coverage > 0.0 {
        color = color * (1.0 - cloud_coverage) + Vector3::new(0.95, 0.97, 1.0) * cloud_coverage;
    }
    
    // Reflejo especular suave (brillo en el agua)
    let light_dir = Vector3::new(1.0, 1.0, 1.0);
    let normal = Vector3::new(rotated_pos.x, rotated_pos.y, rotated_pos.z);
    let dot = (normal.x * light_dir.x + normal.y * light_dir.y + normal.z * light_dir.z).max(0.0);
    let specular = dot.powf(32.0) * 0.5; // Brillo concentrado
    color += Vector3::new(1.0, 1.0, 1.0) * specular;
    
    // Iluminación difusa
    let lighting = simulate_lighting(&normal, &light_dir);
    color * lighting
}

// Función para renderizar anillos
pub fn render_rings(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light) {
    let mut ring_uniforms = uniforms.clone();
    ring_uniforms.render_type = 1;

    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|v| vertex_shader(v, &ring_uniforms))
        .collect();

    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            let tri = [
                &transformed_vertices[i],
                &transformed_vertices[i + 1],
                &transformed_vertices[i + 2],
            ];
            let fragments = triangle::triangle(tri[0], tri[1], tri[2], light);
            
            for fragment in fragments {
                // 🔴 Color base: rojo cálido (como rubí o polvo de óxido)
                let mut color = Vector3::new(0.75, 0.25, 0.25); // Rojo saturado pero no puro
                
                // Añadir ligera variación con ruido (simular textura de partículas)
                let noise = fractal_noise(&fragment.world_position, 2);
                let variation = 0.1 * noise;
                color.x = (color.x + variation * 0.3).max(0.0).min(1.0); // Más variación en rojo
                color.y = (color.y + variation * 0.2).max(0.0).min(1.0); // Menos en verde
                color.z = (color.z + variation * 0.1).max(0.0).min(1.0); // Muy poca en azul
                
                // Iluminación por distancia a la luz
                let dx = fragment.world_position.x - light.position.x;
                let dy = fragment.world_position.y - light.position.y;
                let dz = fragment.world_position.z - light.position.z;
                let dist_sq = dx * dx + dy * dy + dz * dz;
                let attenuation = (1.0 / (1.0 + 0.08 * dist_sq)).min(1.0);
                
                let final_color = color * (attenuation * 0.7 + 0.3);
                
                framebuffer.point(
                    fragment.position.x as i32,
                    fragment.position.y as i32,
                    final_color,
                    fragment.depth,
                );
            }
        }
    }
}

// Función para renderizar luna
pub fn render_moon(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light) {
    let mut moon_uniforms = uniforms.clone();
    moon_uniforms.render_type = 2;

    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|v| vertex_shader(v, &moon_uniforms))
        .collect();

    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            let tri = [
                &transformed_vertices[i],
                &transformed_vertices[i + 1],
                &transformed_vertices[i + 2],
            ];
            let fragments = triangle::triangle(tri[0], tri[1], tri[2], light);
            
            for fragment in fragments {
                // Generar color lunar procedural usando world_position
                let noise = fractal_noise(&fragment.world_position, 3);
                let elevation = noise * 0.5 + 0.5;
                
                let base = Vector3::new(0.65, 0.65, 0.6);
                let dust = Vector3::new(0.75, 0.72, 0.68);
                let cracks = Vector3::new(0.3, 0.3, 0.35);
                
                let color = if elevation > 0.65 {
                    dust
                } else if elevation < 0.35 {
                    cracks
                } else {
                    base
                };
                
                // Iluminación por distancia (sin normales)
                let dx = fragment.world_position.x - light.position.x;
                let dy = fragment.world_position.y - light.position.y;
                let dz = fragment.world_position.z - light.position.z;
                let dist_sq = dx * dx + dy * dy + dz * dz;
                let attenuation = (1.0 / (1.0 + 0.1 * dist_sq)).min(1.0);
                
                let final_color = color * (attenuation * 0.8 + 0.2);
                
                framebuffer.point(
                    fragment.position.x as i32,
                    fragment.position.y as i32,
                    final_color,
                    fragment.depth,
                );
            }
        }
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;
    let planet_type = uniforms.planet_type;
    
    let color = match planet_type {
        0 => rocky_planet_color(&pos, time),         // Planeta rocoso
        1 => gas_giant_1_color(&pos, time),          // Gigante gaseoso tipo Júpiter
        2 => gas_giant_2_color(&pos, time),          // Gigante gaseoso tipo Neptuno/azul-verde
        3 => sci_fi_green_planet_color(&pos, time),  // Planeta de ciencia ficción (verde)
        4 => ocean_planet_color(&pos, time),         // Planeta de agua
        _ => Vector3::new(0.5, 0.5, 0.5),            // Fallback
    };
    
    // Asegurar rango [0, 1]
    Vector3::new(
        color.x.max(0.0).min(1.0),
        color.y.max(0.0).min(1.0),
        color.z.max(0.0).min(1.0),
    )
}

pub fn set_planet_type(_planet_type: i32) {
    // Esta función podría ser usada si necesitas manejar el estado globalmente
    // Por ahora no es necesaria ya que el tipo se pasa en los uniforms
}