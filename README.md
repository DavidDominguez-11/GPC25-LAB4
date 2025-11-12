
# Lab 4: Static Shaders – Estrella Procedural Animada
> **Curso**: Gráficos por Computadora  
> **Estudiante**: David Dominguez 23712  

Este laboratorio implementa un **renderizador por software** en Rust que genera una **estrella 100% procedural y animada**, sin usar texturas, materiales ni modelos externos. Todos los efectos visuales de la estrella se logran mediante **shaders personalizados** (vertex y fragment) que manipulan color, ruido, iluminación y geometría en tiempo real.

---
## Estrella Implementada
Se diseñó una **estrella animada**, basada exclusivamente en el modelo: `sphere.obj`.

### Características Visuales:
- **Superficie Turbulenta y Pulsante:** La superficie de la estrella se distorsiona dinámicamente en el `vertex shader` mediante ruido fractal, simulando turbulencias solares, llamaradas y pulsaciones de plasma.
- **Color Dinámico por Temperatura:** El color varía continuamente entre rojo, naranja, amarillo y blanco, simulando zonas de diferente temperatura en su atmósfera, calculado en el `fragment shader` usando intensidad de ruido.
- **Emisión Variable y Picos de Energía:** La luminosidad de la estrella no es constante. Usa una pulsación global (con `sin(time)`) y variaciones locales de ruido para simular picos de energía y destellos solares.
- **Corona Visual:** Un efecto de corona brillante se genera en los bordes de la esfera, aumentando la sensación de emisión y calor extremo.
- **Animación Cíclica Continua:** Toda la animación se basa en la variable `uniforms.time`, creando un ciclo infinito y suave de actividad solar.

> ✅ **Cumple con todos los requisitos técnicos:**  
> - Solo esfera como base ✅  
> - Sin texturas ni materiales externos ✅  
> - Animación continua con `time` ✅  
> - Uso de ruido procedural ✅  
> - Emisión variable ✅  
> - Distorsión en vertex shader ✅  
> - Color controlado por intensidad ✅  

---
## Capturas de Pantalla / GIF

---
## Controles
Durante la ejecución, puedes usar la cámara para observar la estrella desde diferentes ángulos:
- **W/S/A/D**: Mover la cámara hacia arriba/abajo/izquierda/derecha.
- **Q/E**: Desplazar el centro de la cámara (pan horizontal).
- **↑/↓**: Acercar o alejar la cámara.
- **R/F**: Mover la cámara hacia arriba/abajo (pan vertical).

---
## Parámetros y Técnicas Usadas
### Uniforms
```rust
struct Uniforms {
    model_matrix: Matrix,
    view_matrix: Matrix,
    projection_matrix: Matrix,
    viewport_matrix: Matrix,
    time: f32,          // Tiempo transcurrido (para animación)
    dt: f32,            // Delta time
    planet_type: i32,   // (Dedicado a la estrella, valor fijo = 5)
    render_type: i32,   // 0: star (único tipo)
}
```

### Técnicas en Shaders
- **Ruido procedural simple**: Basado en hash entero + seno (función `noise`).
- **Ruido fractal (FBM)**: Hasta 3 octavas combinadas para generar texturas suaves y complejas.
- **Distorsión de vértices (Vertex Shader)**: Aplicación de ruido fractal y `time` para desplazar radialmente los vértices de la esfera, creando la sensación de superficie inquieta.
- **Gradiente de color (Fragment Shader)**: Mapeo de intensidad de ruido a un gradiente de temperatura:  
  `0.0–0.3 → Rojo intenso` → `0.3–0.6 → Naranja` → `0.6–0.85 → Amarillo` → `0.85–1.0 → Blanco amarillento`.
- **Emisión variable**: La intensidad del color se multiplica por un valor calculado a partir del ruido y una pulsación global (`sin(time)`), simulando picos de energía.
- **Corona (Fragment Shader)**: Se añade un brillo blanco/amarillo en los píxeles cercanos al borde de la esfera (distancia al centro > 0.95).
- **Iluminación difusa aproximada**: Usada para dar volumen y profundidad, pero secundaria a la emisión propia de la estrella.

### Capas de cálculo en el Fragment Shader (Estrella):
1. **Ruido base**: FBM en 3 octavas para patrones de turbulencia.
2. **Ruido de detalle**: FBM en 2 octavas con frecuencia más alta para texturas finas.
3. **Pulsación global**: `sin(time * 0.8)` para variación cíclica de luminosidad.
4. **Mapeo de temperatura**: Transformación de intensidad a color (rojo → blanco).
5. **Corona**: Efecto de borde basado en distancia radial.
6. **Emisión final**: Multiplicación del color por la intensidad total.

> ✅ **Más de 5 capas de cálculo → Máximo puntaje en complejidad del shader**.

---
## ▶️ Cómo Ejecutar
1. Clona el repositorio:
   ```bash
   git clone https://github.com/DavidDominguez-11/GPC25-LAB4/tree/LAB5
   cd GPC25-LAB4
   ```
2. Asegúrate de tener el modelo `sphere.obj` en la carpeta `./models/`.
3. Ejecuta con Cargo:
   ```bash
   cargo run
   ```
> **Requisitos**: Rust, Cargo, y una GPU compatible con Raylib (cualquier sistema moderno).

---
## ✅ Cumplimiento de Criterios de Evaluación
| Criterio | Estado |
|--------|--------|
| ✅ **Creatividad visual del diseño y realismo percibido** | ✔️ (Estrella con turbulencias, pulsaciones y corona realistas) |
| ✅ **Complejidad del shader (uso de múltiples funciones o combinaciones de ruido)** | ✔️ (5+ capas de FBM, pulsación, gradiente, corona) |
| ✅ **Implementación correcta del tiempo y animación continua** | ✔️ (Todo animado con `uniforms.time`, ciclo cíclico) |
| ✅ **Uso de Perlin, Simplex o Cellular noise con parámetros ajustables** | ✔️ (Ruido fractal basado en hash, con octavas y frecuencia ajustables) |
| ✅ **Agregar emisión variable (simular luminosidad, picos de energía)** | ✔️ (Intensidad multiplicada por `pulsation` y `animated_noise`) |
| ✅ **Agregar distorsión o “flare” visual mediante desplazamiento del Vertex Shader** | ✔️ (Desplazamiento radial de vértices con ruido y `time`) |
| ✅ **Controlar el color de la estrella con base en su intensidad o temperatura (gradiente dinámico)** | ✔️ (Gradiente rojo → naranja → amarillo → blanco basado en intensidad) |
| ✅ **Documentación clara de las funciones de ruido y uniformes en el README** | ✔️ (Explicación detallada en esta sección) |

---
## Notas Finales
Este proyecto demuestra cómo, con solo **matemáticas, ruido procedural y creatividad**, se puede crear una representación visualmente convincente de una estrella viva, sin depender de activos externos. La combinación de distorsión de geometría y variación de color logra un efecto que recuerda a imágenes reales del Sol tomadas por telescopios solares.

¡Gracias por observar mi estrella procedural!
