use serde_json::json;
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System,
};

/// 汇总当前机器的硬件与运行环境信息（Windows / macOS / Linux）。
#[tauri::command]
pub fn get_hardware_info() -> Result<serde_json::Value, String> {
    if !sysinfo::IS_SUPPORTED_SYSTEM {
        return Err("当前平台暂不支持通过 sysinfo 采集硬件信息".to_string());
    }

    let mut sys = System::new();
    sys.refresh_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    let os = os_info::get();
    let hostname = System::host_name().unwrap_or_else(|| "—".to_string());

    let cpus = sys.cpus();
    let cpu_brand = cpus
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_default();
    let logical_cores = cpus.len();
    let physical_cores = sys.physical_core_count().unwrap_or(logical_cores);
    let freq_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);

    let total_mem = sys.total_memory();
    let avail_mem = sys.available_memory();
    let used_mem = sys.used_memory();
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();

    let disks = Disks::new_with_refreshed_list();
    let disk_list: Vec<serde_json::Value> = disks
        .list()
        .iter()
        .map(|d| {
            json!({
                "name": d.name().to_string_lossy(),
                "kind": format!("{:?}", d.kind()),
                "mountPoint": d.mount_point().display().to_string(),
                "fileSystem": d.file_system().to_string_lossy(),
                "totalBytes": d.total_space(),
                "availableBytes": d.available_space(),
                "isRemovable": d.is_removable(),
                "isReadOnly": d.is_read_only(),
            })
        })
        .collect();

    let networks = Networks::new_with_refreshed_list();
    let mut net_list = Vec::new();
    for (iface, data) in &networks {
        net_list.push(json!({
            "interface": format!("{iface}"),
            "receivedBytes": data.total_received(),
            "transmittedBytes": data.total_transmitted(),
            "packetsReceived": data.total_packets_received(),
            "packetsTransmitted": data.total_packets_transmitted(),
            "mtu": data.mtu(),
            "macAddress": format!("{}", data.mac_address()),
        }));
    }

    let load = System::load_average();
    let load_json = if cfg!(unix) {
        Some(json!({
            "one": load.one,
            "five": load.five,
            "fifteen": load.fifteen,
        }))
    } else {
        None
    };

    Ok(json!({
        "hostname": hostname,
        "os": {
            "type": os.os_type().to_string(),
            "version": os.version().to_string(),
            "edition": os.edition().map(|s| s.to_string()),
            "codename": os.codename().map(|s| s.to_string()),
            "architecture": os.architecture().map(|s| s.to_string()),
            "bitness": os.bitness().to_string(),
        },
        "kernel": System::kernel_version().unwrap_or_else(|| "—".to_string()),
        "longOsVersion": System::long_os_version().unwrap_or_else(|| "—".to_string()),
        "osName": System::name().unwrap_or_else(|| "—".to_string()),
        "osVersionShort": System::os_version().unwrap_or_else(|| "—".to_string()),
        "distributionId": System::distribution_id(),
        "cpuArch": System::cpu_arch(),
        "bootTimeUnix": System::boot_time(),
        "uptimeSeconds": System::uptime(),
        "cpu": {
            "brand": cpu_brand,
            "logicalCores": logical_cores,
            "physicalCores": physical_cores,
            "frequencyMhz": freq_mhz,
        },
        "memory": {
            "totalBytes": total_mem,
            "availableBytes": avail_mem,
            "usedBytes": used_mem,
        },
        "swap": {
            "totalBytes": total_swap,
            "usedBytes": used_swap,
        },
        "disks": disk_list,
        "networks": net_list,
        "loadAverage": load_json,
    }))
}
