#!/bin/bash
# configure-conflicts.sh - Configura el manejo de conflictos desde CLI

set -e

echo "⚙️  Configurando manejo de conflictos para CopyMaster..."

# Configuración
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

show_help() {
    echo "Uso: $0 [OPCIONES]"
    echo ""
    echo "Configura el manejo de conflictos de archivos para CopyMaster"
    echo ""
    echo "Opciones:"
    echo "  --ask           Preguntar siempre (por defecto)"
    echo "  --overwrite     Sobrescribir siempre"
    echo "  --skip          Saltar siempre"
    echo "  --rename-new    Renombrar siempre el nuevo archivo"
    echo "  --rename-old    Renombrar siempre el archivo existente"
    echo "  --no-confirm    No pedir confirmación"
    echo "  --help          Mostrar esta ayuda"
    echo ""
    echo "Ejemplos:"
    echo "  $0 --ask                 # Preguntar siempre"
    echo "  $0 --overwrite --no-confirm  # Sobrescribir sin preguntar"
    echo ""
    echo "Nota: Los cambios se aplican a la configuración global"
}

if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    show_help
    exit 0
fi

# Valores por defecto
ACTION="ask"
CONFIRM="true"

# Parsear argumentos
while [[ $# -gt 0 ]]; do
    case $1 in
        --ask)
            ACTION="ask"
            shift
            ;;
        --overwrite)
            ACTION="overwrite"
            shift
            ;;
        --skip)
            ACTION="skip"
            shift
            ;;
        --rename-new)
            ACTION="rename_new"
            shift
            ;;
        --rename-old)
            ACTION="rename_old"
            shift
            ;;
        --no-confirm)
            CONFIRM="false"
            shift
            ;;
        *)
            print_error "Opción desconocida: $1"
            show_help
            exit 1
            ;;
    esac
done

# Verificar que el binario existe
BINARY="$PROJECT_ROOT/target/release/copymaster"
if [ ! -f "$BINARY" ]; then
    print_warning "Binary no encontrado, compilando..."
    cd "$PROJECT_ROOT"
    cargo build --release
fi

# Usar el comando de configuración del binario
print_info "Configurando manejo de conflictos..."
print_info "Acción: $ACTION"
print_info "Confirmación: $CONFIRM"

# Ejecutar comando de configuración
"$BINARY" config --conflict-action "$ACTION" --no-confirm "$CONFIRM"

if [ $? -eq 0 ]; then
    print_success "✅ Configuración de conflictos actualizada"
    echo ""
    echo "La nueva configuración será:"
    echo "  • Acción por defecto: $ACTION"
    echo "  • Pedir confirmación: $CONFIRM"
    echo ""
    echo "Los cambios se aplicarán en el próximo inicio de CopyMaster."
else
    print_error "❌ Error al actualizar la configuración"
    exit 1
fi