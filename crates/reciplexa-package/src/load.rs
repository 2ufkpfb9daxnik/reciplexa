//! Local-path package discovery and module loading.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use reciplexa_bind::{
    elaborate_units_with_bodies, parse_imports, ElaboratedUnit, ModuleError, UnitBody,
};
use reciplexa_core::expr::CoreExpr;

use reciplexa_eval::{eval_expr_with_extra, primitive_env, RuntimeValue, UnitHost};

use crate::domain_native::{DomainNativeBindMap, DomainNativeModule, DomainNativeRegistry};
use crate::domain_native_env::build_domain_native_eval_env;
use crate::manifest::{DependencySpec, PackageManifest};
use crate::registry::LocalRegistryMirror;
use crate::resource_value::maybe_materialize_package_resources_for_entry;
use crate::rpi::parse_rpi_exports;
use crate::rpxm::{parse_rpxm, RpxmError};
use crate::workspace::verify_package_lock_for_entry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageLoadError {
    Io(String),
    Manifest(RpxmError),
    Module(ModuleError),
    Interface(String),
    NotFound(String),
    Ambiguous(String),
}

impl From<ModuleError> for PackageLoadError {
    fn from(e: ModuleError) -> Self {
        Self::Module(e)
    }
}

impl From<RpxmError> for PackageLoadError {
    fn from(e: RpxmError) -> Self {
        Self::Manifest(e)
    }
}

impl std::fmt::Display for PackageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(s) | Self::NotFound(s) | Self::Ambiguous(s) | Self::Interface(s) => {
                write!(f, "{s}")
            }
            Self::Manifest(e) => write!(f, "manifest error: {e:?}"),
            Self::Module(e) => write!(f, "{}", e.message),
        }
    }
}

/// A package module source plus optional `.rpi` export boundary (MOD §8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImport {
    pub unit_name: String,
    pub source: String,
    /// When `interface-root` is set, the public export names from the `.rpi` stub.
    pub interface_exports: Option<Vec<String>>,
    /// Canonical `package/module` native registry key, when this import is native-backed.
    ///
    /// Distinct from [`Self::unit_name`], which keeps the author import spelling
    /// (`g/shapes`, bare `graphics`, or `graphics/shapes`).
    pub native_module_path: Option<String>,
}

/// Index of packages discovered under local search roots (`packages/` dirs).
#[derive(Debug, Clone, Default)]
pub struct LocalPackageIndex {
    /// Formal package name → (package root, manifest).
    packages: BTreeMap<String, (PathBuf, PackageManifest)>,
    /// Import first-segment alias → formal package name (Slice B path deps).
    aliases: BTreeMap<String, String>,
    /// Modules whose bodies are Rust-synthesized (skip `src/*.rpx`).
    native: DomainNativeRegistry,
    /// Differential / test harness only. Production [`Self::discover`] leaves this empty.
    hybrid_source_overrides: BTreeMap<String, String>,
}

impl LocalPackageIndex {
    /// Attach / replace the domain-native registry (N0.4+).
    pub fn with_native(mut self, native: DomainNativeRegistry) -> Self {
        self.native = native;
        self
    }

    pub fn set_native(&mut self, native: DomainNativeRegistry) {
        self.native = native;
    }

    pub fn native(&self) -> &DomainNativeRegistry {
        &self.native
    }

    pub fn native_mut(&mut self) -> &mut DomainNativeRegistry {
        &mut self.native
    }

    /// Register a single native module on this index.
    pub fn register_native(&mut self, module: DomainNativeModule) {
        self.native.register(module);
    }

    /// Load a native module from Hybrid/portable RPX instead of the DN2 stub.
    ///
    /// Production hosts never call this. Differential tests use it to compare
    /// reference bodies with Direct Native without putting those bodies on
    /// [`DomainNativeRegistry`].
    pub fn set_hybrid_source_override(
        &mut self,
        module_path: &str,
        source: impl Into<String>,
    ) -> bool {
        if !self.native.contains(module_path) {
            return false;
        }
        self.hybrid_source_overrides
            .insert(module_path.to_string(), source.into());
        true
    }

