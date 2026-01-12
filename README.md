# 🛠️ Analizador de Hardware (v0.1.0)

![Vista previa de la aplicación](../screenshots/vista-previa_1.png)
![Vista previa de la aplicación](../screenshots/vista-previa_2.png)
![Vista previa de la aplicación](../screenshots/reporte-guardado_txt.png)

Una aplicación de escritorio moderna y multiplataforma diseñada para el diagnóstico rápido de hardware. Construida con **Rust**, **Tauri** y **React**, esta herramienta ofrece una visión técnica detallada del sistema con la capacidad de exportar informes físicos.

## ✨ Características principaless

* **Detección de Hardware Real:** Identificación de placa base (OEM), modelo de BIOS y especificaciones de CPU.
* **Gestión de Memoria:** Reporte detallado de capacidad y tipo de RAM (DDR4/DDR5/Velocidad).
* **Mapeo de Almacenamiento:** Listado completo de unidades montadas y particiones con detección dinámica de rutas.
* **Exportación Inteligente:** Generación de informes `.txt` guardados automáticamente en el Escritorio (compatible con nombres de usuario dinámicos y sistemas en español/inglés).
* **Interfaz Moderna:** UI oscura optimizada con React y CSS responsivo.

## 🚀 Tecnologías utilizadas

* **Backend:** [Rust](https://www.rust-lang.org/) (Seguridad y rendimiento).
* **Frontend:** [React](https://reactjs.org/) + [TypeScript](https://www.typescriptlang.org/).
* **Framework:** [Tauri](https://tauri.app/) (Binarios ligeros y seguros).
* **CI/CD:** GitHub Actions para compilación automatizada de ejecutables (.exe).

## 🛠️ Instalación y Desarrollo

### Requisitos previos
* Rust y Cargo.
* Node.js y npm.

### Pasos para desarrollo
1. Clonar el repositorio:
   ```bash
   git clone [https://github.com/TU_USUARIO/TU_REPO.git](https://github.com/TU_USUARIO/TU_REPO.git)

2. Instalar dependencias:

Bash

npm install
3. Ejecutar en modo desarrollo:

Bash

npm run tauri dev
📦 Compilación
Para generar el instalador optimizado:

Bash

npm run tauri build

Desarrollado como proyecto de diagnóstico técnico en Fedora/Bazzite Linux.
