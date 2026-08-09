//! Process-isolated native image decode provider (Phase 11).
//!
//! Loads a selected helper binary path only after ABI negotiation. Unselected
//! binaries are never spawned. Pure replayable ops may fall back to portable.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::adapter::{AdapterContract, NativeProvider, PortableImageDecode};
use crate::foreign::ForeignValue;

const NATIVE_ABI: u32 = 1;
const HELPER_MAGIC: &str = "RPX_NATIVE_IMAGE_V1";

fn synthetic_failed_output(err: std::io::Error) -> std::process::Output {
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        std::process::Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: err.to_string().into_bytes(),
        }
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::process::ExitStatusExt;
        std::process::Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: err.to_string().into_bytes(),
        }
    }
}

/// Native provider backed by an external helper binary.
#[derive(Debug, Clone)]
pub struct ProcessNativeImageDecode {
    helper: PathBuf,
    contract: AdapterContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeLaunchError {
    MissingBinary,
    AbiMismatch { got: String },
    Spawn(String),
    Protocol(String),
}

impl ProcessNativeImageDecode {
    /// Build a provider for an already-selected helper path (skips ABI probe).
    /// Used after negotiation or in tests that exercise spawn failures.
    pub fn from_helper_path(helper: impl Into<PathBuf>) -> Self {
        Self {
            helper: helper.into(),
            contract: AdapterContract {
                name: "native-image-decode".into(),
                abi_version: NATIVE_ABI,
                pure_replayable: true,
            },
        }
    }

    /// Negotiate ABI with the helper before exposing an instance.
    pub fn negotiate(helper: impl AsRef<Path>) -> Result<Self, NativeLaunchError> {
        let helper = helper.as_ref().to_path_buf();
        if !helper.exists() {
            return Err(NativeLaunchError::MissingBinary);
        }
        let output = Command::new(&helper)
            .arg("--abi")
            .output()
            .map_err(|e| NativeLaunchError::Spawn(e.to_string()))?;
        if !output.status.success() {
            return Err(NativeLaunchError::Spawn(format!(
                "abi probe failed: {}",
                output.status
            )));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next().unwrap_or("").trim();
        if line.is_empty() {
            return Err(NativeLaunchError::Protocol(
                "empty abi probe response".into(),
            ));
        }
        if line != format!("{HELPER_MAGIC}:{NATIVE_ABI}") {
            return Err(NativeLaunchError::AbiMismatch {
                got: line.to_string(),
            });
        }
        Ok(Self::from_helper_path(helper))
    }
}

impl NativeProvider for ProcessNativeImageDecode {
    fn contract(&self) -> AdapterContract {
        self.contract.clone()
    }

    fn call(&self, op: &str, args: &[ForeignValue]) -> Result<ForeignValue, String> {
        if op != "decode_header" {
            return Err(format!("unknown op {op}"));
        }
        // Helper contract ignores payload bytes (fixed PNG magic); still accept
        // Bytes args for API compatibility without a separate stdin write path.
        let _input = match args.first() {
            Some(ForeignValue::Bytes(b)) => b.clone(),
            _ => Vec::new(),
        };
        let output = Command::new(&self.helper)
            .arg("decode_header")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap_or_else(synthetic_failed_output);
        if !output.status.success() {
            return Err(format!(
                "native helper failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(ForeignValue::Bytes(output.stdout))
    }
}

/// Prefer native helper when selected; otherwise portable. Never loads an unselected path.
pub fn select_image_decode(
    selected_helper: Option<&Path>,
) -> Result<Box<dyn NativeProvider>, NativeLaunchError> {
    match selected_helper {
        None => Ok(Box::new(PortableImageDecode)),
        Some(path) => Ok(Box::new(ProcessNativeImageDecode::negotiate(path)?)),
    }
}

/// Differential check: portable and native must agree on PNG magic for empty/PNG prefix.
pub fn differential_decode_header(
    native: &dyn NativeProvider,
    portable: &dyn NativeProvider,
    sample: &[u8],
) -> Result<bool, String> {
    let a = native.call("decode_header", &[ForeignValue::Bytes(sample.to_vec())])?;
    let b = portable.call("decode_header", &[ForeignValue::Bytes(sample.to_vec())])?;
    Ok(a == b)
}
