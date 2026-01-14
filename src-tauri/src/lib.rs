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
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
        .arg("-Command")
        .arg("Get-CimInstance Win32_PhysicalMemory | Select-Object -First 1 @{L='Info';E={($_.ConfiguredClockSpeed).ToString() + ' MHz ' + (if($_.MemoryType -eq 0){'DDR4/DDR5'})}} | Select-Object -ExpandProperty Info")
        .output();

        match output {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() { "DDR4".to_string() } else { s }
            },
            Err(_) => "DDR4".to_string(),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Intentamos leer la frecuencia desde el sistema de archivos de Linux
        let speed = fs::read_to_string("/sys/class/dmi/id/memory_speed") // Algunos sistemas Fedora lo tienen aquí
        .unwrap_or_else(|_| {
            // Si no, intentamos un comando rápido que no suele pedir sudo
            let out = Command::new("sh")
            .arg("-c")
            .arg("dmidecode -t memory | grep 'Speed' | head -n 1 | awk '{print $2}'")
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
async fn guardar_informe(contenido: String) -> Result<String, String> {
    // Usamos la librería 'directories' que ya tienes y funciona bien
    let user_dirs = directories::UserDirs::new()
    .ok_or("No se pudieron detectar directorios de usuario")?;

    let ruta_escritorio = user_dirs.desktop_dir()
    .ok_or("No se encontró el Escritorio")?;

    let ruta_archivo = ruta_escritorio.join("Informe_Sistema.txt");

    fs::write(&ruta_archivo, contenido)
    .map_err(|e| format!("Error (os error {}): {}", e.raw_os_error().unwrap_or(0), e))?;

    Ok(format!("¡Guardado en el Escritorio!"))
}

pub fn run() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![run_system_report, guardar_informe])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
