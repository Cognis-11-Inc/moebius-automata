//! Versioned anatomy and the canonical free-space ship runtime (MORPH-004).
//! The unchanged MORPH-003 book supplies material transitions; nursery core is separate.
mod anatomy;
mod compiler;
pub mod emission;
pub mod filament;
pub mod food;
pub mod runtime;
pub mod witness;
pub use anatomy::*;
mod math;
mod spec;
mod surface;
pub use compiler::{AttachmentFrame, CompiledWorm, Organism, Pose};
pub use spec::{Appearance, Flagellum, Mouth, Ridges, SurfaceCoordinate, WormSpec};
use std::{fmt, path::Path};
pub use surface::{MeshAudit, Surface};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphError(pub String);
impl fmt::Display for MorphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for MorphError {}
pub(crate) fn require(ok: bool, message: &str) -> Result<(), MorphError> {
    if ok {
        Ok(())
    } else {
        Err(MorphError(message.into()))
    }
}
/// Validated canonical JSON; no runtime pose or simulation state is serialized here.
pub fn to_json(spec: &WormSpec) -> Result<String, MorphError> {
    spec.validate()?;
    serde_json::to_string_pretty(spec).map_err(|e| MorphError(e.to_string()))
}
pub fn from_json(json: &str) -> Result<WormSpec, MorphError> {
    let spec: WormSpec = serde_json::from_str(json).map_err(|e| MorphError(e.to_string()))?;
    spec.validate()?;
    Ok(spec)
}
pub fn load(path: &Path) -> Result<WormSpec, MorphError> {
    from_json(&std::fs::read_to_string(path).map_err(|e| MorphError(e.to_string()))?)
}
/// Save is an explicit editor action. Write/flush a sibling before replacing the file.
pub fn save(path: &Path, spec: &WormSpec) -> Result<(), MorphError> {
    let data = to_json(spec)?;
    write_atomic(path, &data)
}
pub(crate) fn write_atomic(path: &Path, data: &str) -> Result<(), MorphError> {
    use std::io::Write;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| MorphError(e.to_string()))?;
    }
    let temporary = path.with_extension(format!("json.{}.tmp", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| MorphError(e.to_string()))?;
    let result = (|| {
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result.map_err(|e| MorphError(e.to_string()))
}
