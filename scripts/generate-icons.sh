#!/bin/bash
# generate-icons.sh - Genera iconos en múltiples tamaños y formatos

set -e

echo "🖼️  Generando iconos para CopyMaster..."

# Configuración
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ICON_DIR="$PROJECT_ROOT/data/icons"
ASSETS_DIR="$PROJECT_ROOT/assets"
BUILD_DIR="$PROJECT_ROOT/build/icons"

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Funciones de utilidad
print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Crear directorios
create_directories() {
    print_info "Creando estructura de directorios..."
    
    mkdir -p "$ICON_DIR/scalable/apps"
    mkdir -p "$ICON_DIR/scalable/status"
    mkdir -p "$ICON_DIR/scalable/devices"
    mkdir -p "$ICON_DIR/scalable/actions"
    
    for size in 16 22 24 32 48 64 96 128 256 512; do
        mkdir -p "$ICON_DIR/${size}x${size}/apps"
        mkdir -p "$ICON_DIR/${size}x${size}/status"
        mkdir -p "$ICON_DIR/${size}x${size}/devices"
        mkdir -p "$ICON_DIR/${size}x${size}/actions"
    done
    
    mkdir -p "$BUILD_DIR"
}

# Verificar dependencias
check_dependencies() {
    print_info "Verificando dependencias..."
    
    local missing_deps=()
    
    # Verificar Inkscape (para SVG)
    if ! command -v inkscape &> /dev/null; then
        print_warning "Inkscape no encontrado. Se necesitará para generar desde SVG."
        missing_deps+=("inkscape")
    fi
    
    # Verificar ImageMagick (para conversiones)
    if ! command -v convert &> /dev/null; then
        print_error "ImageMagick (convert) no encontrado. Es requerido."
        exit 1
    fi
    
    # Verificar rsvg-convert (alternativa para SVG)
    if ! command -v rsvg-convert &> /dev/null; then
        print_warning "librsvg (rsvg-convert) no encontrado."
    fi
    
    # Verificar optipng (para optimizar PNG)
    if command -v optipng &> /dev/null; then
        OPTIPNG_AVAILABLE=true
    else
        print_warning "optipng no encontrado. Los PNG no serán optimizados."
        OPTIPNG_AVAILABLE=false
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        print_warning "Dependencias faltantes: ${missing_deps[*]}"
        print_info "Instala con: sudo apt install ${missing_deps[*]}"
    fi
}

# Crear SVG de ejemplo si no existe
create_sample_svg() {
    print_info "Creando SVG de ejemplo..."
    
    cat > "$BUILD_DIR/copymaster-sample.svg" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg width="256" height="256" viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
    <!-- Fondo circular -->
    <circle cx="128" cy="128" r="120" fill="#4a86e8" opacity="0.1"/>
    
    <!-- Disquete (símbolo de guardar) -->
    <rect x="70" y="70" width="116" height="116" rx="8" fill="#4a86e8"/>
    <rect x="80" y="80" width="96" height="96" rx="4" fill="#2d5aa0"/>
    <rect x="90" y="90" width="76" height="26" rx="4" fill="#4a86e8"/>
    
    <!-- Flecha de copia -->
    <polygon points="128,130 160,170 96,170" fill="white"/>
    
    <!-- Aro exterior -->
    <circle cx="128" cy="128" r="120" fill="none" stroke="#4a86e8" stroke-width="4"/>
</svg>
EOF
}

# Generar icono principal desde SVG
generate_main_icons() {
    local svg_source="$ASSETS_DIR/copymaster.svg"
    
    if [ ! -f "$svg_source" ]; then
        print_warning "Archivo SVG fuente no encontrado: $svg_source"
        print_info "Usando SVG de ejemplo..."
        create_sample_svg
        svg_source="$BUILD_DIR/copymaster-sample.svg"
    fi
    
    print_info "Generando iconos principales desde: $(basename "$svg_source")"
    
    # Generar en diferentes tamaños
    local sizes=(16 22 24 32 48 64 96 128 256 512)
    
    for size in "${sizes[@]}"; do
        local output_png="$ICON_DIR/${size}x${size}/apps/copymaster.png"
        
        if command -v inkscape &> /dev/null; then
            inkscape -w "$size" -h "$size" "$svg_source" -o "$output_png"
        elif command -v rsvg-convert &> /dev/null; then
            rsvg-convert -w "$size" -h "$size" "$svg_source" -o "$output_png"
        else
            convert -background none -resize "${size}x${size}" "$svg_source" "$output_png"
        fi
        
        # Optimizar PNG si está disponible
        if [ "$OPTIPNG_AVAILABLE" = true ]; then
            optipng -quiet -o2 "$output_png"
        fi
        
        print_info "  ✓ Generado ${size}x${size}"
    done
    
    # Copiar SVG escalable
    cp "$svg_source" "$ICON_DIR/scalable/apps/copymaster.svg"
}

# Crear iconos de estado
generate_state_icons() {
    print_info "Generando iconos de estado..."
    
    local states=("active" "paused" "error" "warning")
    local state_colors=("#4CAF50" "#FF9800" "#F44336" "#FFC107")
    
    for i in "${!states[@]}"; do
        local state="${states[$i]}"
        local color="${state_colors[$i]}"
        
        print_info "  Generando estado: $state ($color)"
        
        # Crear variante de color
        for size in 16 22 24 32 48 64 96 128 256; do
            local source_png="$ICON_DIR/${size}x${size}/apps/copymaster.png"
            local output_png="$ICON_DIR/${size}x${size}/status/copymaster-${state}.png"
            
            if [ -f "$source_png" ]; then
                # Cambiar color azul por el color del estado
                convert "$source_png" \
                    -channel RGB \
                    -fuzz 20% \
                    -fill "$color" \
                    -opaque "#4a86e8" \
                    "$output_png"
                
                # Optimizar
                if [ "$OPTIPNG_AVAILABLE" = true ]; then
                    optipng -quiet -o2 "$output_png"
                fi
            fi
        done
    done
}

# Generar iconos simbólicos (monocromáticos)
generate_symbolic_icons() {
    print_info "Generando iconos simbólicos..."
    
    # Convertir iconos principales a escala de grises
    for size in 16 22 24 32; do
        local source="$ICON_DIR/${size}x${size}/apps/copymaster.png"
        local symbolic="$ICON_DIR/${size}x${size}/apps/copymaster-symbolic.png"
        
        if [ -f "$source" ]; then
            convert "$source" \
                -colorspace Gray \
                -threshold 60% \
                -negate \
                "$symbolic"
        fi
    done
    
    # Crear también SVG simbólico
    if [ -f "$ICON_DIR/scalable/apps/copymaster.svg" ]; then
        cp "$ICON_DIR/scalable/apps/copymaster.svg" \
           "$ICON_DIR/scalable/apps/copymaster-symbolic.svg"
    fi
}

# Función principal
main() {
    print_success "Iniciando generación de iconos..."
    
    # Verificar dependencias
    check_dependencies
    
    # Crear directorios
    create_directories
    
    # Generar iconos principales
    generate_main_icons
    
    # Generar iconos de estado
    generate_state_icons
    
    # Generar iconos simbólicos
    generate_symbolic_icons
    
    print_success "✅ Iconos generados exitosamente!"
    print_info "Los iconos están en: $ICON_DIR"
    
    # Mostrar resumen
    echo ""
    echo "📁 Estructura generada:"
    find "$ICON_DIR" -type f -name "*.png" -o -name "*.svg" | sort | head -20
    echo "..."
    echo "Total de archivos: $(find "$ICON_DIR" -type f | wc -l)"
}

# Ejecutar función principal
main