//! Module skeleton for multi-unit programs (MOD-001).
//!
//! Outer module = one source unit. `(import other)` / `(import other only (a b))`
//! are resolved via [`elaborate_units`] (in-memory) or [`load_module_tree`]
//! (filesystem sibling `.rpx` files).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use reciplexa_core::elaborate::{elaborate_source, ElaborateError};
use reciplexa_core::expr::CoreExpr;
use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::resource::SourceResourceId;
use reciplexa_syntax::{
    coalesce_slash_paths, parse_source, SlashAtom, SyntaxElement, SyntaxKind, SyntaxNode,
};

/// A single compilable unit within a package instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleUnit {
    pub module_id: ModuleId,
    pub source_resource_id: SourceResourceId,
    pub name: String,
}

/// Minimal module graph container.
#[derive(Debug, Clone)]
pub struct ModuleSkeleton {
    pub package_instance_id: PackageInstanceId,
    pub units: Vec<ModuleUnit>,
    pub entry: Option<ModuleId>,
}

impl ModuleSkeleton {
    pub fn new(package_instance_id: PackageInstanceId) -> Self {
        Self {
            package_instance_id,
            units: Vec::new(),
            entry: None,
        }
    }

    pub fn add_unit(&mut self, unit: ModuleUnit) {
        if self.entry.is_none() {
            self.entry = Some(unit.module_id);
        }
        self.units.push(unit);
    }

    pub fn find(&self, module_id: ModuleId) -> Option<&ModuleUnit> {
        self.units.iter().find(|u| u.module_id == module_id)
    }
}

/// One selective-import item: `name` or `name as local-name` (MOD-001 §6.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportItem {
    pub name: String,
    pub rename: Option<String>,
}

/// `(import path)`, `(import path as alias)`, `(import path only a b)`, …
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDecl {
    /// Module path (`lib` or `graphics/color`).
    pub module: String,
    /// Optional module alias from `as name` (MOD-001 §6.2).
    pub alias: Option<String>,
    /// Selective imports: flat `only a b` / `only a as b` or legacy `only (a b)`.
    pub only: Option<Vec<ImportItem>>,
}

