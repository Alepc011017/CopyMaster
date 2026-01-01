#!/bin/bash
# install.sh - Script de instalación para CopyMaster

set -e

echo "🚀 Instalando CopyMaster..."

# Configuración
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

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
    
    # Verificar Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust no está instalado"
        echo "Instala Rust con:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Verificar make
    if ! command -v make &> /dev/null; then
        print_error "make no está instalado"
        echo "Instala make con:"
        echo "  sudo apt install make"
        exit 1
    fi
    
    # Verificar dependencias de desarrollo
    local missing_deps=()
    
    # GTK4
    if ! pkg-config --exists gtk4; then
        missing_deps+=("libgtk-4-dev")
    fi
    
    # Libadwaita
    if ! pkg-config --exists libadwaita-1; then
        missing_deps+=("libadwaita-1-dev")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        print_warning "Dependencias de desarrollo faltantes: ${missing_deps[*]}"
        print_info "Instala con: sudo apt install ${missing_deps[*]}"
        
        # Preguntar si instalar
        read -p "¿Instalar dependencias? (s/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Ss]$ ]]; then
            sudo apt update
            sudo apt install -y "${missing_deps[@]}"
        else
            print_error "Instalación cancelada"
            exit 1
        fi
    fi
}

# Generar iconos
generate_icons() {
    print_info "Generando iconos..."
    
    if [ -f "$SCRIPT_DIR/generate-icons.sh" ]; then
        chmod +x "$SCRIPT_DIR/generate-icons.sh"
        "$SCRIPT_DIR/generate-icons.sh"
    else
        print_warning "Script generate-icons.sh no encontrado"
        print_info "Creando iconos básicos..."
        
        mkdir -p "$PROJECT_ROOT/data/icons/64x64/apps"
        mkdir -p "$PROJECT_ROOT/data/icons/32x32/apps"
        
        # Crear icono simple
        convert -size 64x64 xc:transparent \
            -fill "#4a86e8" -draw "rectangle 10,10 54,54" \
            -fill "#2d5aa0" -draw "rectangle 15,15 49,49" \
            -fill "white" -draw "polygon 32,25 45,38 19,38" \
            "$PROJECT_ROOT/data/icons/64x64/apps/copymaster.png"
        
        convert -resize 32x32 \
            "$PROJECT_ROOT/data/icons/64x64/apps/copymaster.png" \
            "$PROJECT_ROOT/data/icons/32x32/apps/copymaster.png"
    fi
}

# Compilar
build_project() {
    print_info "Compilando proyecto..."
    
    cd "$PROJECT_ROOT"
    
    # Limpiar build anterior
    cargo clean
    
    # Compilar en modo release
    if ! cargo build --release; then
        print_error "Error al compilar"
        exit 1
    fi
    
    print_success "Compilación completada"
}

# Instalar
install_system() {
    print_info "Instalando en el sistema..."
    
    cd "$PROJECT_ROOT"
    
    if ! sudo make install; then
        print_error "Error en la instalación"
        exit 1
    fi
    
    print_success "Instalación completada"
}

# Configurar auto-arranque
configure_autostart() {
    print_info "Configurando auto-arranque..."
    
    if [ -f "$PROJECT_ROOT/target/release/copymaster" ]; then
        "$PROJECT_ROOT/target/release/copymaster" autostart --enable --minimized
        print_success "Auto-arranque configurado"
    else
        print_warning "No se pudo configurar auto-arranque: binario no encontrado"
    fi
}

# Función principal
main() {
    print_success "=== Instalación de CopyMaster ==="
    
    # Verificar que estamos en el directorio correcto
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        print_error "No se encontró Cargo.toml. Asegúrate de estar en el directorio del proyecto."
        exit 1
    fi
    
    # 1. Verificar dependencias
    check_dependencies
    
    # 2. Generar iconos
    generate_icons
    
    # 3. Compilar
    build_project
    
    # 4. Instalar
    install_system
    
    # 5. Configurar auto-arranque
    configure_autostart
    
    # Resumen
    echo ""
    print_success "✅ ¡CopyMaster instalado exitosamente!"
    echo ""
    echo "📋 Resumen de la instalación:"
    echo "  • Binario: /usr/local/bin/copymaster"
    echo "  • Iconos: /usr/local/share/icons/hicolor/"
    echo "  • Entrada de menú: /usr/local/share/applications/copymaster.desktop"
    echo "  • Auto-arranque: ~/.config/autostart/copymaster.desktop"
    echo ""
    echo "🎯 Para usar CopyMaster:"
    echo "  • Ejecutar desde terminal: copymaster"
    echo "  • Buscar 'CopyMaster' en el menú de aplicaciones"
    echo "  • Se iniciará automáticamente al encender la PC"
    echo ""
    echo "🛠️  Comandos útiles:"
    echo "  copymaster --help           # Ver todas las opciones"
    echo "  copymaster --minimized      # Iniciar minimizado"
    echo "  copymaster --daemon         # Ejecutar como servicio"
    echo "  copymaster autostart --help # Configurar auto-arranque"
    echo ""
    echo "📚 Documentación:"
    echo "  Visita el repositorio para más información"
}

# Ejecutar función principal
main