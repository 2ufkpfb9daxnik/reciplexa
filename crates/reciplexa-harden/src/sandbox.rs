//! Sandbox policy and fence stubs.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SandboxRequirement {
    None,
    ProcessIsolation,
    SandboxRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxDecision {
    Allow,
    Deny,
    RequireFence,
}

/// Declared policy for an untrusted or foreign boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxPolicy {
    pub requirement: SandboxRequirement,
    pub allow_network: bool,
    pub allow_filesystem_write: bool,
    pub allow_native_code: bool,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            requirement: SandboxRequirement::ProcessIsolation,
            allow_network: false,
            allow_filesystem_write: false,
            allow_native_code: false,
        }
    }
}

impl SandboxPolicy {
    pub fn decide(
        &self,
        wants_network: bool,
        wants_fs_write: bool,
        wants_native: bool,
    ) -> SandboxDecision {
        if wants_network && !self.allow_network {
            return SandboxDecision::Deny;
        }
        if wants_fs_write && !self.allow_filesystem_write {
            return SandboxDecision::Deny;
        }
        if wants_native && !self.allow_native_code {
            return SandboxDecision::Deny;
        }
        match self.requirement {
            SandboxRequirement::None => SandboxDecision::Allow,
            SandboxRequirement::ProcessIsolation | SandboxRequirement::SandboxRequired => {
                SandboxDecision::RequireFence
            }
        }
    }

    pub fn is_strict(&self) -> bool {
        !self.allow_network && !self.allow_filesystem_write && !self.allow_native_code
    }
}

/// Fence stub: marks that a sandboxed worker boundary was entered/exited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxFence {
    pub policy: SandboxPolicy,
    entered: bool,
}

impl SandboxFence {
    pub fn new(policy: SandboxPolicy) -> Self {
        Self {
            policy,
            entered: false,
        }
    }

    pub fn enter(&mut self) -> Result<(), &'static str> {
        if self.entered {
            return Err("fence already entered");
        }
        if self.policy.requirement == SandboxRequirement::None {
            self.entered = true;
            return Ok(());
        }
        self.entered = true;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), &'static str> {
        if !self.entered {
            return Err("fence not entered");
        }
        self.entered = false;
        Ok(())
    }

    pub fn is_entered(&self) -> bool {
        self.entered
    }
}