/// One elaborated outer module unit.
#[derive(Debug, Clone, PartialEq)]
pub struct ElaboratedUnit {
    pub name: String,
    pub imports: Vec<ImportDecl>,
    /// Core expression for this unit with imports linked as outer `let`s.
    pub expr: CoreExpr,
    /// Top-level binding names defined in this unit (before import linking).
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleError {
    pub message: String,
}

impl ModuleError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<ElaborateError> for ModuleError {
    fn from(e: ElaborateError) -> Self {
        Self { message: e.message }
    }
}

/// Elaborate multiple in-memory units and link `(import …)` skeletons.
///
/// Each `(name, src)` pair is one outer module. Imports refer to sibling names
/// in the same slice (no path IO). Prefer [`load_module_tree`] to read `.rpx`
/// files from disk first.
pub fn elaborate_units(units: &[(&str, &str)]) -> Result<Vec<ElaboratedUnit>, ModuleError> {
    if units.is_empty() {
        return Err(ModuleError::new(
            "elaborate_units requires at least one unit",
        ));
    }

    let mut names = HashMap::new();
    for (name, _) in units {
        if names.insert((*name).to_string(), ()).is_some() {
            return Err(ModuleError::new(format!("duplicate module name `{name}`")));
        }
    }

    let mut parsed: Vec<(String, Vec<ImportDecl>, CoreExpr, Vec<String>)> = Vec::new();
    for (name, src) in units {
        let (imports, body_src) = split_imports(src)?;
        for imp in &imports {
            if !names.contains_key(&imp.module) {
                return Err(ModuleError::new(format!(
                    "module `{name}` imports unknown unit `{}`",
                    imp.module
                )));
            }
            if imp.module == *name {
                return Err(ModuleError::new(format!(
                    "module `{name}` cannot import itself"
                )));
            }
        }
        // MOD-001 §7.4: same identity via multiple imports is ok; colliding
        // local names from different module identities are errors.
        check_import_local_collisions(name, &imports)?;
        let expr = if body_src.trim().is_empty() {
            CoreExpr::Seq(vec![])
        } else {
            elaborate_source(&body_src)?
        };
        let exports = collect_export_names(&expr);
        parsed.push(((*name).to_string(), imports, expr, exports));
    }

    let binding_tables: HashMap<String, HashMap<String, CoreExpr>> = parsed
        .iter()
        .map(|(name, _, expr, _)| (name.clone(), collect_bindings(expr)))
        .collect();

    let mut out = Vec::with_capacity(parsed.len());
    for (name, imports, expr, exports) in parsed {
        let mut linked = expr;
        for imp in imports.iter().rev() {
            let table = binding_tables.get(&imp.module).ok_or_else(|| {
                ModuleError::new(format!("missing unit `{}` during link", imp.module))
            })?;

            // Bare names from `only` (§6.3–6.4), applied innermost so they
            // shadow any same-named qualified bindings from outer wraps.
            if let Some(items) = &imp.only {
                for item in items.iter().rev() {
                    let value = table.get(&item.name).ok_or_else(|| {
                        ModuleError::new(format!(
                            "module `{name}` imports `{}` from `{}`, but it is not exported",
                            item.name, imp.module
                        ))
                    })?;
                    let local = item.rename.clone().unwrap_or_else(|| item.name.clone());
                    linked = CoreExpr::Let {
                        name: local,
                        value: Box::new(value.clone()),
                        body: Box::new(linked),
                    };
                }
            }

            // Qualified refs (§6.1–6.2, §6.5): `prefix/export` for every export
            // when there is a module alias, or when `only` is absent (formal path).
            if imp.alias.is_some() || imp.only.is_none() {
                let prefix = imp.alias.as_deref().unwrap_or(imp.module.as_str());
                let mut keys: Vec<_> = table.keys().cloned().collect();
                keys.sort();
                for export_name in keys.into_iter().rev() {
                    let value = table.get(&export_name).expect("key from table");
                    linked = CoreExpr::Let {
                        name: format!("{prefix}/{export_name}"),
                        value: Box::new(value.clone()),
                        body: Box::new(linked),
                    };
                }
            }
        }
        out.push(ElaboratedUnit {
            name,
            imports,
            expr: linked,
            exports,
        });
    }
    Ok(out)
}

/// MOD-001 §7.4: same formal identity may be imported multiple times; distinct
/// identities that introduce the same local name collide.
fn check_import_local_collisions(unit: &str, imports: &[ImportDecl]) -> Result<(), ModuleError> {
    let mut bare: HashMap<String, String> = HashMap::new();
    let mut prefixes: HashMap<String, String> = HashMap::new();
    for imp in imports {
        if let Some(items) = &imp.only {
            for item in items {
                let local = item.rename.clone().unwrap_or_else(|| item.name.clone());
                if let Some(prev) = bare.get(&local) {
                    if prev != &imp.module {
                        return Err(ModuleError::new(format!(
                            "module `{unit}`: local name `{local}` imported from distinct modules `{prev}` and `{}`",
                            imp.module
                        )));
                    }
                } else {
                    bare.insert(local, imp.module.clone());
                }
            }
        }
        if imp.alias.is_some() || imp.only.is_none() {
            let prefix = imp.alias.clone().unwrap_or_else(|| imp.module.clone());
            if let Some(prev) = prefixes.get(&prefix) {
                if prev != &imp.module {
                    return Err(ModuleError::new(format!(
                        "module `{unit}`: import prefix `{prefix}` maps to distinct modules `{prev}` and `{}`",
                        imp.module
                    )));
                }
            } else {
                prefixes.insert(prefix, imp.module.clone());
            }
        }
    }
    Ok(())
}

/// Parse leading `(import …)` forms from a source unit (body discarded).
pub fn parse_imports(src: &str) -> Result<Vec<ImportDecl>, ModuleError> {
    let (imports, _) = split_imports(src)?;
    Ok(imports)
}

fn split_imports(src: &str) -> Result<(Vec<ImportDecl>, String), ModuleError> {
    let parse = parse_source(src);
    if let Some(err) = parse.errors.first() {
        return Err(ModuleError::new(format!("parse error: {}", err.message)));
    }
    let mut imports = Vec::new();
    let mut body_parts = Vec::new();
    for el in parse.root.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                body_parts.push(t.text().to_string());
            }
            SyntaxElement::Node(n) => {
                if n.kind() == SyntaxKind::StructuredComment {
                    continue;
                }
                if n.kind() == SyntaxKind::List {
                    if let Some(imp) = parse_import_list(&n)? {
                        imports.push(imp);
                        continue;
                    }
                }
                let range = n.text_range();
                let start: usize = range.start().into();
                let end: usize = range.end().into();
                body_parts.push(src[start..end].to_string());
                body_parts.push("\n".into());
            }
        }
    }
    Ok((imports, body_parts.concat()))
}

