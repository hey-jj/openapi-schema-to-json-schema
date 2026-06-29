//! Conversion options and their resolution.

use serde_json::Value;
use std::sync::Arc;

use crate::consts::{NOT_SUPPORTED, STRUCTS};

/// A hook that runs on each node before keyword processing.
///
/// It receives the node and the resolved options and returns the node to
/// process. Returning a different value replaces the node.
pub type BeforeTransform = Arc<dyn Fn(Value, &ResolvedOptions) -> Value + Send + Sync>;

/// A hook that runs on each node after keyword processing.
///
/// Its return value becomes the converted node. At the root the return value
/// receives the `$schema` member.
pub type AfterTransform = Arc<dyn Fn(Value, &ResolvedOptions) -> Value + Send + Sync>;

/// A handler that rewrites a node after `x-patternProperties` becomes
/// `patternProperties`. Its return value replaces the node.
pub type PatternPropertiesHandler = Arc<dyn Fn(Value) -> Value + Send + Sync>;

/// Caller-facing options. Every field is optional. Unset fields take the
/// defaults described on [`ResolvedOptions`].
#[derive(Clone, Default)]
pub struct Options {
    /// Rewrite `format: "date"` to `format: "date-time"`. Default false.
    pub date_to_date_time: Option<bool>,
    /// Clone the input before converting so the input is never mutated.
    /// Default true. This crate owns its input, so the value affects only
    /// internal behavior, never an observable side effect on the caller.
    pub clone_schema: Option<bool>,
    /// Move `x-patternProperties` to `patternProperties` and run the handler.
    /// Default false.
    pub support_pattern_properties: Option<bool>,
    /// Keywords to keep that would otherwise be stripped. Each entry is removed
    /// from the strip list. Default empty.
    pub keep_not_supported: Option<Vec<String>>,
    /// Reject input `type` values outside the draft-04 type set. Default true.
    pub strict_mode: Option<bool>,
    /// Drop object properties marked `readOnly: true`. Default false.
    pub remove_read_only: Option<bool>,
    /// Drop object properties marked `writeOnly: true`. Default false.
    pub remove_write_only: Option<bool>,
    /// Replacement for the default `patternProperties` handler.
    pub pattern_properties_handler: Option<PatternPropertiesHandler>,
    /// Dotted paths whose values hold named sub-schemas to convert, for example
    /// `"definitions"` or `"schema.definitions"`. Default empty.
    pub definition_keywords: Option<Vec<String>>,
    /// Hook run on each node before keyword processing.
    pub before_transform: Option<BeforeTransform>,
    /// Hook run on each node after keyword processing.
    pub after_transform: Option<AfterTransform>,
}

impl Options {
    /// Construct empty options. Equivalent to [`Options::default`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// Options with defaults applied and internal fields derived.
///
/// Defaults follow the source library:
///
/// - `date_to_date_time` false, coerced with truthiness.
/// - `clone_schema` true, only `None` takes the default.
/// - `support_pattern_properties` false, coerced with truthiness.
/// - `keep_not_supported` empty.
/// - `strict_mode` true.
/// - `remove_read_only` / `remove_write_only` false.
/// - `definition_keywords` empty.
#[derive(Clone)]
pub struct ResolvedOptions {
    pub(crate) date_to_date_time: bool,
    pub(crate) support_pattern_properties: bool,
    pub(crate) strict_mode: bool,
    pub(crate) definition_keywords: Vec<String>,
    pub(crate) pattern_properties_handler: Option<PatternPropertiesHandler>,
    pub(crate) before_transform: Option<BeforeTransform>,
    pub(crate) after_transform: Option<AfterTransform>,
    /// Property flags that trigger property removal, in the order
    /// `readOnly` then `writeOnly`.
    pub(crate) remove_props: Vec<&'static str>,
    /// Keywords stripped after transform, in `NOT_SUPPORTED` order minus
    /// anything kept.
    pub(crate) not_supported: Vec<&'static str>,
}

/// Apply defaults and derive internal fields.
///
/// Mirrors `resolveOptions`. `date_to_date_time` and
/// `support_pattern_properties` use truthiness coercion. The rest use nullish
/// defaults, so an explicit `false` survives.
pub(crate) fn resolve_options(options: &Options) -> ResolvedOptions {
    let date_to_date_time = options.date_to_date_time.unwrap_or(false);
    // clone_schema controls input cloning in the source. This crate owns its
    // input, so conversion never mutates the caller's data and the flag has no
    // observable effect. The field stays on Options for API parity.
    let _clone_schema = options.clone_schema.unwrap_or(true);
    let support_pattern_properties = options.support_pattern_properties.unwrap_or(false);
    let keep_not_supported = options.keep_not_supported.clone().unwrap_or_default();
    let definition_keywords = options.definition_keywords.clone().unwrap_or_default();
    let strict_mode = options.strict_mode.unwrap_or(true);

    let mut remove_props = Vec::new();
    if options.remove_read_only.unwrap_or(false) {
        remove_props.push("readOnly");
    }
    if options.remove_write_only.unwrap_or(false) {
        remove_props.push("writeOnly");
    }

    let not_supported: Vec<&'static str> = NOT_SUPPORTED
        .iter()
        .copied()
        .filter(|kw| !keep_not_supported.iter().any(|k| k == kw))
        .collect();

    ResolvedOptions {
        date_to_date_time,
        support_pattern_properties,
        strict_mode,
        definition_keywords,
        pattern_properties_handler: options.pattern_properties_handler.clone(),
        before_transform: options.before_transform.clone(),
        after_transform: options.after_transform.clone(),
        remove_props,
        not_supported,
    }
}

impl ResolvedOptions {
    /// The struct keywords recursed during conversion.
    pub(crate) fn structs(&self) -> &'static [&'static str] {
        STRUCTS
    }
}
