use proc_macro2::Span;
use syn::LitStr;

#[derive(Default)]
pub struct FluvioCallableAttributes {
    pub min_version: Option<LitStr>,
    pub max_version: Option<LitStr>,
}
impl FluvioCallableAttributes {
    pub fn min_version(&self) -> Option<(String, Span)> {
        self.min_version.as_ref().map(|s| (s.value(), s.span()))
    }
    pub fn max_version(&self) -> Option<(String, Span)> {
        self.max_version.as_ref().map(|s| (s.value(), s.span()))
    }
}
macro_rules! parse_callable_attributes {
    ($input:ident, $attr_ident:literal) => {{
        let mut result = FluvioCallableAttributes::default();
        $crate::utils::parse_attributes!($input.attrs.iter(), $attr_ident, meta,
            "min_version", result.name => {
                meta.input.parse::<::syn::Token![=]>()?;
                let litstr: ::syn::LitStr = meta.input.parse()?;
                result.min_version = Some(litstr);
            }
            "max_version", result.abi => {
                meta.input.parse::<::syn::Token![=]>()?;
                let litstr: ::syn::LitStr = meta.input.parse()?;
                result.max_version = Some(litstr);
            }
        );
        result
    }};
}
pub(crate) use parse_callable_attributes;
