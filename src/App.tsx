import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface DiskInfo {
  name: string;
  total_gb: number;
  mount_point: string;
}

interface SystemInfo {
  os_name: string;
  motherboard: string;
  bios: string;
  cpu_model: string;
  ram_total_gb: number;
  ram_type: string;
  disks: DiskInfo[];
}

function App() {
  const [datos, setDatos] = useState<SystemInfo | null>(null);

  const obtenerDatos = async () => {
    try {
      const info: SystemInfo = await invoke("run_system_report");
      setDatos(info);
    } catch (error) {
      console.error("Error obteniendo datos:", error);
    }
  };

  useEffect(() => {
    obtenerDatos();
  }, []);

  const manejarGuardado = async () => {
    if (!datos) return;

    const fecha = new Date().toLocaleString();

    // Decidimos qué etiqueta usar según si hay frecuencia detectada
    const tieneFrecuencia = datos.ram_type.includes("MHz") || datos.ram_type.includes("MT/s");
    const ramDetalle = tieneFrecuencia
    ? `Tipo/Velocidad:    ${datos.ram_type}`
    : `Tipo de Memoria:   ${datos.ram_type}`;

    const texto = `============================================================
    REPORTE TÉCNICO DE DIAGNÓSTICO DE HARDWARE
    ============================================================
    Fecha de emisión: ${fecha}
    Generado por: Analizador de Sistema (Tauri + Rust)
    ------------------------------------------------------------

    [ INFORMACIÓN DEL SISTEMA OPERATIVO ]
    ------------------------------------------------------------
    S.O. Instalado:    ${datos.os_name}
    Arquitectura:      x64 (64-bit)

    [ ESPECIFICACIONES DE LA PLACA BASE ]
    ------------------------------------------------------------
    Fabricante/Modelo: ${datos.motherboard}
    Versión de BIOS:   ${datos.bios}

    [ PROCESADOR (CPU) ]
    ------------------------------------------------------------
    Modelo:            ${datos.cpu_model}

    [ MEMORIA RAM ]
    ------------------------------------------------------------
    Capacidad Total:   ${datos.ram_total_gb.toFixed(2)} GB
    ${ramDetalle}

    [ UNIDADES DE ALMACENAMIENTO ]
    ------------------------------------------------------------
    ${datos.disks.map(d => `Punto de Montaje: ${d.mount_point.padEnd(5)} | Nombre: ${(d.name || 'Disco').padEnd(15)} | Total: ${d.total_gb} GB`).join('\n')}

    ============================================================
    ESTADO DEL DIAGNÓSTICO: FINALIZADO CON ÉXITO
    ============================================================`.trim();

    try {
      const respuesta: string = await invoke("guardar_informe", { contenido: texto });
      alert(respuesta);
    } catch (error) {
      alert("Error al guardar: " + error);
    }
  };

  if (!datos) return <div className="loading">Iniciando diagnóstico...</div>;

  return (
    <div className="container" data-tauri-drag-region>
    <style>{`
      .container {
        padding: 30px;
        color: #e0e0e0;
        background: #121212;
        min-height: 100vh;
        font-family: 'Segoe UI', system-ui, sans-serif;
        border-radius: 12px;
        position: relative;
      }
      .close-btn {
        position: absolute;
        top: 15px;
        right: 20px;
        background: #ff5f56;
        border: none;
        border-radius: 50%;
        width: 14px;
        height: 14px;
        cursor: pointer;
        z-index: 100;
      }
      h1 {
        text-align: center;
        color: #00d4ff;
        margin-bottom: 30px;
        font-weight: 300;
      }
      .grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
        gap: 20px;
      }
      .card {
        background: #1e1e1e;
        padding: 20px;
        border-radius: 15px;
        border: 1px solid #333;
      }
      .card h3 {
        margin: 0 0 15px 0;
        color: #00d4ff;
        font-size: 0.75rem;
        text-transform: uppercase;
      }
      .btn-container { margin-top: 40px; text-align: center; }
      .btn-guardar {
        background: linear-gradient(135deg, #00d4ff, #0055ff);
        color: white;
        border: none;
        padding: 14px 35px;
        border-radius: 30px;
        font-weight: bold;
        cursor: pointer;
        transition: transform 0.2s;
      }
      .btn-guardar:hover { transform: scale(1.05); }
      .loading { color: #00d4ff; text-align: center; padding-top: 100px; }
      `}</style>

      <button className="close-btn" onClick={() => window.close()} />

      <h1>Analizador de Hardware</h1>

      <div className="grid">
      <div className="card">
      <h3>Sistema y Placa Base</h3>
      <p><strong>{datos.os_name}</strong></p>
      <p style={{ color: '#00d4ff' }}>{datos.motherboard}</p>
      <p style={{ fontSize: '0.8rem', color: '#777' }}>BIOS: {datos.bios}</p>
      </div>

      <div className="card">
      <h3>Procesador</h3>
      <p>{datos.cpu_model}</p>
      </div>

      <div className="card">
      <h3>Memoria RAM</h3>
      <p style={{ fontSize: '1.8rem' }}>{datos.ram_total_gb.toFixed(2)} GB</p>
      <p style={{ color: '#00d4ff' }}>{datos.ram_type}</p>
      </div>

      <div className="card">
      <h3>Almacenamiento</h3>
      {datos.disks.map((d, i) => (
        <div key={i} style={{
          backgroundColor: '#262626',
          borderRadius: '8px',
          padding: '12px',
          marginTop: '10px',
          borderLeft: '4px solid #00d4ff',
          overflow: 'hidden'
        }}>
        <h4 style={{
          margin: 0,
          color: '#00d4ff',
          fontSize: '13px',
          whiteSpace: 'nowrap',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          width: '100%'
        }} title={d.mount_point}>
        {d.mount_point}
        </h4>
        <p style={{ margin: '5px 0 0', color: '#e0e0e0', fontSize: '0.9rem' }}>
        <strong>{d.total_gb} GB</strong> <span style={{ color: '#777', fontSize: '0.8rem' }}>({d.name})</span>
        </p>
        </div>
      ))}
      </div>
      </div>

      <div className="btn-container">
      <button className="btn-guardar" onClick={manejarGuardado}>
      💾 GENERAR INFORME .TXT
      </button>
      </div>
      </div>
  );
}

export default App;
