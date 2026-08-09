//! Resource quotas beyond decode (memory, files, wall time stubs).

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceKind {
    MemoryBytes,
    OpenFiles,
    WallTimeMs,
    CpuTimeMs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaExceeded {
    pub kind: ResourceKind,
    pub limit: u64,
    pub attempted: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceQuota {
    pub memory_bytes: u64,
    pub open_files: u64,
    pub wall_time_ms: u64,
    pub cpu_time_ms: u64,
    memory_used: u64,
    files_used: u64,
    wall_used_ms: u64,
    cpu_used_ms: u64,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self::new(256 * 1024 * 1024, 256, 60_000, 30_000)
    }
}

impl ResourceQuota {
    pub fn new(memory_bytes: u64, open_files: u64, wall_time_ms: u64, cpu_time_ms: u64) -> Self {
        Self {
            memory_bytes,
            open_files,
            wall_time_ms,
            cpu_time_ms,
            memory_used: 0,
            files_used: 0,
            wall_used_ms: 0,
            cpu_used_ms: 0,
        }
    }

    pub fn charge(&mut self, kind: ResourceKind, amount: u64) -> Result<(), QuotaExceeded> {
        let (used, limit) = match kind {
            ResourceKind::MemoryBytes => (&mut self.memory_used, self.memory_bytes),
            ResourceKind::OpenFiles => (&mut self.files_used, self.open_files),
            ResourceKind::WallTimeMs => (&mut self.wall_used_ms, self.wall_time_ms),
            ResourceKind::CpuTimeMs => (&mut self.cpu_used_ms, self.cpu_time_ms),
        };
        let attempted = used.saturating_add(amount);
        if attempted > limit {
            return Err(QuotaExceeded {
                kind,
                limit,
                attempted,
            });
        }
        *used = attempted;
        Ok(())
    }

    pub fn used(&self, kind: ResourceKind) -> u64 {
        match kind {
            ResourceKind::MemoryBytes => self.memory_used,
            ResourceKind::OpenFiles => self.files_used,
            ResourceKind::WallTimeMs => self.wall_used_ms,
            ResourceKind::CpuTimeMs => self.cpu_used_ms,
        }
    }

    pub fn release(&mut self, kind: ResourceKind, amount: u64) {
        let used = match kind {
            ResourceKind::MemoryBytes => &mut self.memory_used,
            ResourceKind::OpenFiles => &mut self.files_used,
            ResourceKind::WallTimeMs => &mut self.wall_used_ms,
            ResourceKind::CpuTimeMs => &mut self.cpu_used_ms,
        };
        *used = used.saturating_sub(amount);
    }
}
