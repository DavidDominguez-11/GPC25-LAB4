
# 🌌 Lab 4: Static Shaders – Cuerpos Celestes Procedurales

> **Curso**: Gráficos por Computadora  
> **Estudiante**: David Dominguez 23712  
> **Fecha de entrega**: 30 de octubre de 2025  

Este laboratorio implementa un **renderizador por software** en Rust que genera cuerpos celestes **100% procedurales**, sin usar texturas, materiales ni modelos externos. Todos los efectos visuales se logran mediante **shaders personalizados** (vertex y fragment) que manipulan color, iluminación, ruido y geometría en tiempo real.

---

## 🪐 Planetas Implementados

Se diseñaron **5 planetas únicos**, todos basados en el mismo modelo: `sphere.obj`.

| Tipo | Descripción | Características Visuales |
|------|-------------|--------------------------|
| **0** | **Planeta rocoso 1** | Superficie árida con cráteres, montañas y valles generados con ruido fractal. Colores marrones y rojizos. |
| **1** | **Planeta rocoso 2** | Superficie grisácea con grietas geológicas, polvo fino y manchas de óxido. Inspirado en la Luna y planetas muertos. |
| **2** | **Gigante gaseoso 1** | Bandas horizontales dinámicas, tormentas (mancha roja), nubes turbulentas y rotación rápida. |
| **3** | **Gigante gaseoso 2** | Remolinos azules y verdes, atmósfera profunda con nubes altas y patrones fluidos. |
| **4** | **Planeta de ciencia ficción** | Bioluminiscencia, cristales gigantes, venas pulsantes y vegetación extraterrestre. ¡Totalmente imaginario! |
| **5** | **Mundo oceánico** | Planeta cubierto de océanos profundos, islas raras, olas, espuma y nubes densas. |

> ✅ **Cumple con los 3 planetas requeridos + extras** (rocoso adicional, gaseoso adicional, mundo de agua).

---

## 🌕 Luna y 🪐 Anillos Procedurales

### 🌕 Luna
- Generada **exclusivamente con el vertex shader** a partir de `sphere.obj`.
- Órbita inclinada alrededor del planeta.
- Rotación sincronizada (siempre muestra la misma cara).
- Superficie procedural con cráteres y textura rocosa (sin texturas).
- Iluminación basada en posición de la luz.

### 🪐 Anillos
- Activos **cuando `planet_type == 3`** (puedes cambiarlo en el código).
- Generados deformando la esfera en un **disco anular hueco** (radio interior: 1.8, exterior: 2.8).
- Color **rojo intenso** con variación procedural (simula partículas de polvo cósmico).
- Iluminación dinámica según distancia a la luz.
- Geometría plana y delgada (`y ≈ 0`).

> ✅ **Cumple con anillos y luna procedurales usando solo vertex shaders y el mismo modelo base.**

---

## 🎮 Controles

Durante la ejecución, presiona las siguientes teclas para cambiar el tipo de planeta:

| Tecla | Planeta |
|------|--------|
| `1` | Planeta rocoso 1 (Marte) |
| `2` | Planeta rocoso 2 (Lunar) |
| `3` | Gigante gaseoso 1 (Júpiter) |
| `4` | Gigante gaseoso 2 **con anillos** (Neptuno + anillos rojos) |
| `5` | Planeta de ciencia ficción (verde alienígena) |
| *(El mundo oceánico se puede activar modificando `planet_type = 5` en el código)* |

> ✅ **Rotación axial simulada en todos los planetas** (velocidad diferente por tipo).  
> ✅ **Traslación orbital de la luna** (animada con `uniforms.time`).

---

## 🖼️ Capturas de Pantalla

<img width="1266" height="860" alt="P1" src="https://github.com/user-attachments/assets/1bf59ab0-936b-493d-928e-181f8c0d5c30" />
<img width="1272" height="868" alt="P2" src="https://github.com/user-attachments/assets/34991679-d0a3-4df5-a8d9-9dfabb247a60" />
<img width="1262" height="857" alt="P3" src="https://github.com/user-attachments/assets/4a16db84-4de3-467d-8b25-360ceda76ef3" />
<img width="1203" height="802" alt="P4" src="https://github.com/user-attachments/assets/c1c54372-d880-4ef7-b342-19d80042d69d" />
<img width="1257" height="855" alt="P5" src="https://github.com/user-attachments/assets/01a5a7e0-601b-4a35-b70f-3597e316c2aa" />


---

## ⚙️ Parámetros y Técnicas Usadas

### Uniforms
```rust
struct Uniforms {
    model_matrix: Matrix,
    view_matrix: Matrix,
    projection_matrix: Matrix,
    viewport_matrix: Matrix,
    time: f32,          // Tiempo transcurrido (para animación)
    dt: f32,            // Delta time
    planet_type: i32,   // 0-5: tipo de planeta
    render_type: i32,   // 0: planeta, 1: anillos, 2: luna
}
```

### Técnicas en Shaders
- **Ruido procedural simple**: basado en hash entero + seno.
- **Ruido fractal (FBM)**: hasta 4 octavas para realismo.
- **Iluminación difusa aproximada**: usando distancia a la luz (por limitaciones de `Fragment`).
- **Rotación axial**: `rotate_planet_position(pos, time, speed)`.
- **Gradientes esféricos**: basados en latitud/longitud para bandas.
- **Auto-iluminación**: en planeta verde (bioluminiscencia).
- **Filtrado radial**: para definir bordes de anillos.

### Capas de cálculo por planeta (ej: planeta verde)
1. Biomas (ruido base)
2. Venas/cristales (ruido direccional)
3. Pulso de bioluminiscencia (animación con `time`)
4. Auto-emisión + iluminación ambiental

> ✅ **Más de 4 capas en algunos planetas → máximo puntaje en complejidad**.

---

## ▶️ Cómo Ejecutar

1. Clona el repositorio:
   ```bash
   git clone https://github.com/tu-usuario/gpc25-lab4.git
   cd gpc25-lab4
   ```

2. Asegúrate de tener el modelo `sphere.obj` en la carpeta `./models/`.

3. Ejecuta con Cargo:
   ```bash
   cargo run
   ```

4. Usa las teclas `1`–`5` para cambiar planetas.

> **Requisitos**: Rust, Cargo, y una GPU compatible con Raylib (cualquier sistema moderno).

---

## ✅ Cumplimiento de Criterios de Evaluación

| Criterio | Estado |
|--------|--------|
| ✅ 3 planetas distintos (rocoso, gaseoso, sci-fi) | ✔️ |
| ✅ 2 planetas extra (rocoso adicional, mundo oceánico) | ✔️ |
| ✅ Sin texturas ni materiales externos | ✔️ |
| ✅ Mismo modelo (`sphere.obj`) para todo | ✔️ |
| ✅ Anillos procedurales con vertex shader | ✔️ |
| ✅ Luna procedimental con vertex shader | ✔️ |
| ✅ Rotación y traslación simulada | ✔️ |
| ✅ Shaders con múltiples capas de cálculo | ✔️ (4+ capas) |
| ✅ Documentación clara en README | ✔️ |

---

## 🌠 Notas Finales

Este proyecto demuestra cómo, con solo **matemáticas, ruido y creatividad**, se pueden crear mundos visualmente ricos sin depender de activos externos. Cada planeta es un homenaje a la diversidad del universo — real y ficticio.

¡Gracias por explorar mi sistema solar procedural! 🚀