    /// Apply Hybrid v1 reference bodies for the listed native modules.
    pub fn with_hybrid_reference_bodies(&self, module_paths: &[&str]) -> Result<Self, String> {
        let mut out = self.clone();
        for path in module_paths {
            let source = crate::domain_bodies::hybrid_reference_source(path)
                .ok_or_else(|| format!("no hybrid reference body registered for `{path}`"))?;
            if !out.set_hybrid_source_override(path, source) {
                return Err(format!("unknown native module `{path}`"));
            }
        }
        Ok(out)
    }

    fn native_module_source(&self, module_path: &str, native: &DomainNativeModule) -> String {
        self.hybrid_source_overrides
            .get(module_path)
            .cloned()
            .unwrap_or_else(|| native.effective_source())
    }
    /// Scan each search root for immediate child directories containing `package.rpxm`.
    pub fn discover(search_roots: &[impl AsRef<Path>]) -> Result<Self, PackageLoadError> {
        let mut packages: BTreeMap<String, (PathBuf, PackageManifest)> = BTreeMap::new();
        for root in search_roots {
            let root = root.as_ref();
            if !root.is_dir() {
                continue;
            }
            let entries = fs::read_dir(root)
                .map_err(|e| PackageLoadError::Io(format!("read_dir `{}`: {e}", root.display())))?;
            for entry in entries {
                let entry =
                    entry.map_err(|e| PackageLoadError::Io(format!("read_dir entry: {e}")))?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let manifest_path = path.join("package.rpxm");
                if !manifest_path.is_file() {
                    continue;
                }
                let src = fs::read_to_string(&manifest_path).map_err(|e| {
                    PackageLoadError::Io(format!("read `{}`: {e}", manifest_path.display()))
                })?;
                let manifest = parse_rpxm(&src)?;
                if let Some((existing, _)) = packages.get(&manifest.name) {
                    return Err(PackageLoadError::Ambiguous(format!(
                        "duplicate package `{}` at `{}` and `{}`",
                        manifest.name,
                        existing.display(),
                        path.display()
                    )));
                }
                packages.insert(manifest.name.clone(), (path, manifest));
            }
        }
        Ok(Self {
            packages,
            aliases: BTreeMap::new(),
            native: crate::domain_bodies::std_domain_natives(),
            hybrid_source_overrides: BTreeMap::new(),
        })
    }

    pub fn get(&self, name: &str) -> Option<&(PathBuf, PackageManifest)> {
        let formal = self.aliases.get(name).map(String::as_str).unwrap_or(name);
        self.packages.get(formal)
    }

    pub fn package_names(&self) -> impl Iterator<Item = &str> {
        self.packages.keys().map(String::as_str)
    }

    pub fn resolve_alias<'a>(&'a self, alias: &'a str) -> &'a str {
        self.aliases.get(alias).map(String::as_str).unwrap_or(alias)
    }

    /// Slice B: register `(dependencies (alias package name version path "…"))` entries
    /// relative to `consumer_root`, mapping import aliases to package instances.
    pub fn register_path_dependencies(
        &mut self,
        consumer_root: &Path,
        manifest: &PackageManifest,
    ) -> Result<(), PackageLoadError> {
        for dep in &manifest.dependencies {
            let Some(rel) = &dep.path else {
                continue;
            };
            let dep_root = consumer_root.join(rel);
            let manifest_path = dep_root.join("package.rpxm");
            if !manifest_path.is_file() {
                return Err(PackageLoadError::NotFound(format!(
                    "path dependency `{}` missing package.rpxm at `{}`",
                    dep.name,
                    manifest_path.display()
                )));
            }
            let src = fs::read_to_string(&manifest_path).map_err(|e| {
                PackageLoadError::Io(format!("read `{}`: {e}", manifest_path.display()))
            })?;
            let parsed = parse_rpxm(&src)?;
            let formal = dep.package.clone().unwrap_or_else(|| parsed.name.clone());
            if formal != parsed.name {
                return Err(PackageLoadError::NotFound(format!(
                    "path dependency `{}` expects package `{formal}` but found `{}`",
                    dep.name, parsed.name
                )));
            }
            // Exact version match for path deps (Slice B; no range solver).
            if dep.version_req != "*" && !version_matches_exact(&dep.version_req, &parsed.version) {
                return Err(PackageLoadError::NotFound(format!(
                    "path dependency `{}` wants version `{}` but `{}` is `{}`",
                    dep.name, dep.version_req, formal, parsed.version
                )));
            }
            // If already discovered via search roots, keep that instance and only wire the alias.
            if !self.packages.contains_key(&formal) {
                self.packages.insert(formal.clone(), (dep_root, parsed));
            }
            self.aliases.insert(dep.name.clone(), formal);
        }
        Ok(())
    }