fn parse_import_list(node: &SyntaxNode) -> Result<Option<ImportDecl>, ModuleError> {
    let atoms = list_idents_and_nodes(node);
    let Some(head) = atoms.first() else {
        return Ok(None);
    };
    let AtomRef::Ident(h) = head else {
        return Ok(None);
    };
    if h != "import" {
        return Ok(None);
    }
    if atoms.len() < 2 {
        return Err(ModuleError::new("`import` requires a module path"));
    }
    let AtomRef::Ident(module) = &atoms[1] else {
        return Err(ModuleError::new(
            "`import` module path must be an identifier (segments joined by `/`)",
        ));
    };

    let mut alias = None;
    let mut only = None;
    let mut i = 2;
    while i < atoms.len() {
        match &atoms[i] {
            AtomRef::Ident(kw) if kw == "as" => {
                if alias.is_some() {
                    return Err(ModuleError::new("`import` has duplicate `as` clause"));
                }
                i += 1;
                let Some(AtomRef::Ident(name)) = atoms.get(i) else {
                    return Err(ModuleError::new(
                        "`import … as` requires an alias identifier",
                    ));
                };
                alias = Some(name.clone());
                i += 1;
            }
            AtomRef::Ident(kw) if kw == "only" => {
                if only.is_some() {
                    return Err(ModuleError::new("`import` has duplicate `only` clause"));
                }
                i += 1;
                if i >= atoms.len() {
                    return Err(ModuleError::new(
                        "`import … only` requires at least one name",
                    ));
                }
                // Legacy: `(import m only (a b))`
                if let AtomRef::Node(list) = &atoms[i] {
                    let mut names = Vec::new();
                    for a in list_idents_and_nodes(list) {
                        match a {
                            AtomRef::Ident(n) => names.push(ImportItem {
                                name: n,
                                rename: None,
                            }),
                            AtomRef::Node(_) => {
                                return Err(ModuleError::new(
                                    "`import … only` list entries must be identifiers",
                                ));
                            }
                        }
                    }
                    only = Some(names);
                    i += 1;
                } else {
                    // Flat: `(import m only a b)` / `only a as b` (MOD-001 §6.3–6.6)
                    let mut names = Vec::new();
                    while i < atoms.len() {
                        match &atoms[i] {
                            AtomRef::Ident(n) if n != "as" && n != "only" => {
                                let name = n.clone();
                                i += 1;
                                let rename = if matches!(atoms.get(i), Some(AtomRef::Ident(kw)) if kw == "as")
                                {
                                    i += 1;
                                    let Some(AtomRef::Ident(local)) = atoms.get(i) else {
                                        return Err(ModuleError::new(
                                            "`import … only name as` requires a local name",
                                        ));
                                    };
                                    i += 1;
                                    Some(local.clone())
                                } else {
                                    None
                                };
                                names.push(ImportItem { name, rename });
                            }
                            AtomRef::Ident(_) => break,
                            AtomRef::Node(_) => {
                                return Err(ModuleError::new(
                                    "`import … only` names must be identifiers",
                                ));
                            }
                        }
                    }
                    if names.is_empty() {
                        return Err(ModuleError::new(
                            "`import … only` requires at least one name",
                        ));
                    }
                    only = Some(names);
                }
            }
            AtomRef::Ident(other) => {
                return Err(ModuleError::new(format!(
                    "unexpected `{other}` in import; expected `as` or `only`"
                )));
            }
            AtomRef::Node(_) => {
                return Err(ModuleError::new(
                    "`import` form is `(import path)`, `(import path as alias)`, \
                     or `(import path only names…)`",
                ));
            }
        }
    }

    Ok(Some(ImportDecl {
        module: module.clone(),
        alias,
        only,
    }))
}

enum AtomRef {
    Ident(String),
    Node(SyntaxNode),
}

fn list_idents_and_nodes(node: &SyntaxNode) -> Vec<AtomRef> {
    let mut raw = Vec::new();
    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind().is_trivia()
                    || matches!(
                        t.kind(),
                        SyntaxKind::LParen
                            | SyntaxKind::RParen
                            | SyntaxKind::LBracket
                            | SyntaxKind::RBracket
                    )
                {
                    continue;
                }
                if t.kind() == SyntaxKind::Ident {
                    raw.push(AtomRef::Ident(t.text().to_string()));
                }
            }
            SyntaxElement::Node(n) => {
                if n.kind() != SyntaxKind::StructuredComment {
                    raw.push(AtomRef::Node(n));
                }
            }
        }
    }
    coalesce_slash_paths(raw, |a| match a {
        AtomRef::Ident(s) => Some(s.as_str()),
        AtomRef::Node(_) => None,
    })
    .into_iter()
    .map(|a| match a {
        SlashAtom::Path(p) => AtomRef::Ident(p),
        SlashAtom::Item(item) => item,
    })
    .collect()
}

