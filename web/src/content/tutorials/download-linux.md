---
title: "Descargar e instalar en Linux"
description: "Guía paso a paso para instalar Academix en distribuciones Linux basadas en Debian/Ubuntu."
os: linux
type: download
order: 3
---

## Requisitos

- Distribución basada en Debian/Ubuntu (Ubuntu 22.04+, Debian 12+, Linux Mint 21+)
- WebKitGTK instalado
- Arquitectura x64 o arm64

## Instalar dependencias

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0
```

## Pasos de instalación

1. Visita la [página de descargas](/downloads) y haz clic en el botón de descarga para Linux.
2. Instala el paquete `.deb` descargado:

```bash
sudo dpkg -i academix_*.deb
```

3. Si hay dependencias faltantes, ejecuta:

```bash
sudo apt --fix-broken install
```

4. Abre Academix desde el menú de aplicaciones o ejecutando `academix` en la terminal.

## Primer inicio

Al abrir la aplicación por primera vez, se te pedirá crear una cuenta de administrador. Ingresa tu email y una contraseña segura para comenzar.