    /// Register path-free `source registry` (and default-registry) deps from a local mirror.
    pub fn register_registry_dependencies(
        &mut self,
        manifest: &PackageManifest,
        mirror: &LocalRegistryMirror,
    ) -> Result<(), PackageLoadError> {
        for dep in &manifest.dependencies {
            if dep.path.is_some() || dep.source.as_deref() == Some("workspace") {
                continue;
            }
            if dep.source.as_deref() != Some("registry") && dep.source.is_some() {
                continue;
            }
            let formal = dep.package.as_deref().unwrap_or(dep.name.as_str());
            if self.packages.contains_key(formal) {
                self.aliases.insert(dep.name.clone(), formal.to_string());
                continue;
            }
            let (root, parsed) = mirror
                .resolve(formal, &dep.version_req)
                .map_err(|e| PackageLoadError::NotFound(e.to_string()))?;
            if parsed.name != formal {
                return Err(PackageLoadError::NotFound(format!(
                    "registry dependency `{}` expects package `{formal}` but mirror has `{}`",
                    dep.name, parsed.name
                )));
            }
            self.packages.insert(formal.to_string(), (root, parsed));
            self.aliases.insert(dep.name.clone(), formal.to_string());
        }
        Ok(())
    }

    /// Discover search roots, then overlay a consumer manifest's path deps / aliases.
    pub fn discover_with_consumer(
        search_roots: &[impl AsRef<Path>],
        consumer_root: impl AsRef<Path>,
    ) -> Result<(Self, PackageManifest), PackageLoadError> {
        let mut index = Self::discover(search_roots)?;
        let consumer_root = consumer_root.as_ref();
        let manifest_path = consumer_root.join("package.rpxm");
        let src = fs::read_to_string(&manifest_path).map_err(|e| {
            PackageLoadError::Io(format!("read `{}`: {e}", manifest_path.display()))
        })?;
        let manifest = parse_rpxm(&src)?;
        index.register_path_dependencies(consumer_root, &manifest)?;
        if let Some(mirror) = LocalRegistryMirror::discover_near(consumer_root) {
            index.register_registry_dependencies(&manifest, &mirror)?;
        }
        Ok((index, manifest))
    }

    /// Resolve `package` or `package/module/…` to `(unit_name, source)`.
    pub fn resolve_import(&self, import_path: &str) -> Result<(String, String), PackageLoadError> {
        let resolved = self.resolve_import_detailed(import_path)?;
        Ok((resolved.unit_name, resolved.source))
    }

