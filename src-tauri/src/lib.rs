use serde::Serialize;
use std::fs;
use std::process::Command;
use tauri::command;
use std::env;

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
    cpu_freq_mhz: u64,
    ram_total_gb: f64,
    ram_type: String,
    disks: Vec<DiskInfo>,
}

fn obtener_tipo_ram() -> String {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-CimInstance Win32_PhysicalMemory | Select-Object -First 1 -Property ConfiguredClockSpeed, SMBIOSMemoryType | ForEach-Object { \"$($_.ConfiguredClockSpeed) MHz|$($_.SMBIOSMemoryType)\" }"
        ])
        .output();

        match output {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() || s.contains("0 MHz") {
                    "DDR4 (Velocidad no detectada)".to_string()
                } else {
                    let partes: Vec<&str> = s.split('|').collect();
                    let freq = partes.get(0).unwrap_or(&"N/A");
                    let tipo_code = partes.get(1).unwrap_or(&"0");

                    let tipo_nombre = match *tipo_code {
                        "26" => "DDR4",
                        "34" => "DDR5",
                        "24" => "DDR3",
                        _ => "DDR"
                    };
                    format!("{} @ {}", tipo_nombre, freq)
                }
            },
            Err(_) => "DDR4".to_string(),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // En Fedora, dmidecode requiere sudo, pero intentamos leer la velocidad si está disponible
        let speed = fs::read_to_string("/sys/class/dmi/id/memory_speed")
        .unwrap_or_else(|_| {
            let out = Command::new("sh")
            .args(["-c", "dmidecode -t memory | grep 'Speed' | head -n 1 | awk '{print $2}'"])
            .output();
            match out {
                Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
                        Err(_) => "".to_string(),
            }
        });

        if speed.is_empty() || speed == "Unknown" {
            "DDR4".to_string()
        } else {
            format!("DDR4 @ {} MHz", speed)
        }
    }
}

#[command]
fn run_system_report() -> SystemInfo {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    sys.refresh_cpu_usage();

    // Lógica de placa base (simplificada para brevedad)
    let (vendor, model, bios_ver) = if cfg!(target_os = "linux") {
        (
            fs::read_to_string("/sys/class/dmi/id/board_vendor").unwrap_or_default().trim().to_string(),
         fs::read_to_string("/sys/class/dmi/id/board_name").unwrap_or_default().trim().to_string(),
         fs::read_to_string("/sys/class/dmi/id/bios_version").unwrap_or_default().trim().to_string()
        )
    } else {
        let w_vendor = Command::new("powershell").args(["-Command", "(Get-CimInstance Win32_BaseBoard).Manufacturer"]).output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
        let w_model = Command::new("powershell").args(["-Command", "(Get-CimInstance Win32_BaseBoard).Product"]).output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
        let w_bios = Command::new("powershell").args(["-Command", "(Get-CimInstance Win32_BIOS).SMBIOSBIOSVersion"]).output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
        (w_vendor, w_model, w_bios)
    };

    let mut disks = Vec::new();
    let d_list = sysinfo::Disks::new_with_refreshed_list();
    for disk in &d_list {
        if disk.total_space() > 0 {
            disks.push(DiskInfo {
                name: disk.name().to_string_lossy().into_owned(),
                       total_gb: disk.total_space() / 1_073_741_824,
                       mount_point: disk.mount_point().to_str().unwrap_or("").to_string(),
            });
        }
    }

    SystemInfo {
        os_name: sysinfo::System::name().unwrap_or_else(|| "Sistema Desconocido".into()),
        motherboard: format!("{} {}", vendor, model),
        bios: bios_ver,
        cpu_model: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
        cpu_freq_mhz: sys.cpus().first().map(|c| c.frequency()).unwrap_or(0),
        ram_total_gb: sys.total_memory() as f64 / 1_073_741_824.0,
        ram_type: obtener_tipo_ram(),
        disks,
    }
}

#[command]
async fn guardar_informe(contenido: String) -> Result<String, String> {
    let user_dirs = directories::UserDirs::new()
    .ok_or("No se pudieron detectar directorios de usuario")?;

    // CORRECCIÓN DE TIPOS: Mantenemos referencias hasta el final
    let ruta_base = user_dirs.desktop_dir()
    .or_else(|| user_dirs.document_dir())
    .unwrap_or_else(|| user_dirs.home_dir())
    .to_path_buf(); // Aquí lo convertimos en dueño (PathBuf)

    if !ruta_base.exists() {
        fs::create_dir_all(&ruta_base).map_err(|e| e.to_string())?;
    }

    let ruta_archivo = ruta_base.join("Informe_Sistema.txt");

    fs::write(&ruta_archivo, contenido)
    .map_err(|e| format!("Error (os error {}): {}. Revisa los permisos.", e.raw_os_error().unwrap_or(0), e))?;

    Ok(format!("¡Guardado en: {}!", ruta_archivo.display()))
}

pub fn run() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![run_system_report, guardar_informe])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
