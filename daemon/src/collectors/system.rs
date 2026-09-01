use sysinfo::System;

pub struct SystemReading {
    pub cpu_percent: f32,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
}

/// Wraps sysinfo's System so one instance stays alive across polls. CPU
/// usage is measured as the delta since the last refresh, so a fresh
/// System on every read would always report 0%.
pub struct SystemCollector {
    sys: System,
}

impl SystemCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        Self { sys }
    }

    pub fn read(&mut self) -> SystemReading {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        SystemReading {
            cpu_percent: self.sys.global_cpu_usage(),
            mem_used_bytes: self.sys.used_memory(),
            mem_total_bytes: self.sys.total_memory(),
        }
    }
}