    /// Resolve an import and load the optional `.rpi` export list (MOD §8 / §2.3).
    ///
    /// When the resolved `package/module` path is in [`DomainNativeRegistry`], the
    /// `.rpx` body is **not** read; [`DomainNativeRegistry::module_source`] is used.
    pub fn resolve_import_detailed(
        &self,
        import_path: &str,
    ) -> Result<ResolvedImport, PackageLoadError> {
        let (pkg_alias, module_rest) = split_package_import(import_path);
        let pkg_name = self.resolve_alias(pkg_alias);

        // Pure-native modules with no package.rpxm on disk.
        if !self.packages.contains_key(pkg_name) {
            if let Some(native) = self.native.get(import_path).or_else(|| {
                if module_rest.is_empty() {
                    None
                } else {
                    self.native.get(&format!("{pkg_name}/{module_rest}"))
                }
            }) {
                return Ok(ResolvedImport {
                    unit_name: import_path.to_string(),
                    source: self.native_module_source(&native.module_path, native),
                    interface_exports: Some(native.exports.clone()),
                    native_module_path: Some(native.module_path.clone()),
                });
            }
        }

        let (root, manifest) = self.packages.get(pkg_name).ok_or_else(|| {
            PackageLoadError::NotFound(format!("package `{pkg_alias}` not found on search path"))
        })?;

        let module_path = if module_rest.is_empty() {
            // Bare `(import graphics)` — require a public module with the same name, else first public.
            if manifest.public_modules.iter().any(|m| m == pkg_name) {
                pkg_name.to_string()
            } else if let Some(first) = manifest.public_modules.first() {
                first.clone()
            } else {
                return Err(PackageLoadError::NotFound(format!(
                    "package `{pkg_name}` has no public modules to import"
                )));
            }
        } else {
            module_rest.to_string()
        };

        if !manifest.public_modules.is_empty()
            && !manifest.public_modules.iter().any(|m| m == &module_path)
        {
            return Err(PackageLoadError::NotFound(format!(
                "module `{module_path}` is not public in package `{pkg_name}`"
            )));
        }

        let native_key = format!("{pkg_name}/{module_path}");
        if let Some(native) = self.native.get(&native_key) {
            // Direct Native v2: registry wins over any on-disk `src/*.rpx` for this module.
            // Portable fallback when native is unavailable: OPEN-NATIVE-PKG-001 (not implemented).
            let interface_exports =
                if let Some(rpi) = manifest.module_interface_path(root, &module_path) {
                    if rpi.is_file() {
                        let rpi_src = fs::read_to_string(&rpi).map_err(|e| {
                            PackageLoadError::Io(format!("failed to read `{}`: {e}", rpi.display()))
                        })?;
                        Some(parse_rpi_exports(&rpi_src)?)
                    } else {
                        Some(native.exports.clone())
                    }
                } else {
                    Some(native.exports.clone())
                };
            return Ok(ResolvedImport {
                unit_name: import_path.to_string(),
                source: self.native_module_source(&native_key, native),
                interface_exports,
                native_module_path: Some(native.module_path.clone()),
            });
        }

        // When interface-root is set, require `.rpi` and parse its public export names.
        let interface_exports =
            if let Some(rpi) = manifest.module_interface_path(root, &module_path) {
                if !rpi.is_file() {
                    return Err(PackageLoadError::NotFound(format!(
                        "public module `{module_path}` in `{pkg_name}` missing interface stub `{}`",
                        rpi.display()
                    )));
                }
                let rpi_src = fs::read_to_string(&rpi).map_err(|e| {
                    PackageLoadError::Io(format!("failed to read `{}`: {e}", rpi.display()))
                })?;
                Some(parse_rpi_exports(&rpi_src)?)
            } else {
                None
            };

        let file = manifest.module_source_path(root, &module_path);
        let src = fs::read_to_string(&file).map_err(|e| {
            PackageLoadError::Io(format!(
                "failed to read module `{pkg_name}/{module_path}` at `{}`: {e}",
                file.display()
            ))
        })?;
        // Unit name matches the import path the dependent wrote (`graphics` or `graphics/shapes`).
        Ok(ResolvedImport {
            unit_name: import_path.to_string(),
            source: src,
            interface_exports,
            native_module_path: None,
        })
    }

