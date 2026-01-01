#!/bin/bash
# build-appimage.sh - Construye una AppImage de CopyMaster

set -e

echo "📦 Construyendo AppImage para CopyMaster..."

# Configuración
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build/appimage"
APPIMAGE_DIR="$BUILD_DIR/AppDir"
OUTPUT_DIR="$PROJECT_ROOT/dist"

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Funciones
print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Verificar dependencias
check_dependencies() {
    print_info "Verificando dependencias..."
    
    local missing_deps=()
    
    # Verificar linuxdeploy
    if ! command -v linuxdeploy &> /dev/null; then
        if [ ! -f "linuxdeploy-x86_64.AppImage" ]; then
            missing_deps+=("linuxdeploy")
        fi
    fi
    
    # Verificar appimagetool
    if ! command -v appimagetool &> /dev/null; then
        if [ ! -f "appimagetool-x86_64.AppImage" ]; then
            missing_deps+=("appimagetool")
        fi
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        print_warning "Dependencias faltantes: ${missing_deps[*]}"
        
        # Descargar linuxdeploy si no existe
        if [[ " ${missing_deps[*]} " =~ " linuxdeploy " ]]; then
            print_info "Descargando linuxdeploy..."
            wget -q "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
            chmod +x linuxdeploy-x86_64.AppImage
            export LINUXDEPLOY=$(pwd)/linuxdeploy-x86_64.AppImage
        fi
        
        # Descargar appimagetool si no existe
        if [[ " ${missing_deps[*]} " =~ " appimagetool " ]]; then
            print_info "Descargando appimagetool..."
            wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
            chmod +x appimagetool-x86_64.AppImage
            export APPIMAGETOOL=$(pwd)/appimagetool-x86_64.AppImage
        fi
    fi
    
    # Configurar variables si no están definidas
    if [ -z "$LINUXDEPLOY" ] && [ -f "linuxdeploy-x86_64.AppImage" ]; then
        export LINUXDEPLOY=$(pwd)/linuxdeploy-x86_64.AppImage
    fi
    
    if [ -z "$APPIMAGETOOL" ] && [ -f "appimagetool-x86_64.AppImage" ]; then
        export APPIMAGETOOL=$(pwd)/appimagetool-x86_64.AppImage
    fi
    
    if [ -z "$LINUXDEPLOY" ] || [ -z "$APPIMAGETOOL" ]; then
        print_error "No se encontraron las herramientas necesarias"
        exit 1
    fi
}

# Preparar directorio de construcción
prepare_build_dir() {
    print_info "Preparando directorio de construcción..."
    
    # Limpiar build anterior
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"
    mkdir -p "$APPIMAGE_DIR"
    mkdir -p "$OUTPUT_DIR"
    
    # Cambiar al directorio de construcción
    cd "$BUILD_DIR"
}

# Compilar proyecto
build_project() {
    print_info "Compilando proyecto..."
    
    cd "$PROJECT_ROOT"
    
    # Compilar en modo release
    if ! cargo build --release; then
        print_error "Error al compilar"
        exit 1
    fi
    
    # Copiar binario
    cp "target/release/copymaster" "$APPIMAGE_DIR/"
    
    print_success "Proyecto compilado"
}

# Crear estructura AppDir
create_appdir() {
    print_info "Creando estructura AppDir..."
    
    # Crear directorios estándar de AppImage
    mkdir -p "$APPIMAGE_DIR/usr/bin"
    mkdir -p "$APPIMAGE_DIR/usr/share/applications"
    mkdir -p "$APPIMAGE_DIR/usr/share/icons/hicolor"
    mkdir -p "$APPIMAGE_DIR/usr/share/metainfo"
    
    # Mover binario
    mv "$APPIMAGE_DIR/copymaster" "$APPIMAGE_DIR/usr/bin/"
    
    # Crear archivo .desktop
    cat > "$APPIMAGE_DIR/usr/share/applications/copymaster.desktop" << EOF
[Desktop Entry]
Type=Application
Name=CopyMaster
Comment=Advanced copy manager for Linux
Exec=copymaster
Icon=copymaster
Terminal=false
Categories=Utility;
EOF
    
    # Crear archivo metainfo
    cat > "$APPIMAGE_DIR/usr/share/metainfo/copymaster.appdata.xml" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop">
  <id>org.copymaster</id>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>GPL-3.0</project_license>
  <name>CopyMaster</name>
  <summary>Advanced copy manager for Linux</summary>
  <description>
    <p>CopyMaster is an advanced copy manager for Linux, inspired by tools like Teracopy and Supercopier.</p>
  </description>
  <url type="homepage">https://github.com/tuusuario/copymaster</url>
</component>
EOF
    
    # Copiar iconos
    if [ -d "$PROJECT_ROOT/data/icons" ]; then
        cp -r "$PROJECT_ROOT/data/icons" "$APPIMAGE_DIR/usr/share/icons/hicolor/"
    fi
    
    # Crear AppRun
    cat > "$APPIMAGE_DIR/AppRun" << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin/:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib/:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS}"
exec "${HERE}/usr/bin/copymaster" "$@"
EOF
    
    chmod +x "$APPIMAGE_DIR/AppRun"
}

# Ejecutar linuxdeploy
run_linuxdeploy() {
    print_info "Ejecutando linuxdeploy..."
    
    cd "$BUILD_DIR"
    
    # Configurar variables de entorno
    export ARCH=x86_64
    export VERSION=0.1.0
    
    # Ejecutar linuxdeploy
    "$LINUXDEPLOY" \
        --appdir "$APPIMAGE_DIR" \
        --desktop-file "$APPIMAGE_DIR/usr/share/applications/copymaster.desktop" \
        --icon-file "$APPIMAGE_DIR/usr/share/icons/hicolor/256x256/apps/copymaster.png" \
        --output appimage
    
    if [ $? -ne 0 ]; then
        print_warning "linuxdeploy encontró problemas, continuando de todos modos..."
    fi
}

# Crear AppImage
create_appimage() {
    print_info "Creando AppImage..."
    
    cd "$BUILD_DIR"
    
    # Crear AppImage con appimagetool
    "$APPIMAGETOOL" "$APPIMAGE_DIR"
    
    # Mover AppImage al directorio de salida
    mv CopyMaster*.AppImage "$OUTPUT_DIR/copymaster-x86_64.AppImage"
    
    print_success "AppImage creada: $OUTPUT_DIR/copymaster-x86_64.AppImage"
}

# Función principal
main() {
    print_success "=== Construyendo AppImage de CopyMaster ==="
    
    # Verificar que estamos en el directorio correcto
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        print_error "No se encontró Cargo.toml. Asegúrate de estar en el directorio del proyecto."
        exit 1
    fi
    
    # 1. Verificar dependencias
    check_dependencies
    
    # 2. Preparar directorio de construcción
    prepare_build_dir
    
    # 3. Compilar proyecto
    build_project
    
    # 4. Crear estructura AppDir
    create_appdir
    
    # 5. Ejecutar linuxdeploy
    run_linuxdeploy
    
    # 6. Crear AppImage
    create_appimage
    
    # Resumen
    echo ""
    print_success "✅ AppImage construida exitosamente!"
    echo ""
    echo "📋 Archivos generados:"
    echo "  • $OUTPUT_DIR/copymaster-x86_64.AppImage"
    echo ""
    echo "🎯 Para usar la AppImage:"
    echo "  chmod +x copymaster-x86_64.AppImage"
    echo "  ./copymaster-x86_64.AppImage"
    echo ""
    echo "📦 La AppImage es portable y no necesita instalación."
}

# Ejecutar función principal
main