fn collect_bindings(expr: &CoreExpr) -> HashMap<String, CoreExpr> {
    let mut map = HashMap::new();
    collect_bindings_into(expr, &mut map);
    map
}

fn collect_bindings_into(expr: &CoreExpr, map: &mut HashMap<String, CoreExpr>) {
    match expr {
        CoreExpr::Let { name, value, body } => {
            map.insert(name.clone(), *value.clone());
            collect_bindings_into(body, map);
        }
        CoreExpr::LetRec { bindings, body } => {
            for (name, value) in bindings {
                map.insert(name.clone(), value.clone());
            }
            collect_bindings_into(body, map);
        }
        _ => {}
    }
}

fn collect_export_names(expr: &CoreExpr) -> Vec<String> {
    let mut names = Vec::new();
    collect_export_names_into(expr, &mut names);
    names
}

fn collect_export_names_into(expr: &CoreExpr, names: &mut Vec<String>) {
    match expr {
        CoreExpr::Let { name, body, .. } => {
            names.push(name.clone());
            collect_export_names_into(body, names);
        }
        CoreExpr::LetRec { bindings, body } => {
            for (name, _) in bindings {
                names.push(name.clone());
            }
            collect_export_names_into(body, names);
        }
        _ => {}
    }
}

/// Load a module tree from the filesystem (sibling `.rpx` imports).
///
/// - If `root_path` is a **directory**, every `*.rpx` file becomes a unit
///   (module name = file stem).
/// - If `root_path` is a **`.rpx` file**, that unit is the entry; `(import foo)`
///   loads `./foo.rpx` relative to the entry's directory (transitively).
///
/// Returns `(module_name, source)` pairs suitable for [`elaborate_units`].
pub fn load_module_tree(root_path: impl AsRef<Path>) -> Result<Vec<(String, String)>, ModuleError> {
    let root = root_path.as_ref();
    if root.is_dir() {
        return load_directory_units(root);
    }
    if !root.is_file() {
        return Err(ModuleError::new(format!(
            "load_module_tree: path not found `{}`",
            root.display()
        )));
    }
    let dir = root.parent().unwrap_or_else(|| Path::new("."));
    let entry_name = root
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ModuleError::new("entry file must have a UTF-8 stem"))?
        .to_string();
    let mut loaded: HashMap<String, String> = HashMap::new();
    let mut pending = vec![entry_name.clone()];
    let mut seen = HashSet::new();
    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let path = dir.join(format!("{name}.rpx"));
        let src = fs::read_to_string(&path).map_err(|e| {
            ModuleError::new(format!(
                "failed to read module `{name}` at `{}`: {e}",
                path.display()
            ))
        })?;
        let (imports, _) = split_imports(&src)?;
        for imp in &imports {
            if imp.module == name {
                return Err(ModuleError::new(format!(
                    "module `{name}` cannot import itself"
                )));
            }
            pending.push(imp.module.clone());
        }
        loaded.insert(name, src);
    }
    // Stable order: entry first, then remaining sorted by name.
    let mut out = Vec::with_capacity(loaded.len());
    if let Some(src) = loaded.remove(&entry_name) {
        out.push((entry_name, src));
    }
    let mut rest: Vec<_> = loaded.into_iter().collect();
    rest.sort_by(|a, b| a.0.cmp(&b.0));
    out.extend(rest);
    Ok(out)
}

fn load_directory_units(dir: &Path) -> Result<Vec<(String, String)>, ModuleError> {
    let mut units = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| {
        ModuleError::new(format!(
            "failed to read module directory `{}`: {e}",
            dir.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| ModuleError::new(format!("read_dir entry: {e}")))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rpx") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ModuleError::new(format!("non-UTF-8 module path `{}`", path.display())))?
            .to_string();
        let src = fs::read_to_string(&path)
            .map_err(|e| ModuleError::new(format!("failed to read `{}`: {e}", path.display())))?;
        units.push((name, src));
    }
    if units.is_empty() {
        return Err(ModuleError::new(format!(
            "no `.rpx` modules in `{}`",
            dir.display()
        )));
    }
    units.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(units)
}

/// Convenience: [`load_module_tree`] then [`elaborate_units`].
pub fn elaborate_module_tree(
    root_path: impl AsRef<Path>,
) -> Result<Vec<ElaboratedUnit>, ModuleError> {
    let loaded = load_module_tree(root_path)?;
    let refs: Vec<(&str, &str)> = loaded
        .iter()
        .map(|(n, s)| (n.as_str(), s.as_str()))
        .collect();
    elaborate_units(&refs)
}
