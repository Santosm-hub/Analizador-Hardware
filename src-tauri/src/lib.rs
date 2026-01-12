use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tauri::command;
use std::env;
use std::path::PathBuf;

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    total_gb: u64,
    mount_point: String,
}

#[derive(Serialize)]
pub struct SystemInfo {
    os_name: String,
    motherboard: String,
    bios: String,
    cpu_model: String,
    ram_total_gb: f64,
    ram_type: String,
    disks: Vec<DiskInfo>,
}

fn read_sys_file(path: &str) -> String {
    fs::read_to_string(path)
    .unwrap_or_else(|_| "N/A".to_string())
    .trim()
    .to_string()
}

fn obtener_tipo_ram() -> String {
    // LÓGICA PARA WINDOWS
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
        .arg("-Command")
        .arg("Get-CimInstance Win32_PhysicalMemory | Select-Object -First 1 @{L='Info';E={($_.ConfiguredClockSpeed).ToString() + ' MT/s ' + (if($_.MemoryType -eq 0){'DDR4/DDR5'})}} | Select-Object -ExpandProperty Info")
        .output();

        match output {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() { "DDR4".to_string() } else { s }
            },
            Err(_) => "DDR4".to_string(),
        }
    }

    // LÓGICA PARA LINUX (Bazzite/Fedora)
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("sh")
        .arg("-c")
        .arg("/usr/sbin/dmidecode -t memory | grep -E 'Type: DDR|Speed: [0-9]' | head -n 2")
        .output();

        match output {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() {
                    "DDR4 (Probable)".to_string()
                } else {
                    s.replace("Type: ", "")
                    .replace("\nSpeed: ", " @ ")
                    .replace("Speed: ", " @ ")
                    .trim()
                    .to_string()
                }
            },
            Err(_) => "DDR4".to_string(),
        }
    }
}

#[command]
fn run_system_report() -> SystemInfo {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    // Datos de placa base (ajustados para ser multiplataforma)
    let (vendor, model, bios_ver) = if cfg!(target_os = "linux") {
        (
            read_sys_file("/sys/class/dmi/id/board_vendor"),
         read_sys_file("/sys/class/dmi/id/board_name"),
         read_sys_file("/sys/class/dmi/id/bios_version")
        )
    } else {
        // MEJORA PARA WINDOWS: Consultar fabricante y modelo real
        let wmic_vendor = Command::new("powershell")
        .args(["-Command", "(Get-CimInstance Win32_BaseBoard).Manufacturer"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "Generic".to_string());

        let wmic_model = Command::new("powershell")
        .args(["-Command", "(Get-CimInstance Win32_BaseBoard).Product"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "PC Windows".to_string());

        let wmic_bios = Command::new("powershell")
        .args(["-Command", "(Get-CimInstance Win32_BIOS).SMBIOSBIOSVersion"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "N/A".to_string());

        (wmic_vendor, wmic_model, wmic_bios)
    };

    let ram_tipo = obtener_tipo_ram();

    let mut disks = Vec::new();
    let d_list = sysinfo::Disks::new_with_refreshed_list();
    for disk in &d_list {
        let mount = disk.mount_point().to_str().unwrap_or("");
        if disk.total_space() > 0 {
            disks.push(DiskInfo {
                name: disk.name().to_string_lossy().into_owned(),
                       total_gb: disk.total_space() / 1_073_741_824,
                       mount_point: mount.to_string(),
            });
        }
    }

    SystemInfo {
        os_name: sysinfo::System::name().unwrap_or_else(|| "Sistema Desconocido".into()),
        motherboard: format!("{} {}", vendor, model),
        bios: bios_ver,
        cpu_model: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
        ram_total_gb: sys.total_memory() as f64 / 1_073_741_824.0,
        ram_type: ram_tipo,
        disks,
    }
}

#[command]
fn guardar_informe(contenido: String) -> Result<String, String> {
    // 1. OBTENER RUTA BASE (Igual que antes)
    let mut ruta = if cfg!(target_os = "windows") {
        let base = env::var("USERPROFILE").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("C:\\"));
        base.join("Desktop")
    } else {
        let base = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"));
        let mut p = base.join("Desktop");
        if !p.exists() {
            p = base.join("Escritorio");
        }
        p
    };

    // --- NUEVA LÓGICA DE SEGURIDAD ---
    // 2. ASEGURAR QUE LA CARPETA EXISTE (Si no existe, la crea)
    fs::create_dir_all(&ruta).map_err(|e| format!("No se pudo crear el directorio: {}", e))?;

    // 3. AÑADIR NOMBRE DEL ARCHIVO
    ruta.push("reporte_sistema.txt");

    // 4. GUARDAR
    let mut file = File::create(&ruta).map_err(|e| format!("Error al crear archivo: {}", e))?;
    file.write_all(contenido.as_bytes()).map_err(|e| e.to_string())?;

    // 5. PERMISOS (Solo Linux)
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&ruta) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o644);
            let _ = std::fs::set_permissions(&ruta, perms);
        }
    }

    Ok(format!("¡Informe guardado en: {}!", ruta.display()))
}

pub fn run() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![run_system_report, guardar_informe])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
