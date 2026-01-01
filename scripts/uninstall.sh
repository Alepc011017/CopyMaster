#!/bin/bash
# uninstall.sh - Desinstala CopyMaster del sistema

set -e

echo "🗑️  Desinstalando CopyMaster..."

# Configuración
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Funciones
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Verificar si está instalado
check_installed() {
    if [ ! -f "/usr/local/bin/copymaster" ]; then
        print_warning "CopyMaster no parece estar instalado en /usr/local/bin/"
        return 1
    fi
    return 0
}

# Desinstalar usando make
uninstall_with_make() {
    cd "$PROJECT_ROOT"
    
    if [ -f "Makefile" ]; then
        print_success "Usando Makefile para desinstalar..."
        
        if sudo make uninstall; then
            return 0
        else
            print_error "Error al desinstalar con make"
            return 1
        fi
    else
        print_warning "Makefile no encontrado"
        return 1
    fi
}

# Desinstalar manualmente
uninstall_manually() {
    print_success "Desinstalando manualmente..."
    
    # Remover binario
    if [ -f "/usr/local/bin/copymaster" ]; then
        sudo rm -f "/usr/local/bin/copymaster"
        print_success "  Binario removido"
    fi
    
    # Remover iconos
    local icon_dirs=(
        "/usr/local/share/icons/hicolor/scalable/apps/copymaster.svg"
        "/usr/local/share/icons/hicolor/256x256/apps/copymaster.png"
        "/usr/local/share/icons/hicolor/128x128/apps/copymaster.png"
        "/usr/local/share/icons/hicolor/64x64/apps/copymaster.png"
        "/usr/local/share/icons/hicolor/32x32/apps/copymaster.png"
    )
    
    for icon in "${icon_dirs[@]}"; do
        if [ -f "$icon" ]; then
            sudo rm -f "$icon"
        fi
    done
    
    # Actualizar caché de iconos
    if [ -d "/usr/local/share/icons/hicolor" ]; then
        sudo gtk-update-icon-cache -q -t -f "/usr/local/share/icons/hicolor" 2>/dev/null || true
    fi
    
    # Remover archivo .desktop
    if [ -f "/usr/local/share/applications/copymaster.desktop" ]; then
        sudo rm -f "/usr/local/share/applications/copymaster.desktop"
        print_success "  Entrada de menú removida"
    fi
    
    # Actualizar base de datos de aplicaciones
    if [ -d "/usr/local/share/applications" ]; then
        sudo update-desktop-database "/usr/local/share/applications" 2>/dev/null || true
    fi
    
    # Remover auto-arranque
    local autostart_file="$HOME/.config/autostart/copymaster.desktop"
    if [ -f "$autostart_file" ]; then
        rm -f "$autostart_file"
        print_success "  Auto-arranque removido"
    fi
    
    # Remover configuración
    local config_dir="$HOME/.config/copymaster"
    if [ -d "$config_dir" ]; then
        read -p "¿Eliminar configuración en $config_dir? (s/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Ss]$ ]]; then
            rm -rf "$config_dir"
            print_success "  Configuración removida"
        fi
    fi
}

# Función principal
main() {
    echo "=== Desinstalación de CopyMaster ==="
    echo ""
    
    # Preguntar confirmación
    read -p "¿Estás seguro de que quieres desinstalar CopyMaster? (s/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Ss]$ ]]; then
        echo "Desinstalación cancelada"
        exit 0
    fi
    
    # Verificar si está instalado
    if ! check_installed; then
        print_warning "CopyMaster no está instalado o no se encontró en /usr/local/bin/"
        read -p "¿Continuar con la desinstalación de todos modos? (s/n): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Ss]$ ]]; then
            exit 0
        fi
    fi
    
    # Intentar desinstalar con make primero
    if ! uninstall_with_make; then
        print_warning "Falló la desinstalación con make, intentando manualmente..."
        uninstall_manually
    fi
    
    print_success "✅ CopyMaster desinstalado exitosamente"
    echo ""
    echo "Los siguientes elementos fueron removidos:"
    echo "  • Binario: /usr/local/bin/copymaster"
    echo "  • Iconos del sistema"
    echo "  • Entrada del menú de aplicaciones"
    echo "  • Configuración de auto-arranque"
    echo ""
    echo "Nota: Los archivos de configuración en ~/.config/copymaster"
    echo "      pueden necesitar ser removidos manualmente."
}

# Ejecutar función principal
main