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
use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode};

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

/// `(import other)` or `(import other only (a b))`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDecl {
    pub module: String,
    pub only: Option<Vec<String>>,
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
            let export_names: Vec<String> = match &imp.only {
                Some(only) => only.clone(),
                None => {
                    let mut keys: Vec<_> = table.keys().cloned().collect();
                    keys.sort();
                    keys
                }
            };
            for export_name in export_names.into_iter().rev() {
                let value = table.get(&export_name).ok_or_else(|| {
                    ModuleError::new(format!(
                        "module `{name}` imports `{export_name}` from `{}`, but it is not exported",
                        imp.module
                    ))
                })?;
                linked = CoreExpr::Let {
                    name: export_name,
                    value: Box::new(value.clone()),
                    body: Box::new(linked),
                };
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
        return Err(ModuleError::new("`import` requires a module name"));
    }
    let AtomRef::Ident(module) = &atoms[1] else {
        return Err(ModuleError::new(
            "`import` module name must be an identifier",
        ));
    };
    if atoms.len() == 2 {
        return Ok(Some(ImportDecl {
            module: module.clone(),
            only: None,
        }));
    }
    // (import other only (a b))
    if atoms.len() != 4 {
        return Err(ModuleError::new(
            "`import` form is `(import name)` or `(import name only (a b …))`",
        ));
    }
    let AtomRef::Ident(only_kw) = &atoms[2] else {
        return Err(ModuleError::new("expected `only` in import form"));
    };
    if only_kw != "only" {
        return Err(ModuleError::new("expected `only` in import form"));
    }
    let AtomRef::Node(list) = &atoms[3] else {
        return Err(ModuleError::new("`import … only` requires a name list"));
    };
    let mut names = Vec::new();
    for a in list_idents_and_nodes(list) {
        match a {
            AtomRef::Ident(n) => names.push(n),
            AtomRef::Node(_) => {
                return Err(ModuleError::new(
                    "`import … only` list entries must be identifiers",
                ));
            }
        }
    }
    Ok(Some(ImportDecl {
        module: module.clone(),
        only: Some(names),
    }))
}

enum AtomRef {
    Ident(String),
    Node(SyntaxNode),
}

fn list_idents_and_nodes(node: &SyntaxNode) -> Vec<AtomRef> {
    let mut items = Vec::new();
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
                    items.push(AtomRef::Ident(t.text().to_string()));
                }
            }
            SyntaxElement::Node(n) => {
                if n.kind() != SyntaxKind::StructuredComment {
                    items.push(AtomRef::Node(n));
                }
            }
        }
    }
    items
}

fn collect_bindings(expr: &CoreExpr) -> HashMap<String, CoreExpr> {
    let mut map = HashMap::new();
    let mut cur = expr;
    while let CoreExpr::Let { name, value, body } = cur {
        map.insert(name.clone(), *value.clone());
        cur = body;
    }
    map
}

fn collect_export_names(expr: &CoreExpr) -> Vec<String> {
    let mut names = Vec::new();
    let mut cur = expr;
    while let CoreExpr::Let { name, body, .. } = cur {
        names.push(name.clone());
        cur = body;
    }
    names
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
        let src = fs::read_to_string(&path).map_err(|e| {
            ModuleError::new(format!("failed to read `{}`: {e}", path.display()))
        })?;
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
