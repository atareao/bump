¡Por supuesto\! Analizando los archivos `main.rs`, `cli.rs` y `config.rs`, he determinado que estás creando una herramienta de línea de comandos (CLI) en **Rust** llamada **`vampus`** para automatizar el **versionado de proyectos** basado en el **Versionado Semántico (SemVer)** y la actualización de archivos de configuración (`Cargo.toml` u otros).

Aquí tienes un archivo **`README.md`** completo y profesional para tu proyecto, optimizado para GitHub:

-----

# 🧛‍♂️ Vampus: CLI de Versionado Semántico

**Vampus** es una herramienta de línea de comandos asíncrona escrita en **Rust** diseñada para simplificar y automatizar el proceso de actualización de la versión de tu proyecto, asegurando la consistencia en múltiples archivos de configuración (como `Cargo.toml`, `package.json`, etc.) basados en el esquema de **Versionado Semántico (SemVer)**.

## ✨ Características Principales

  * **Versionado Semántico (SemVer):** Soporte nativo para incrementos **`--patch`**, **`--minor`** y **`--major`** de la versión.
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
# cp target/release/vampus /usr/local/bin/
```

## ⚙️ Configuración

`Vampus` busca un archivo de configuración llamado **`.vampus.yml`** en el directorio actual. Si no existe, se creará uno por defecto automáticamente al ejecutar el primer comando.

### Formato del Archivo `.vampus.yml`

El archivo YAML debe definir la versión actual y una lista de archivos y patrones de reemplazo:

```yaml
# .vampus.yml
current_version: "0.1.0"
replaces:
  - file: Cargo.toml
    # Patrón a buscar. Usar {{current_version}} como marcador de posición.
    search: "version = \"{{current_version}}\""
    # Patrón de reemplazo. Usar {{new_version}} como marcador de posición.
    replace: "version = \"{{new_version}}\""
  
  - file: README.md
    search: "Vampus v{{current_version}}"
    replace: "Vampus v{{new_version}}"
```

### Valores por Defecto (si no existe el archivo)

Si el archivo no existe, se creará con esta configuración por defecto:

```yaml
# Valor por defecto generado
current_version: "0.1.0"
replaces:
  - file: Cargo.toml
    search: "version = \"{{current_version}}\""
    replace: "version = \"{{new_version}}\""
```

## 🚀 Uso

`Vampus` utiliza opciones excluyentes para definir el tipo de incremento de versión, adhiriéndose estrictamente a SemVer.

### Subir de Versión (`upgrade`)

Aplica el incremento de versión y actualiza todos los archivos configurados.

```bash
# Incrementa la versión PATCH (e.g., 1.0.0 -> 1.0.1) - Es el valor por defecto
vampus upgrade

# O explícitamente:
vampus upgrade --patch 

# Incrementa la versión MINOR (e.g., 1.0.1 -> 1.1.0)
vampus upgrade --minor

# Incrementa la versión MAJOR (e.g., 1.1.0 -> 2.0.0)
vampus upgrade --major
```

> ⚠️ **Nota:** Solo puedes usar una de las opciones (`--patch`, `--minor`, `--major`) a la vez.

### Previsualizar la Versión (`preview`)

Muestra el número de la nueva versión sin modificar ningún archivo.

```bash
# Ver el resultado de un incremento MINOR
vampus preview --minor 

# Salida de ejemplo:
# Current version: 1.2.3
# New version (preview): 1.3.0
```

### Mostrar Versión Actual (`show`)

Muestra la versión actual del proyecto definida en `.vampus.yml`.

```bash
vampus show
# Salida de ejemplo:
# Current version: 1.2.3
```

## ⌨️ Comandos y Opciones

| Comando | Descripción |
| :--- | :--- |
| `vampus upgrade` | Incrementa la versión del proyecto y aplica los cambios en los archivos. |
| `vampus preview` | Muestra la versión resultante sin realizar cambios permanentes. |
| `vampus show` | Muestra la versión actual leída desde el archivo de configuración. |
| **`-d`, `--debug`** | Habilita los mensajes de depuración (útil para diagnosticar fallos en el reemplazo de archivos). |

-----

## 🏗️ Estructura del Código (Para Desarrolladores)

La aplicación está organizada en tres módulos principales:

  * **`main.rs`:** Lógica principal asíncrona (`tokio::main`), manejo de la CLI, y contiene las funciones de utilidad: `get_increment_type`, `increment_version`, `get_config_path`, y `replace_in_file`.
  * **`cli.rs`:** Define la estructura de los comandos y argumentos (`Cli`, `Commands`, `UpgradeArgs`) utilizando la librería `clap`, incluyendo la **exclusión mutua** de las *flags* de versión (`--patch`, `--minor`, `--major`).
  * **`config.rs`:** Define las estructuras de datos (`Config` y `Replace`) para la serialización/deserialización YAML a través de `serde_yaml`, y contiene las funciones de I/O asíncrona para leer y escribir el archivo de configuración (`read`, `write`, `write_default`).
