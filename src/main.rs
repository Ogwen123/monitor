mod utils;

use actix_web::{get, App, HttpResponse, HttpServer, Error};
use actix_files::NamedFile;
use std::path::PathBuf;
use std::env::{current_dir, current_exe};
use sysinfo::{Disks, System};
use serde::Serialize;
use crate::utils::logger::{fatal};
use whoami;

#[derive(Serialize)]
struct CoreUsage {
    name: String,
    usage: f32
}

#[derive(Serialize)]
struct CpuStats {
    name: String,
    average_usage: f32,
    frequency: u64,
    cores: Vec<CoreUsage>
}

#[derive(Serialize)]
struct MemoryStats {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64
}

#[derive(Serialize)]
struct DiskUsage {
    name: String,
    disk_type: String,
    total_space: u64,
    available_space: u64
}

#[derive(Serialize)]
struct Stats {
    name: String,
    os: String,
    architecture: String,
    uptime: u64,
    cpu: CpuStats,
    memory: MemoryStats,
    disk: Vec<DiskUsage>,
}

fn get_dir_path() -> String {
    let exe_path = current_exe().unwrap();
    if std::env::consts::OS == "linux" {
        let mut path_vec: Vec<&str> = exe_path.to_str().unwrap().split("/").collect();
        path_vec.pop();
        (path_vec.join("/") + "/pages/").to_string()
    } else if std::env::consts::OS == "windows" {
        let mut path_vec: Vec<&str> = exe_path.to_str().unwrap().split("\\").collect();
        path_vec.pop();
        (path_vec.join("\\") + "\\pages\\").to_string()
    } else {
        "".to_string()
    }
}

#[get("/")]
async fn index() -> actix_web::Result<NamedFile> {
    let mut path: PathBuf = PathBuf::from(get_dir_path() + "index.html");
    Ok(NamedFile::open(path)?)
}

#[get("/index.css")] // why do I have to do this 💀
async fn css() -> actix_web::Result<NamedFile> {
    let mut path: PathBuf = PathBuf::from(get_dir_path() + "index.css");
    Ok(NamedFile::open(path)?)
}

#[get("/stats")]
async fn stats() -> Result<HttpResponse, Error> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // get cpu data
    let mut cpu_stats = CpuStats {
        name: sys.cpus()[0].brand().trim().parse()?,
        average_usage: sys.global_cpu_usage(),
        frequency: 0,
        cores: Vec::new()
    };
    let mut frequency_total: u64 = 0;
    for cpu in sys.cpus() {
        let core = CoreUsage {
            name: cpu.name().parse()?,
            usage: cpu.cpu_usage()
        };
        frequency_total += cpu.frequency();
        cpu_stats.cores.push(core)
    }

    cpu_stats.frequency = frequency_total / sys.cpus().len() as u64;

    // get memory data
    let memory_stats = MemoryStats {
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap()
    };

    // get disk data
    let mut disk_vec: Vec<DiskUsage> = Vec::new();

    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        if disk.is_removable() { continue; }
        let disk_stats = DiskUsage {
            name: disk.name().to_str().unwrap().parse()?,
            disk_type: disk.kind().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space()
        };

        disk_vec.push(disk_stats);
    }

    let stats = Stats {
        name: whoami::devicename(),
        os: whoami::platform().to_string(),
        architecture: whoami::arch().to_string(),
        uptime: System::uptime(),
        cpu: cpu_stats,
        memory: memory_stats,
        disk: disk_vec
    };

    return Ok(HttpResponse::Ok().json(stats));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("{}", std::env::consts::OS);
    if sysinfo::IS_SUPPORTED_SYSTEM == false {
        fatal!("This is not a supported operating system.");
        std::process::exit(1);
    }

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(stats)
            .service(css)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
