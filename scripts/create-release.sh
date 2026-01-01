#!/bin/bash
# create-release.sh - Crea una release completa de CopyMaster

set -e

echo "🚀 Creando release de CopyMaster..."

# Configuración
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RELEASE_DIR="$PROJECT_ROOT/release"
VERSION=$(grep -m1 version "$PROJECT_ROOT/Cargo.toml" | cut -d'"' -f2)

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

# Crear directorio de release
create_release_dir() {
    print_info "Creando directorio de release..."
    
    rm -rf "$RELEASE_DIR"
    mkdir -p "$RELEASE_DIR"
    
    # Crear subdirectorios
    mkdir -p "$RELEASE_DIR/bin"
    mkdir -p "$RELEASE_DIR/data"
    mkdir -p "$RELEASE_DIR/docs"
    mkdir -p "$RELEASE_DIR/scripts"
}

# Construir binarios
build_binaries() {
    print_info "Construyendo binarios..."
    
    cd "$PROJECT_ROOT"
    
    # Construir en modo release
    cargo build --release
    
    # Copiar binario
    cp "target/release/copymaster" "$RELEASE_DIR/bin/"
    
    # Hacer ejecutable
    chmod +x "$RELEASE_DIR/bin/copymaster"
}

# Copiar datos
copy_data() {
    print_info "Copiando archivos de datos..."
    
    # Copiar estructura data/
    if [ -d "$PROJECT_ROOT/data" ]; then
        cp -r "$PROJECT_ROOT/data" "$RELEASE_DIR/"
    fi
    
    # Copiar documentación
    if [ -f "$PROJECT_ROOT/README.md" ]; then
        cp "$PROJECT_ROOT/README.md" "$RELEASE_DIR/docs/"
    fi
    
    if [ -f "$PROJECT_ROOT/LICENSE" ]; then
        cp "$PROJECT_ROOT/LICENSE" "$RELEASE_DIR/docs/"
    fi
    
    if [ -f "$PROJECT_ROOT/CHANGELOG.md" ]; then
        cp "$PROJECT_ROOT/CHANGELOG.md" "$RELEASE_DIR/docs/"
    fi
}

# Copiar scripts
copy_scripts() {
    print_info "Copiando scripts..."
    
    # Copiar scripts de instalación
    if [ -d "$PROJECT_ROOT/scripts" ]; then
        cp -r "$PROJECT_ROOT/scripts" "$RELEASE_DIR/"
        
        # Hacer ejecutables los scripts
        chmod +x "$RELEASE_DIR/scripts/"*.sh
    fi
}

# Crear paquetes
create_packages() {
    print_info "Creando paquetes..."
    
    cd "$RELEASE_DIR"
    
    # 1. Crear tarball
    print_info "Creando tarball..."
    tar -czf "../copymaster-$VERSION.tar.gz" .
    
    # 2. Crear zip
    print_info "Creando zip..."
    zip -r "../copymaster-$VERSION.zip" .
    
    # 3. Intentar crear AppImage si existe el script
    if [ -f "$PROJECT_ROOT/scripts/build-appimage.sh" ]; then
        print_info "Creando AppImage..."
        "$PROJECT_ROOT/scripts/build-appimage.sh"
        
        # Copiar AppImage a release
        if [ -f "$PROJECT_ROOT/dist/copymaster-x86_64.AppImage" ]; then
            cp "$PROJECT_ROOT/dist/copymaster-x86_64.AppImage" \
               "../copymaster-$VERSION-x86_64.AppImage"
        fi
    fi
}

# Generar checksums
generate_checksums() {
    print_info "Generando checksums..."
    
    cd "$PROJECT_ROOT"
    
    # Generar SHA256 checksums
    for file in copymaster-*; do
        if [ -f "$file" ]; then
            sha256sum "$file" > "$file.sha256"
        fi
    done
}

# Crear archivo de información de release
create_release_info() {
    print_info "Creando información de release..."
    
    cat > "$RELEASE_DIR/RELEASE.md" << EOF
# CopyMaster $VERSION

## Archivos de release

1. **copymaster-$VERSION.tar.gz** - Tarball con todos los archivos
2. **copymaster-$VERSION.zip** - Archivo ZIP
3. **copymaster-$VERSION-x86_64.AppImage** (si está disponible) - AppImage portable

## Instalación

### Desde tarball/ZIP:

\`\`\`bash
# Extraer
tar -xzf copymaster-$VERSION.tar.gz
# o
unzip copymaster-$VERSION.zip

# Instalar
cd copymaster-$VERSION
sudo ./scripts/install.sh
\`\`\`

### Desde AppImage:

\`\`\`bash
# Hacer ejecutable
chmod +x copymaster-$VERSION-x86_64.AppImage

# Ejecutar
./copymaster-$VERSION-x86_64.AppImage
\`\`\`

## Verificación

Verifica la integridad de los archivos con:

\`\`\`bash
sha256sum -c copymaster-$VERSION.tar.gz.sha256
\`\`\`

## Cambios en esta versión

*(Consultar CHANGELOG.md para detalles)*

## Soporte

- GitHub: https://github.com/tuusuario/copymaster
- Issues: https://github.com/tuusuario/copymaster/issues
EOF
}

# Función principal
main() {
    print_success "=== Creando release de CopyMaster v$VERSION ==="
    
    # Verificar que estamos en el directorio correcto
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        print_error "No se encontró Cargo.toml. Asegúrate de estar en el directorio del proyecto."
        exit 1
    fi
    
    # 1. Crear directorio de release
    create_release_dir
    
    # 2. Construir binarios
    build_binaries
    
    # 3. Copiar datos
    copy_data
    
    # 4. Copiar scripts
    copy_scripts
    
    # 5. Crear paquetes
    create_packages
    
    # 6. Generar checksums
    generate_checksums
    
    # 7. Crear información de release
    create_release_info
    
    # Resumen
    echo ""
    print_success "✅ Release creada exitosamente!"
    echo ""
    echo "📋 Archivos generados en $PROJECT_ROOT/:"
    echo ""
    
    # Listar archivos de release
    for file in "$PROJECT_ROOT"/copymaster-*; do
        if [ -f "$file" ]; then
            local size=$(du -h "$file" | cut -f1)
            echo "  • $(basename "$file") ($size)"
        fi
    done
    
    echo ""
    echo "🎯 Para publicar la release:"
    echo "  1. Crear un tag en Git: git tag v$VERSION"
    echo "  2. Subir el tag: git push origin v$VERSION"
    echo "  3. Crear una release en GitHub/GitLab"
    echo "  4. Subir los archivos generados"
    echo ""
    echo "📦 Los usuarios pueden instalar desde:"
    echo "  • Tarball/ZIP: scripts/install.sh"
    echo "  • AppImage: Ejecutable directamente"
}

# Ejecutar función principal
main