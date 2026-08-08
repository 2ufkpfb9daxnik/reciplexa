//! Optional bridge from `reciplexa-syntax::ParseError` to structured diagnostics.

use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::resource::SourceResourceId;
use reciplexa_syntax::ParseError;

use crate::collector::DiagnosticCollector;
use crate::parse_adapter::{push_parse_errors, ParseDiagnosticInput};

/// Convert syntax crate parse errors into collector diagnostics.
pub fn push_syntax_parse_errors(
    collector: &mut DiagnosticCollector,
    package_instance_id: PackageInstanceId,
    module_id: ModuleId,
    source_resource_id: SourceResourceId,
    errors: &[ParseError],
) {
    let inputs: Vec<_> = errors
        .iter()
        .map(|e| ParseDiagnosticInput::new(e.message.clone(), e.start as u32, e.end as u32))
        .collect();
    push_parse_errors(
        collector,
        package_instance_id,
        module_id,
        source_resource_id,
        &inputs,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_syntax::parse_source;

    #[test]
    fn syntax_errors_bridge_to_diagnostics() {
        let parse = parse_source("(unclosed");
        let mut collector = DiagnosticCollector::new();
        push_syntax_parse_errors(
            &mut collector,
            PackageInstanceId::new(1),
            ModuleId::new(1),
            SourceResourceId::new(1),
            &parse.errors,
        );
        assert!(!parse.errors.is_empty());
        assert!(collector.has_errors());
    }
}