    /// Build a path-dep lockfile for a consumer package root.
    ///
    /// Fills each path dep's `checksum` from `content_checksum(package.rpxm)`
    /// under that package's root (CS0; stub hash, not full tree).
    pub fn lock_consumer(
        &self,
        consumer: &PackageManifest,
    ) -> Result<crate::lockfile::Lockfile, PackageLoadError> {
        let mut dep_triples: Vec<(DependencySpec, PackageManifest, PathBuf)> = Vec::new();
        for spec in &consumer.dependencies {
            if spec.path.is_some() {
                let formal = self.resolve_alias(&spec.name);
                let (root, manifest) = self.packages.get(formal).ok_or_else(|| {
                    PackageLoadError::NotFound(format!(
                        "cannot lock missing path dependency `{}`",
                        spec.name
                    ))
                })?;
                dep_triples.push((spec.clone(), manifest.clone(), root.clone()));
                continue;
            }
            if spec.source.as_deref() == Some("registry") {
                let formal = self.resolve_alias(&spec.name);
                let (root, manifest) = self.packages.get(formal).ok_or_else(|| {
                    PackageLoadError::NotFound(format!(
                        "cannot lock missing registry dependency `{}`",
                        spec.name
                    ))
                })?;
                let mirror = LocalRegistryMirror::discover_near(root);
                if mirror.as_ref().is_some_and(|m| root.starts_with(m.root())) {
                    let mut reg_spec = spec.clone();
                    reg_spec.source = Some("registry".into());
                    dep_triples.push((reg_spec, manifest.clone(), root.clone()));
                }
            }
        }
        let refs: Vec<(&DependencySpec, &PackageManifest, Option<&Path>)> = dep_triples
            .iter()
            .map(|(s, m, r)| (s, m, Some(r.as_path())))
            .collect();
        Ok(crate::lockfile::Lockfile::from_consumer_with_roots(
            consumer, &refs,
        ))
    }
}

fn version_matches_exact(req: &str, version: &str) -> bool {
    req.trim_matches('"') == version.trim_matches('"')
}

fn split_package_import(import_path: &str) -> (&str, &str) {
    match import_path.split_once('/') {
        Some((pkg, rest)) => (pkg, rest),
        None => (import_path, ""),
    }
}

/// Load an entry `.rpx` (or directory) resolving sibling modules and local packages.
///
/// Resolution order for each `(import path)`:
/// 1. Sibling file next to the entry: `{entry_dir}/{path}.rpx`
/// 2. Local package index: first path segment = package name
pub fn load_module_tree_with_packages(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<Vec<(String, String)>, PackageLoadError> {
    let root = entry_path.as_ref();
    if root.is_dir() {
        // Directory mode stays sibling-only (same as bind::load_module_tree).
        return reciplexa_bind::load_module_tree(root).map_err(Into::into);
    }
    if !root.is_file() {
        return Err(PackageLoadError::Io(format!(
            "load_module_tree_with_packages: path not found `{}`",
            root.display()
        )));
    }
    let dir = entry_parent_dir(root);
    let entry_name = utf8_file_stem(root)?;

    let mut loaded: HashMap<String, String> = HashMap::new();
    let mut pending = vec![entry_name.clone()];
    let mut seen = HashSet::new();

    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let (unit_name, src) = if name == entry_name {
            let src = fs::read_to_string(root).map_err(|e| {
                PackageLoadError::Io(format!("failed to read `{}`: {e}", root.display()))
            })?;
            (entry_name.clone(), src)
        } else {
            load_unit_source(&name, &dir, index)?
        };
        let imports = parse_imports(&src)?;
        for imp in &imports {
            if imp.module == unit_name {
                return Err(PackageLoadError::Module(ModuleError {
                    message: format!("module `{unit_name}` cannot import itself"),
                }));
            }
            pending.push(imp.module.clone());
        }
        loaded.insert(unit_name, src);
    }

    let mut out = Vec::with_capacity(loaded.len());
    // Entry is always inserted before this point (pending starts with it).
    let entry_src = loaded
        .remove(&entry_name)
        .expect("entry unit loaded before result assembly");
    out.push((entry_name, entry_src));
    let mut rest: Vec<_> = loaded.into_iter().collect();
    rest.sort_by(|a, b| a.0.cmp(&b.0));
    out.extend(rest);
    Ok(out)
}

fn entry_parent_dir(root: &Path) -> PathBuf {
    root.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn utf8_file_stem(root: &Path) -> Result<String, PackageLoadError> {
    root.file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_string)
        .ok_or_else(|| PackageLoadError::Io("entry file must have a UTF-8 stem".into()))
}

