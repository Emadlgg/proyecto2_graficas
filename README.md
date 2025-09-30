# Proyecto 2: Diorama 3D con Raytracing en Rust

## 📋 Descripción
Este proyecto implementa un renderizador 3D basado en **raytracing** desarrollado completamente en **Rust**, sin utilizar librerías de renderizado externas.  
El diorama presenta una escena inspirada en *Minecraft* con múltiples materiales, texturas procedurales y efectos físicos realistas como reflexión, refracción y skybox dinámico.

La escena incluye dos **cuevas temáticas**:  
- Una **cueva de agua** con bloques de obsidiana  
- Una **cueva de lava**  
Ambas conectadas por un área exterior con escaleras de césped y paredes de piedra.

---

## ✨ Características Principales

### Materiales Implementados
El proyecto incluye **6 materiales distintos**, cada uno con propiedades físicas únicas:

- **Césped**: Textura procedural verde con variaciones naturales  
- **Tierra**: Tonos marrones con ruido procedural  
- **Piedra**: Textura grisácea base para paredes y estructuras  
- **Agua**: Material transparente con refracción realista (IOR: 1.33)  
- **Lava**: Material emisivo que genera su propia luz  
- **Obsidiana**: Superficie reflectiva con alto valor especular  

### Efectos Físicos
- **Refracción**: Implementada en el agua con índice de refracción 1.33 y algoritmo de Fresnel  
- **Reflexión**: Superficies reflectivas en obsidiana y agua  
- **Emisión de luz**: La lava genera iluminación propia  
- **Skybox procedural**: Cielo dinámico de 6 caras con degradados atmosféricos  

### Cámara Interactiva
Sistema de cámara orbital con controles completos:

- Rotación horizontal y vertical con teclas de flecha  
- Zoom con W/S  
- Reset de posición con barra espaciadora  
- Modo rápido con Shift  

---

## 🏗️ Arquitectura Técnica

### Estructura de la Escena
- Grid de **12×12 cubos** con generación procedural de terreno  
- Sistema de alturas variables con suavizado  
- Dos cuevas con techos y estructuras internas  
- Optimización mediante **Bounding Box** para intersecciones de rayos  

### Sistema de Iluminación
- **2 luces dinámicas** con colores y posiciones diferentes  
- Cálculo de sombras en tiempo real  
- Atenuación física por distancia  
- Luz ambiente ajustada por tipo de material  

---

## 🛠️ Tecnologías

- **Lenguaje**: Rust (Edition 2024)  
- **Dependencias**:  
  ```toml
  nalgebra-glm = "0.19"   # Álgebra lineal  
  minifb = "0.27"         # Ventana y manejo de input  
  image = "0.25"          # Carga de texturas  
  ```

---

## 🚀 Instalación y Uso

```bash
# Clonar el repositorio
git clone https://github.com/Emadlgg/proyecto2_graficas.git

# Navegar al directorio
cd proyecto2_graficas

# Compilar y ejecutar
cargo run --release
```

### 🎮 Controles

| Tecla        | Acción                              |
|--------------|-------------------------------------|
| ← / →        | Rotar cámara horizontalmente        |
| ↑ / ↓        | Rotar cámara verticalmente          |
| W            | Acercar zoom                        |
| S            | Alejar zoom                         |
| Space        | Resetear cámara                     |
| Shift        | Movimiento rápido                   |
| Esc          | Salir                               |

---

## 📊 Especificaciones

- Resolución: **400×300 píxeles**  
- Frame rate: **30 FPS**  
- Profundidad de raytracing: **3 niveles de recursión**  
- Optimizaciones: *Bounding box culling*, *shadow ray reduction*  

---

## 🎬 Video Demostración
[![Ver en YouTube](https://img.youtube.com/vi/I01pmv6Q1Ao/0.jpg)](https://www.youtube.com/watch?v=I01pmv6Q1Ao)

---

## 👨‍💻 Autor
**Emadlgg**  
Universidad del Valle de Guatemala  
Curso: Gráficas por Computadora  

---

## 📝 Licencia
Proyecto académico con fines educativos.

<div align="center">
  <sub>Desarrollado con raytracing en Rust</sub>
</div>
