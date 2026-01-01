#!/bin/bash
# setup-devenv.sh - Configura el entorno de desarrollo para CopyMaster

set -e

echo "🔧 Configurando entorno de desarrollo para CopyMaster..."

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

# Instalar Rust si no está presente
install_rust() {
    if ! command -v rustc &> /dev/null; then
        print_info "Instalando Rust..."
        
        # Instalar rustup
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Configurar PATH
        source "$HOME/.cargo/env"
        
        print_success "Rust instalado"
    else
        print_info "Rust ya está instalado: $(rustc --version)"
    fi
}

# Instalar dependencias del sistema
install_system_deps() {
    print_info "Instalando dependencias del sistema..."
    
    # Detectar distribución
    if command -v apt &> /dev/null; then
        # Debian/Ubuntu
        sudo apt update
        sudo apt install -y \
            build-essential \
            libgtk-4-dev \
            libadwaita-1-dev \
            libgdk-pixbuf-2.0-dev \
            pkg-config \
            libssl-dev
    elif command -v dnf &> /dev/null; then
        # Fedora/RHEL
        sudo dnf install -y \
            gcc-c++ \
            gtk4-devel \
            libadwaita-devel \
            gdk-pixbuf2-devel \
            pkgconfig \
            openssl-devel
    elif command -v pacman &> /dev/null; then
        # Arch/Manjaro
        sudo pacman -Syu --noconfirm \
            base-devel \
            gtk4 \
            libadwaita \
            gdk-pixbuf2 \
            pkg-config \
            openssl
    else
        print_warning "No se pudo detectar el gestor de paquetes"
        print_info "Instala manualmente:"
        echo "  • build-essential / base-devel"
        echo "  • libgtk-4-dev / gtk4-devel"
        echo "  • libadwaita-1-dev / libadwaita-devel"
        echo "  • pkg-config"
        return 1
    fi
    
    print_success "Dependencias del sistema instaladas"
}

# Configurar Git hooks
setup_git_hooks() {
    print_info "Configurando Git hooks..."
    
    if [ -d "$PROJECT_ROOT/.git" ]; then
        # Crear directorio de hooks si no existe
        mkdir -p "$PROJECT_ROOT/.git/hooks"
        
        # Crear pre-commit hook
        cat > "$PROJECT_ROOT/.git/hooks/pre-commit" << 'EOF'
#!/bin/bash
# Pre-commit hook para CopyMaster

echo "🔍 Ejecutando checks pre-commit..."

# Ejecutar cargo fmt
echo "Formateando código..."
cargo fmt

# Ejecutar cargo clippy
echo "Ejecutando clippy..."
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy encontró problemas"
    exit 1
fi

# Ejecutar tests básicos
echo "Ejecutando tests..."
if ! cargo test --lib; then
    echo "❌ Tests fallaron"
    exit 1
fi

echo "✅ Pre-commit checks pasados"
EOF
        
        chmod +x "$PROJECT_ROOT/.git/hooks/pre-commit"
        
        print_success "Git hooks configurados"
    else
        print_warning "No es un repositorio Git, omitiendo hooks"
    fi
}

# Configurar entorno Rust
setup_rust_env() {
    print_info "Configurando entorno Rust..."
    
    # Instalar herramientas Rust
    rustup component add rustfmt
    rustup component add clippy
    
    # Configurar rust-analyzer si está en VSCode
    if command -v code &> /dev/null; then
        print_info "Instalando extensión rust-analyzer para VSCode..."
        code --install-extension rust-lang.rust-analyzer
    fi
    
    print_success "Entorno Rust configurado"
}

