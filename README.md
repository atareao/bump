# 🧛‍♂️ Vampus: CLI de Versionado Semántico

**Vampus** es una herramienta de línea de comandos asíncrona escrita en **Rust** diseñada para simplificar y automatizar el proceso de actualización y **retroceso (downgrade)** de la versión de tu proyecto, asegurando la consistencia en múltiples archivos de configuración (como `Cargo.toml`, `package.json`, etc.) basados en el esquema de **Versionado Semántico (SemVer)**.

## ✨ Características Principales

  * **Versionado Semántico (SemVer):** Soporte nativo para incrementos (`--patch`, `--minor`, `--major`) y **decrementos** de la versión.
  * **Comando `Downgrade`:** Permite retroceder la versión del proyecto a un nivel SemVer anterior (patch, minor, o major).
  * **Reemplazo de Cadenas en Archivos:** Busca y reemplaza de forma asíncrona la versión antigua por la nueva en múltiples archivos definidos por el usuario.
  * **Configuración Flexible:** Utiliza un archivo `.vampus.yml` para definir la versión actual del proyecto y las rutas de los archivos y patrones de búsqueda/reemplazo.
  * **Comando `Preview`:** Permite ver la próxima versión sin aplicar cambios, facilitando la validación.
  * **Alto Rendimiento:** Construido sobre **Tokio** para operaciones de I/O rápidas y asíncronas.

## 🛠️ Instalación

*(Instrucciones genéricas. Reemplaza con tus pasos de publicación si usas `cargo install` o precompilados.)*

```bash
# 1. Clona el repositorio
git clone <URL_DE_TU_REPOSITORIO>
cd vampus

# 2. Compila el proyecto
cargo build --release

# 3. Mueve el binario (opcional, para usarlo globalmente)
# cp target/release/vampus /usr/local/bin
