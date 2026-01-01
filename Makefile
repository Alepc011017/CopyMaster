.PHONY: all build release debug test clean install uninstall configure flatpak appimage run daemon help generate-icons

CARGO = cargo
PREFIX = /usr/local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share
APPDIR = $(DATADIR)/applications
ICONDIR = $(DATADIR)/icons/hicolor
DESKTOP_FILE = copymaster.desktop

all: debug

debug:
	$(CARGO) build

release:
	$(CARGO) build --release

test:
	$(CARGO) test

clean:
	$(CARGO) clean

install: release
	# Crear directorios si no existen
	sudo install -d $(BINDIR)
	sudo install -d $(APPDIR)
	sudo install -d $(ICONDIR)/scalable/apps
	sudo install -d $(ICONDIR)/256x256/apps
	sudo install -d $(ICONDIR)/128x128/apps
	sudo install -d $(ICONDIR)/64x64/apps
	sudo install -d $(ICONDIR)/32x32/apps
	
	# Instalar binario
	sudo install -m755 target/release/copymaster $(BINDIR)/
	
	# Instalar iconos (si existen, si no, omitir)
	[ -f data/icons/scalable/copymaster.svg ] && sudo install -m644 data/icons/scalable/copymaster.svg $(ICONDIR)/scalable/apps/ || true
	[ -f data/icons/256x256/copymaster.png ] && sudo install -m644 data/icons/256x256/copymaster.png $(ICONDIR)/256x256/apps/ || true
	[ -f data/icons/128x128/copymaster.png ] && sudo install -m644 data/icons/128x128/copymaster.png $(ICONDIR)/128x128/apps/ || true
	[ -f data/icons/64x64/copymaster.png ] && sudo install -m644 data/icons/64x64/copymaster.png $(ICONDIR)/64x64/apps/ || true
	[ -f data/icons/32x32/copymaster.png ] && sudo install -m644 data/icons/32x32/copymaster.png $(ICONDIR)/32x32/apps/ || true
	
	# Actualizar caché de iconos
	sudo gtk-update-icon-cache -q -t -f $(ICONDIR) || true
	
	# Instalar archivo .desktop
	sudo install -m644 data/$(DESKTOP_FILE) $(APPDIR)/
	
	# Actualizar base de datos de aplicaciones
	sudo update-desktop-database $(APPDIR) || true
	
	@echo "✅ CopyMaster instalado correctamente"
	@echo "Ejecuta 'copymaster --help' para ver opciones"
	@echo "Configura auto-arranque con: copymaster autostart --enable --minimized"

uninstall:
	# Remover binario
	sudo rm -f $(BINDIR)/copymaster
	
	# Remover iconos
	sudo rm -f $(ICONDIR)/scalable/apps/copymaster.svg
	sudo rm -f $(ICONDIR)/256x256/apps/copymaster.png
	sudo rm -f $(ICONDIR)/128x128/apps/copymaster.png
	sudo rm -f $(ICONDIR)/64x64/apps/copymaster.png
	sudo rm -f $(ICONDIR)/32x32/apps/copymaster.png
	
	# Remover archivo .desktop
	sudo rm -f $(APPDIR)/$(DESKTOP_FILE)
	
	# Actualizar cachés
	sudo gtk-update-icon-cache -q -t -f $(ICONDIR) || true
	sudo update-desktop-database $(APPDIR) || true
	
	@echo "✅ CopyMaster desinstalado"

configure:
	# Configurar auto-arranque
	./target/release/copymaster autostart --enable --minimized
	
	# Crear entrada en el menú
	./target/release/copymaster install --with-desktop
	
	@echo "✅ Configuración completada"

flatpak:
	# Crear paquete Flatpak (requiere flatpak-builder)
	flatpak-builder --repo=repo --force-clean build-dir data/org.copymaster.yml
	flatpak build-bundle repo copymaster.flatpak org.copymaster

appimage:
	# Crear AppImage (requiere linuxdeploy)
	./scripts/build-appimage.sh

run:
	# Ejecutar en modo desarrollo
	$(CARGO) run -- --minimized

daemon:
	# Ejecutar como daemon
	$(CARGO) run -- --daemon

generate-icons:
	# Generar iconos
	./scripts/generate-icons.sh

help:
	@echo "Opciones disponibles:"
	@echo "  make install     - Instalar sistema"
	@echo "  make configure   - Configurar auto-arranque"
	@echo "  make uninstall   - Desinstalar"
	@echo "  make run         - Ejecutar en modo desarrollo"
	@echo "  make daemon      - Ejecutar como daemon"
	@echo "  make flatpak     - Crear paquete Flatpak"
	@echo "  make appimage    - Crear AppImage"
	@echo "  make generate-icons - Generar iconos"