//! Conversion options and their resolution.

use serde_json::Value;
use std::fmt;
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
///
/// Two ways to set options. Use struct-update syntax against [`Options::new`]:
///
/// ```
/// # use openapi_schema_to_json_schema::Options;
/// let options = Options {
///     support_pattern_properties: Some(true),
///     ..Options::new()
/// };
/// ```
///
/// Or chain the setters, which take plain values and wrap them for you:
///
/// ```
/// # use openapi_schema_to_json_schema::Options;
/// let options = Options::new()
///     .support_pattern_properties(true)
///     .strict_mode(false);
/// ```
#[derive(Clone, Default)]
pub struct Options {
    /// Rewrite `format: "date"` to `format: "date-time"`. Default false.
    pub date_to_date_time: Option<bool>,
    /// Accepted for compatibility and ignored. Conversion takes the input by
    /// value and never mutates the caller's data, so the output is the same
    /// whether this is set or unset. Default true.
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

    /// Set `date_to_date_time`. See the field for meaning.
    pub fn date_to_date_time(mut self, value: bool) -> Self {
        self.date_to_date_time = Some(value);
        self
    }

    /// Set `clone_schema`. Accepted for compatibility and has no effect.
    pub fn clone_schema(mut self, value: bool) -> Self {
        self.clone_schema = Some(value);
        self
    }

    /// Set `support_pattern_properties`. See the field for meaning.
    pub fn support_pattern_properties(mut self, value: bool) -> Self {
        self.support_pattern_properties = Some(value);
        self
    }

    /// Set `keep_not_supported`. See the field for meaning.
    pub fn keep_not_supported(mut self, value: Vec<String>) -> Self {
        self.keep_not_supported = Some(value);
        self
    }

    /// Set `strict_mode`. See the field for meaning.
    pub fn strict_mode(mut self, value: bool) -> Self {
        self.strict_mode = Some(value);
        self
    }

    /// Set `remove_read_only`. See the field for meaning.
    pub fn remove_read_only(mut self, value: bool) -> Self {
        self.remove_read_only = Some(value);
        self
    }

    /// Set `remove_write_only`. See the field for meaning.
    pub fn remove_write_only(mut self, value: bool) -> Self {
        self.remove_write_only = Some(value);
        self
    }

    /// Set `definition_keywords`. See the field for meaning.
    pub fn definition_keywords(mut self, value: Vec<String>) -> Self {
        self.definition_keywords = Some(value);
        self
    }

    /// Set the `patternProperties` handler from a plain closure. The closure is
    /// boxed for you, so callers do not write `Arc::new`.
    pub fn pattern_properties_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(Value) -> Value + Send + Sync + 'static,
    {
        self.pattern_properties_handler = Some(Arc::new(handler));
        self
    }

    /// Set the before-transform hook from a plain closure. The closure is boxed
    /// for you.
    pub fn before_transform<F>(mut self, hook: F) -> Self
    where
        F: Fn(Value, &ResolvedOptions) -> Value + Send + Sync + 'static,
    {
        self.before_transform = Some(Arc::new(hook));
        self
    }

    /// Set the after-transform hook from a plain closure. The closure is boxed
    /// for you.
    pub fn after_transform<F>(mut self, hook: F) -> Self
    where
        F: Fn(Value, &ResolvedOptions) -> Value + Send + Sync + 'static,
    {
        self.after_transform = Some(Arc::new(hook));
        self
    }
}

impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Options")
            .field("date_to_date_time", &self.date_to_date_time)
            .field("clone_schema", &self.clone_schema)
            .field(
                "support_pattern_properties",
                &self.support_pattern_properties,
            )
            .field("keep_not_supported", &self.keep_not_supported)
            .field("strict_mode", &self.strict_mode)
            .field("remove_read_only", &self.remove_read_only)
            .field("remove_write_only", &self.remove_write_only)
            .field(
                "pattern_properties_handler",
                &closure_field(self.pattern_properties_handler.is_some()),
            )
            .field("definition_keywords", &self.definition_keywords)
            .field(
                "before_transform",
                &closure_field(self.before_transform.is_some()),
            )
            .field(
                "after_transform",
                &closure_field(self.after_transform.is_some()),
            )
            .finish()
    }
}

/// Render a closure-bearing field as set or unset for `Debug`.
fn closure_field(set: bool) -> &'static str {
    if set {
        "Some(<closure>)"
    } else {
        "None"
    }
}

/// Options with defaults applied and internal fields derived.
///
/// The crate builds this from [`Options`] and passes a reference to the
/// `before_transform` and `after_transform` hooks. A hook can read the resolved
/// settings through the accessors. The fields stay private so the shape can
/// change without breaking callers.
///
/// Defaults:
///
/// - `date_to_date_time` false, coerced with truthiness.
/// - `support_pattern_properties` false, coerced with truthiness.
/// - `strict_mode` true.
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
    pub(crate) const STRUCTS: &'static [&'static str] = STRUCTS;

    /// Whether `format: "date"` is rewritten to `format: "date-time"`.
    pub fn date_to_date_time(&self) -> bool {
        self.date_to_date_time
    }

    /// Whether `x-patternProperties` is moved to `patternProperties`.
    pub fn support_pattern_properties(&self) -> bool {
        self.support_pattern_properties
    }

    /// Whether an invalid input `type` is rejected.
    pub fn strict_mode(&self) -> bool {
        self.strict_mode
    }

    /// The dotted paths whose values hold named sub-schemas to convert.
    pub fn definition_keywords(&self) -> &[String] {
        &self.definition_keywords
    }
}

impl fmt::Debug for ResolvedOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedOptions")
            .field("date_to_date_time", &self.date_to_date_time)
            .field(
                "support_pattern_properties",
                &self.support_pattern_properties,
            )
            .field("strict_mode", &self.strict_mode)
            .field("definition_keywords", &self.definition_keywords)
            .field(
                "pattern_properties_handler",
                &closure_field(self.pattern_properties_handler.is_some()),
            )
            .field(
                "before_transform",
                &closure_field(self.before_transform.is_some()),
            )
            .field(
                "after_transform",
                &closure_field(self.after_transform.is_some()),
            )
            .field("remove_props", &self.remove_props)
            .field("not_supported", &self.not_supported)
            .finish()
    }
}
