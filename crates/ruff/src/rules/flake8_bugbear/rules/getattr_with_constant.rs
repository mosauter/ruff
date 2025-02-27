use ruff_text_size::TextSize;
use rustpython_parser::ast::{Constant, Expr, ExprContext, ExprKind};

use ruff_diagnostics::{AlwaysAutofixableViolation, Diagnostic, Edit};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::helpers::unparse_expr;
use ruff_python_stdlib::identifiers::{is_identifier, is_mangled_private};

use crate::checkers::ast::Checker;
use crate::registry::AsRule;

#[violation]
pub struct GetAttrWithConstant;

impl AlwaysAutofixableViolation for GetAttrWithConstant {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
            "Do not call `getattr` with a constant attribute value. It is not any safer than \
             normal property access."
        )
    }

    fn autofix_title(&self) -> String {
        "Replace `getattr` with attribute access".to_string()
    }
}
fn attribute(value: &Expr, attr: &str) -> Expr {
    Expr::new(
        TextSize::default(),
        TextSize::default(),
        ExprKind::Attribute {
            value: Box::new(value.clone()),
            attr: attr.to_string(),
            ctx: ExprContext::Load,
        },
    )
}

/// B009
pub fn getattr_with_constant(checker: &mut Checker, expr: &Expr, func: &Expr, args: &[Expr]) {
    let ExprKind::Name { id, .. } = &func.node else {
        return;
    };
    if id != "getattr" {
        return;
    }
    let [obj, arg] = args else {
        return;
    };
    let ExprKind::Constant {
        value: Constant::Str(value),
        ..
    } = &arg.node else {
        return;
    };
    if !is_identifier(value) {
        return;
    }
    if is_mangled_private(value.as_str()) {
        return;
    }

    let mut diagnostic = Diagnostic::new(GetAttrWithConstant, expr.range());

    if checker.patch(diagnostic.kind.rule()) {
        diagnostic.set_fix(Edit::range_replacement(
            unparse_expr(&attribute(obj, value), checker.stylist),
            expr.range(),
        ));
    }
    checker.diagnostics.push(diagnostic);
}