# Generar estructura inicial
generate_initial_structure() {
    print_info "Generando estructura inicial..."
    
    # Crear directorios necesarios
    mkdir -p "$PROJECT_ROOT/data/icons"
    mkdir -p "$PROJECT_ROOT/assets"
    mkdir -p "$PROJECT_ROOT/docs"
    
    # Crear archivo .gitignore si no existe
    if [ ! -f "$PROJECT_ROOT/.gitignore" ]; then
        cat > "$PROJECT_ROOT/.gitignore" << 'EOF'
# Rust
/target/
**/*.rs.bk

# OS
.DS_Store
Thumbs.db

# IDE
.vscode/
.idea/
*.swp
*.swo

# Build
/build/
/dist/
/release/

# Configuración local
.env
*.local

# Logs
*.log
EOF
    fi
    
    # Crear archivo de configuración de ejemplo
    if [ ! -f "$PROJECT_ROOT/config.example.json" ]; then
        cat > "$PROJECT_ROOT/config.example.json" << 'EOF'
{
  "autostart_enabled": false,
  "start_minimized": false,
  "minimize_to_tray": true,
  "show_notifications": true,
  "remembered_devices": []
}
EOF
    fi
    
    print_success "Estructura inicial generada"
}

# Verificar configuración
verify_setup() {
    print_info "Verificando configuración..."
    
    local errors=0
    
    # Verificar Rust
    if ! command -v rustc &> /dev/null; then
        print_error "Rust no está instalado"
        errors=$((errors + 1))
    fi
    
    # Verificar cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo no está instalado"
        errors=$((errors + 1))
    fi
    
    # Verificar pkg-config
    if ! command -v pkg-config &> /dev/null; then
        print_error "pkg-config no está instalado"
        errors=$((errors + 1))
    fi
    
    # Verificar GTK4
    if ! pkg-config --exists gtk4; then
        print_error "GTK4 no está instalado"
        errors=$((errors + 1))
    fi
    
    # Verificar libadwaita
    if ! pkg-config --exists libadwaita-1; then
        print_error "libadwaita no está instalado"
        errors=$((errors + 1))
    fi
    
    if [ $errors -eq 0 ]; then
        print_success "✅ Configuración verificada correctamente"
        return 0
    else
        print_error "❌ Se encontraron $errors errores"
        return 1
    fi
}

# Probar compilación
test_build() {
    print_info "Probando compilación..."
    
    cd "$PROJECT_ROOT"
    
    if cargo build; then
        print_success "✅ Compilación exitosa"
        return 0
    else
        print_error "❌ Error en la compilación"
        return 1
    fi
}

# Función principal
main() {
    print_success "=== Configuración de entorno de desarrollo ==="
    
    # 1. Instalar Rust
    install_rust
    
    # 2. Instalar dependencias del sistema
    install_system_deps
    
    # 3. Configurar entorno Rust
    setup_rust_env
    
    # 4. Generar estructura inicial
    generate_initial_structure
    
    # 5. Configurar Git hooks
    setup_git_hooks
    
    # 6. Verificar configuración
    if verify_setup; then
        # 7. Probar compilación
        if test_build; then
            print_success "🎉 ¡Entorno de desarrollo configurado exitosamente!"
            echo ""
            echo "📋 Resumen:"
            echo "  • Rust: $(rustc --version | cut -d' ' -f1-2)"
            echo "  • Cargo: $(cargo --version | cut -d' ' -f1-2)"
            echo "  • GTK4: $(pkg-config --modversion gtk4)"
            echo "  • libadwaita: $(pkg-config --modversion libadwaita-1)"
            echo ""
            echo "🚀 Comandos útiles:"
            echo "  cargo build     # Compilar"
            echo "  cargo run       # Ejecutar"
            echo "  cargo test      # Ejecutar tests"
            echo "  cargo fmt       # Formatear código"
            echo "  cargo clippy    # Análisis de código"
            echo ""
            echo "📚 Recursos:"
            echo "  • GTK4 Rust bindings: https://gtk-rs.org/"
            echo "  • libadwaita docs: https://gnome.pages.gitlab.gnome.org/libadwaita/"
            echo "  • Rust book: https://doc.rust-lang.org/book/"
        fi
    else
        print_error "La verificación falló. Revisa los errores arriba."
        exit 1
    fi
}

# Ejecutar función principal
main