# CopyMaster

**CopyMaster — Gestor avanzado de copias para Linux**

Aun en desarrollo, cualquier contribuciony optimización es bienbenida.

![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg) ![crates.io](https://img.shields.io/badge/crate-copymaster-lightgrey) ![build](https://img.shields.io/badge/build-passing-brightgreen)

CopyMaster es una aplicación para gestionar transferencias de archivos en sistemas Linux, con interfaz gráfica moderna (GTK4 + libadwaita) y una línea de comandos potente. Está diseñada para manejar grandes volúmenes de datos, ofrecer copias paralelas y verificadas, y resolver conflictos de forma interactiva o automática.

---

## Tabla de contenidos

- [Características](#características)
- [Capturas](#capturas)
- [Instalación](#instalación)
  - [Requisitos](#requisitos)
  - [Compilar desde código fuente](#compilar-desde-código-fuente)
  - [AppImage (Linux)](#appimage-linux)
- [Uso](#uso)
  - [Interfaz gráfica (GUI)](#interfaz-gráfica-gui)
  - [Línea de comandos (CLI)](#línea-de-comandos-cli)
- [Configuración](#configuración)
- [Resolución de conflictos](#resolución-de-conflictos)
- [Desarrollo](#desarrollo)
  - [Contribuir](#contribuir)
  - [Pruebas y estilos](#pruebas-y-estilos)
- [Empaquetado y releases](#empaquetado-y-releases)
- [Licencia](#licencia)
- [Contacto y agradecimientos](#contacto-y-agradecimientos)

---

## Características

- Interfaz GTK4 + libadwaita moderna y accesible
- Motor de copia configurable: copias estándar, por chunks en paralelo y con verificación
- Resolución de conflictos: sobrescribir, renombrar (nuevo/antiguo), saltar o preguntar al usuario
- Modo daemon para operaciones en segundo plano y control desde CLI
- Integración con bandeja del sistema y notificaciones de escritorio
- Optimización por tipo de dispositivo (USB/HDD/SSD) y manejo de colas por dispositivo
- Pausa/reanudación de transferencias, estadísticas en tiempo real y verificación de integridad
- Configuración persistente en JSON en $XDG_CONFIG_HOME/copymaster/config.json

---

## Capturas



---

## Instalación

### Requisitos

- Rust (toolchain estable) — instala con `rustup`
- Librerías de desarrollo de GTK4 y libadwaita (paquetes del sistema)
  - Debian/Ubuntu (ejemplo):

```bash
sudo apt update
sudo apt install -y build-essential libglib2.0-dev libgtk-4-dev libadwaita-1-dev libgdk-pixbuf-2.0-dev pkg-config libssl-dev
```

  - Fedora (ejemplo):

```bash
sudo dnf install -y gtk4-devel libadwaita-devel gdk-pixbuf2-devel pkgconfig openssl-devel
```

> Nota: los nombres exactos de paquetes pueden variar según la distribución.

### Compilar desde código fuente

```bash
# Clonar
git clone https://github.com/alepc011017/copymaster.git
cd copymaster

# Compilar con GUI y notificaciones (opcional):
cargo build --release --features "gui,notifications"

# Ejecutar binario:
./target/release/copymaster

# (Opcional) Instalar archivos del sistema
sudo make install
```

### AppImage (Linux)

Se incluye un script de construcción `scripts/build-appimage.sh` que crea una AppImage portable:

```bash
# Requisitos: linuxdeploy + appimagetool (el script puede descargarlos)
./scripts/build-appimage.sh
# Resultado: dist/copymaster-x86_64.AppImage
```

---

## Uso

### Interfaz gráfica (GUI)

Lanza la aplicación desde el menú de aplicaciones o ejecutando el binario `copymaster`.

- Arrastra y suelta archivos/carpetas sobre la ventana o dispositivos montados para iniciar transferencias.
- Usa el panel de colas para ver, pausar y reanudar operaciones.

### Línea de comandos (CLI)

`copymaster` incluye comandos útiles:

```bash
# Ejecutar en modo demonio (sin UI):
copymaster --daemon

# Habilitar/deshabilitar auto-arranque:
copymaster autostart --enable --minimized
copymaster autostart --disable

# Instalar archivos del sistema (iconos, desktop):
copymaster install --with-desktop

# Mostrar información del sistema:
copymaster systeminfo
```

---

## Configuración

La configuración se guarda en JSON en:

- Linux: `$XDG_CONFIG_HOME/copymaster/config.json` (normalmente `~/.config/copymaster/config.json`)

Ejemplo simplificado (`config.json`):

```json
{
  "autostart_enabled": false,
  "start_minimized": false,
  "minimize_to_tray": true,
  "show_notifications": true,
  "default_copy_options": {
    "algorithm": "ParallelChunks",
    "buffer_size": 65536,
    "max_threads": 8,
    "verify_after_copy": true,
    "conflict_resolution": "Ask",
    "preserve_attributes": true,
    "sparse_files": true,
    "sync_io": false
  },
  "remembered_devices": [],
  "window_state": {
    "width": 1000,
    "height": 700,
    "x": -1,
    "y": -1,
    "maximized": false
  },
  "conflict_resolution": {
    "default_action": "Ask",
    "ask_for_confirmation": true,
    "rename_pattern": "{name} ({counter})"
  }
}
```

Puedes editar este archivo manualmente o usar la interfaz/configuración de la aplicación para actualizarlo.

---

## Resolución de conflictos

Cuando la aplicación detecta que un archivo destino ya existe, soporta las siguientes acciones:

- `Ask` — Preguntar al usuario (comportamiento por defecto)
- `Overwrite` / `OverwriteAll` — Sobrescribir
- `RenameNew` / `RenameOld` — Renombrar el nuevo o el antiguo
- `Skip` / `SkipAll` — Saltar el archivo

Las opciones `*All` aplican la acción a todos los conflictos futuros en la sesión.

---

## Desarrollo

### Dependencias útiles

- Rust + cargo
- `cargo fmt`, `cargo clippy` para estilo y linting

### Comandos frecuentes

```bash
# Compilar en modo debug
cargo build

# Ejecutar tests
cargo test

# Ejecutar con características (GUI)
cargo run --features gui

# Formatear código
cargo fmt

# Ejecutar Clippy
cargo clippy --all-targets --all-features -- -D warnings
```

### Contribuir

¡Gracias por querer contribuir! Sugerencias:

- Abre un issue para discutir cambios grandes
- Crea ramas con nombres claros: `feature/<nombre>` o `fix/<nombre>`
- Sigue el estilo de código de Rust (utiliza `cargo fmt` y `clippy`)
- Incluye pruebas para cambios lógicos cuando sea posible

---

## Empaquetado y releases

- El script `scripts/build-appimage.sh` crea una AppImage para distribución en Linux.
- `Makefile` incluye objetivos útiles para instalación y empaquetado (`make install`, `make uninstall`).
- Para subir releases, usa `scripts/create-release.sh` (ajusta la versión y los assets según corresponda).

---

## Licencia

CopyMaster está licenciado bajo la **GPL-3.0**. Consulta el archivo `LICENSE` para más detalles.

---

## Contacto y agradecimientos

- Autor: Alejandro Pérez <alepc011017+github@gmail.com>
- Repositorio: https://github.com/Alepc011017/copymaster

Agradecimientos a las librerías y proyectos que hacen este trabajo posible: GTK4, libadwaita, tokio, serde, etc.

---