fn load_unit_source(
    name: &str,
    entry_dir: &Path,
    index: &LocalPackageIndex,
) -> Result<(String, String), PackageLoadError> {
    let sibling = entry_dir.join(format!("{name}.rpx"));
    if sibling.is_file() {
        let src = fs::read_to_string(&sibling).map_err(|e| {
            PackageLoadError::Io(format!("failed to read `{}`: {e}", sibling.display()))
        })?;
        return Ok((name.to_string(), src));
    }
    index.resolve_import(name)
}

/// Convenience: load + elaborate with local package resolution.
///
/// Package modules that declare `interface-root` export only names listed in
/// their `.rpi` stub (MOD §8.4 / §9.2 light boundary).
pub fn elaborate_with_packages(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<Vec<ElaboratedUnit>, PackageLoadError> {
    let bind_map = DomainNativeBindMap::bind_stubs(index.native())
        .map_err(|e| PackageLoadError::Module(ModuleError { message: e }))?;
    elaborate_with_packages_bound(entry_path, index, &bind_map)
}

pub(crate) fn elaborate_with_packages_bound(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
    bind_map: &DomainNativeBindMap,
) -> Result<Vec<ElaboratedUnit>, PackageLoadError> {
    let entry_path = entry_path.as_ref();
    verify_package_lock_for_entry(entry_path)
        .map_err(|e| PackageLoadError::Io(format!("lock verify: {e}")))?;
    let loaded = load_module_tree_with_packages(entry_path, index)?;
    let mut interface_exports: HashMap<String, Vec<String>> = HashMap::new();
    let mut native_bodies: HashMap<String, CoreExpr> = HashMap::new();
    for (name, _) in &loaded {
        // Sibling units have no package interface; package imports do.
        let Ok(resolved) = index.resolve_import_detailed(name) else {
            continue;
        };
        if let Some(exports) = resolved.interface_exports {
            interface_exports.insert(name.clone(), exports);
        }
        let Some(native_path) = resolved.native_module_path.as_deref() else {
            continue;
        };
        if is_hybrid_override(index, native_path) {
            continue;
        }
        let Some(native) = index.native.get(native_path) else {
            continue;
        };
        if native.typed_exports.is_empty() {
            continue;
        }
        let expr = native
            .stub_core_expr(bind_map)
            .map_err(|e| PackageLoadError::Module(ModuleError { message: e }))?;
        native_bodies.insert(name.clone(), expr);
    }
    let units: Vec<(&str, UnitBody<'_>)> = loaded
        .iter()
        .map(|(name, src)| {
            if let Some(expr) = native_bodies.get(name) {
                (name.as_str(), UnitBody::Native(expr))
            } else {
                (name.as_str(), UnitBody::Source(src.as_str()))
            }
        })
        .collect();
    elaborate_units_with_bodies(&units, &interface_exports).map_err(Into::into)
}

fn is_hybrid_override(index: &LocalPackageIndex, module_path: &str) -> bool {
    index.hybrid_source_overrides.contains_key(module_path)
}

/// Elaborate `entry_path`, eval its entry unit, and auto-materialize `(resource …)`
/// records when an enclosing `package.rpxm` is known (PKG R3 load path).
pub fn eval_package_entry_main(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<RuntimeValue, PackageLoadError> {
    let entry_path = entry_path.as_ref();
    let bind_map = DomainNativeBindMap::bind_stubs(index.native())
        .map_err(|e| PackageLoadError::Module(ModuleError { message: e }))?;
    let units = elaborate_with_packages_bound(entry_path, index, &bind_map)?;
    let stem = utf8_file_stem(entry_path)?;
    let demo = units
        .iter()
        .find(|u| u.name == stem)
        .ok_or_else(|| PackageLoadError::NotFound(format!("missing elaborated unit `{stem}`")))?;
    let extra = build_domain_native_eval_env(&bind_map);
    let v = eval_expr_with_extra(&demo.expr, &primitive_env(), &extra, &mut UnitHost)
        .map_err(|e| PackageLoadError::Module(ModuleError { message: e.message }))?;
    Ok(maybe_materialize_package_resources_for_entry(
        &v, entry_path,
    ))
}

/// Evaluate an elaborated package unit with production DN2 BindingId slot bindings.
///
/// [`DomainNativeBindMap::bind_stubs`] is deterministic (BTreeMap export order), so a
/// map rebuilt here matches the IDs used by a prior [`elaborate_with_packages`].
pub fn eval_elaborated_package_expr(
    expr: &reciplexa_core::expr::CoreExpr,
    index: &LocalPackageIndex,
) -> Result<RuntimeValue, PackageLoadError> {
    let bind_map = DomainNativeBindMap::bind_stubs(index.native())
        .map_err(|e| PackageLoadError::Module(ModuleError { message: e }))?;
    let extra = build_domain_native_eval_env(&bind_map);
    eval_expr_with_extra(expr, &primitive_env(), &extra, &mut UnitHost)
        .map_err(|e| PackageLoadError::Module(ModuleError { message: e.message }))
}

#[cfg(test)]
mod eval_entry_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use reciplexa_eval::RuntimeValue;

    fn tmp_pkg(label: &str) -> (PathBuf, PathBuf) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.tmp")
            .join(format!("pkg-eval-entry-{label}-{nanos}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("resources/images")).unwrap();
        fs::write(root.join("resources/images/logo.png"), b"png").unwrap();
        fs::write(
            root.join("package.rpxm"),
            r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources "images/logo.png"))"#,
        )
        .unwrap();
        let entry = root.join("main.rpx");
        fs::write(&entry, r#"(val main (resource "images/logo.png"))"#).unwrap();
        (root, entry)
    }

    #[test]
    fn eval_package_entry_main_materializes_listed_resources() {
        let (_root, entry) = tmp_pkg("mat");
        let idx = LocalPackageIndex::default();
        let v = eval_package_entry_main(&entry, &idx).unwrap();
        let RuntimeValue::Record(fields) = v else {
            panic!("record, got {v:?}");
        };
        assert!(fields.iter().any(|(k, v)| {
            k == "resolved-path"
                && matches!(v, RuntimeValue::String(s) if s.replace('\\', "/").ends_with("images/logo.png"))
        }));
        assert!(fields.iter().any(|(k, v)| {
            k == "resource-id"
                && matches!(v, RuntimeValue::String(s) if s == "demo@1.0.0/images/logo.png")
        }));
        assert!(fields.iter().any(|(k, v)| {
            k == "content-hash" && matches!(v, RuntimeValue::String(s) if s.starts_with("sha256:"))
        }));
        assert!(fields.iter().any(|(k, v)| {
            k == "effect" && matches!(v, RuntimeValue::String(s) if s == "Resource")
        }));
    }
}

#[cfg(test)]
mod stem_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn entry_parent_dir_falls_back_when_no_parent() {
        assert_eq!(entry_parent_dir(Path::new("")), PathBuf::from("."));
        assert_eq!(entry_parent_dir(Path::new("main.rpx")), PathBuf::from(""));
    }

    #[test]
    fn utf8_file_stem_ok_and_missing() {
        assert_eq!(utf8_file_stem(Path::new("main.rpx")).unwrap(), "main");
        assert!(utf8_file_stem(Path::new(".")).is_err());
        assert!(utf8_file_stem(Path::new("..")).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn utf8_file_stem_rejects_lone_surrogate() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        let p = PathBuf::from(OsString::from_wide(&[0x0061, 0xD800]));
        assert!(utf8_file_stem(&p).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn load_rejects_non_utf8_entry_stem() {
        use std::ffi::OsString;
        use std::fs;
        use std::os::windows::ffi::OsStringExt;
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp/pkg-stem");
        let _ = fs::create_dir_all(&dir);
        // Filename body is a lone surrogate so `file_stem().to_str()` fails.
        let mut name = OsString::from_wide(&[0xD800]);
        name.push(".rpx");
        let path = dir.join(name);
        fs::write(&path, "(val main 1)\n").unwrap();
        let idx = LocalPackageIndex::default();
        let err = load_module_tree_with_packages(&path, &idx).unwrap_err();
        assert!(matches!(err, PackageLoadError::Io(_)));
        let _ = fs::remove_file(&path);
    }
}
