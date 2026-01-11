use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tauri::command;

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
        ("Generic".to_string(), "PC Windows".to_string(), "N/A".to_string())
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
    // RUTA DINÁMICA SEGÚN EL SISTEMA
    let ruta = if cfg!(target_os = "windows") {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:".to_string());
        format!("{}\\Desktop\\reporte_sistema.txt", user_profile)
    } else {
        "/home/Luis/Escritorio/reporte_sistema.txt".to_string()
    };

    let mut file = File::create(&ruta).map_err(|e| e.to_string())?;
    file.write_all(contenido.as_bytes()).map_err(|e| e.to_string())?;

    // En Linux devolvemos el permiso al usuario Luis
    #[cfg(target_os = "linux")]
    let _ = Command::new("chown").arg("Luis:Luis").arg(&ruta).status();

    Ok(format!("¡Informe guardado en: {}!", ruta))
}

pub fn run() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![run_system_report, guardar_informe])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
