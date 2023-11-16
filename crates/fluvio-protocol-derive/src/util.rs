use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Attribute, Expr, ItemFn, Lit, LitStr, Meta, MetaList,
    MetaNameValue, Token, parse_quote_spanned,
};

use crate::ast::prop::PropAttrsType;

/// Parses the specified attributes from a `syn::Attribute` iterator.
macro_rules! parse_attributes {
    ($attrs:expr, $attr_ident:literal, $meta:ident, $($field:pat, $opt:expr => $block:expr)*) => {
        const ERROR: &str = concat!("unrecognized ", $attr_ident, " attribute");
        const ALREADY_SPECIFIED: &str = concat!($attr_ident, " attribute already specified");

        for attr in $attrs {
            if !attr.path().is_ident($attr_ident) {
                continue;
            }

            attr.parse_nested_meta(|$meta| {

                let ident = $meta.path.get_ident().ok_or_else(|| $meta.error(ERROR))?;
                let attr_name = &ident.to_string();
                let attr_span = attr.span();
       
                match attr_name.as_str() {
                    $(
                        // check if the value is already defined
                        // if not we can execute the block of code
                        $field if $opt.is_none() => { 
                            if let Ok(value) = $meta.value() {
                                let expr: Option<Expr> = Some(value.parse()?);
                                dbg!(&attr_span);
                                return $block(expr, attr_span, attr_name)
                            } else {
                                return $block(None, attr_span, attr_name)
                            }
                        },
                        $field => {return Err($meta.error(ALREADY_SPECIFIED))}
                    )*

                    _ => return Err($meta.error(ERROR)),
                }
            })?;
        }
    };
}
pub(crate) use parse_attributes;

pub fn get_expr_value<'a>(attr_name: &'a str, field: &'a Option<Expr>, span: Span) -> syn::Result<PropAttrsType> {
  
    match &field {
        Some(Expr::Lit(lit_expr)) => {
            if let Lit::Int(lit) = &lit_expr.lit {
                Ok(PropAttrsType::LitInt(lit.base10_parse::<i16>()?))
            } else if let Lit::Str(lit) = &lit_expr.lit {
                let value = &lit.value();

                if value.contains("(") && value.contains(")") {
                    if let Some(value) = &lit.value().strip_suffix("()") {
                        return Ok(PropAttrsType::LitFn(syn::Ident::new(
                            value,
                            span,
                        )));
                    }
                }

                Ok(PropAttrsType::LitStr(syn::Ident::new(
                    &lit.value(),
                    span,
                )))
            } else {
                Err(syn::Error::new(
                    span,
                    format!("Expected {attr_name} attribute to be a LitStr or LitInt: `{attr_name} = \"...\"`"),
                ))
            }
        }
        _ => Err(syn::Error::new(
            span,
            format!("Expected {attr_name} attribute to be a Lit: `{attr_name} = \"...\"`"),
        )),
    }
}

pub fn get_lit_int<'a>(attr_name: &'a str, value: &'a Option<Expr>, span: Span) -> syn::Result<&'a syn::LitInt> {
    match &value {
        Some(Expr::Lit(lit_expr)) => {
            if let Lit::Int(lit) = &lit_expr.lit {
                Ok(lit)
            } else {
                Err(syn::Error::new(
                    span,
                    format!("Expected {attr_name} attribute to be an int: `{attr_name} = \"...\"`"),
                ))
            }
        }
        _ => Err(syn::Error::new(
            span,
            format!("Expected {attr_name} attribute to be a Lit: `{attr_name} = \"...\"`"),
        )),
    }
}
pub fn get_lit_str<'a>(attr_name: &'a str, value: &'a Option<Expr>, span: Span) -> syn::Result<&'a syn::LitStr> {
    match &value {
        Some(Expr::Lit(lit_expr)) => {
            if let Lit::Str(lit) = &lit_expr.lit {
                Ok(lit)
            } else {
                Err(syn::Error::new(
                    span,
                    format!(
                        "Expected {attr_name} attribute to be a string: `{attr_name} = \"...\"`"
                    ),
                ))
            }
        }
        _ => Err(syn::Error::new(
            span,
            format!("Expected {attr_name} attribute to be a Lit: `{attr_name} = \"...\"`"),
        )),
    }
}

pub(crate) fn find_attr(attrs: &[Attribute], name: &str) -> Option<Meta> {
    attrs.iter().find_map(|a| {
        if a.meta.path().is_ident(name) {
            Some(a.meta.clone())
        } else {
            None
        }
    })
}
pub(crate) fn find_name_value_from_meta_list(
    meta_list: &MetaList,
    attr_name: &str,
) -> Option<MetaNameValue> {
    for item in meta_list
        .parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)
        .unwrap_or_else(|err| panic!("Invalid #[new] attribute: {}", err))
    {
        if let Meta::NameValue(kv) = &item {
            if kv.path.is_ident(attr_name) {
                return Some(kv.clone());
            }
        };
    }
    None
}

pub(crate) fn find_name_value_from_meta(meta: &Meta, attr_name: &str) -> Option<MetaNameValue> {
    if let Meta::List(list) = meta {
        return find_name_value_from_meta_list(list, attr_name);
    }
    None
}

/// find name value with str value
pub(crate) fn find_string_name_value(meta: &Meta, attr_name: &str) -> Option<LitStr> {
    let meta_name_value = find_name_value_from_meta(meta, attr_name)?;
    get_lit_str(attr_name, &Some(meta_name_value.value), Span::call_site()).ok().cloned()
}

pub(crate) fn find_int_name_value(meta: &Meta, attr_name: &str) -> Option<u64> {
    let meta_name_value = find_name_value_from_meta(meta, attr_name)?;
    let value = get_lit_int(attr_name, &Some(meta_name_value.value), Span::call_site())
        .ok()
        .cloned()?;
    value.base10_parse::<u64>().ok()
}
