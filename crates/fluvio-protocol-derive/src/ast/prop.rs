use proc_macro::Span;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{token, Attribute, Error, Expr, Field, LitInt, LitStr, Meta, Token, Type, parse_quote_spanned, Lit, parse_quote};
use tracing::Instrument;

use crate::util::{get_expr_value, get_lit_int, get_lit_str, parse_attributes};

#[derive(Debug, Clone)]
pub(crate) struct NamedProp {
    pub field_name: String,
    pub field_type: Type,
    pub attrs: PropAttrs,
}

#[derive(Clone, Debug)]
pub(crate) struct UnnamedProp {
    pub field_type: Type,
    pub attrs: PropAttrs,
}

impl NamedProp {
    pub fn from_ast(field: &Field) -> syn::Result<Self> {
        let field_ident = if let Some(ident) = &field.ident {
            ident.clone()
        } else {
            return Err(Error::new(
                field.span(),
                "Named field must have an `ident`.",
            ));
        };
        let field_name = field_ident.to_string();
        let field_type = field.ty.clone();
        let attrs = PropAttrs::from_ast(&field.attrs)?;

        let prop = NamedProp {
            field_name,
            field_type,
            attrs,
        };

        // let tokens = prop_attrs_type_quote(prop.attrs.min_version);
        // let test = prop_attrs_type2!();
        // let result = validate_versions(
        //     prop.attrs.min_version,
        //     prop.attrs.max_version,
        //     Some(&prop.field_name),
        // );

        // if let Some(err) = result {
        //     Err(syn::Error::new(field.span(), err))
        // } else {
        //     Ok(prop)
        // }

        Ok(prop)
    }

    pub fn version_check_token_stream(
        &self,
        field_stream: TokenStream,
        trace: bool,
    ) -> TokenStream {

        // let field_stream = validate_versions_tokens(
        //     field_stream,
        //     &self.attrs.min_version,
        //     &self.attrs.max_version,
        //     Some(&self.field_name),
        // );
       
        let field_name = &self.field_name;

        if let Some(min_version) = &self.attrs.min_version {
            let min = prop_attrs_type_value(&min_version);

            if let Some(max_version) = &self.attrs.max_version {
                let max = prop_attrs_type_value(&max_version);
                let trace = if trace {
                    quote! {
                        else {
                            tracing::trace!("Field: <{}> is skipped because version: {} is outside min: {}, max: {}",stringify!(#field_name),version,#min,#max);
                        }
                    }
                } else {
                    quote! {}
                };
                quote! {
                    if (#min..=#max).contains(&version) {
                        #field_stream
                    }
                    #trace
                }
            } else {
                let trace = if trace {
                    quote! {
                        else {
                            tracing::trace!("Field: <{}> is skipped because version: {} is less than min: {}",stringify!(#field_name),version,#min);
                        }
                    }
                } else {
                    quote! {}
                };
                quote! {
                    if version >= #min {
                        #field_stream
                    }
                    #trace
                }
            }
        } else {
            quote! {}
        }
    }
}

pub fn prop_attrs_type_value(attrs_type: &PropAttrsType) -> TokenStream {
    match &attrs_type {
        PropAttrsType::LitStr(data) =>  parse_quote!(#data),
        PropAttrsType::LitFn(data) => parse_quote!(#data()),
        PropAttrsType::LitInt(data) => parse_quote!(#data),
        PropAttrsType::None => parse_quote!(-1),
    }
}
impl UnnamedProp {
    pub fn from_ast(field: &Field) -> syn::Result<Self> {
        let attrs = PropAttrs::from_ast(&field.attrs)?;
        let field_type = field.ty.clone();
        let prop = UnnamedProp { field_type, attrs };

        // let result = validate_versions(prop.attrs.min_version, prop.attrs.max_version, None);

        // if let Some(err) = result {
        //     Err(syn::Error::new(field.span(), err))
        // } else {
        //     Ok(prop)
        // }
        Ok(prop)
    }

    pub fn version_check_token_stream(
        &self,
        field_stream: TokenStream,
        trace: bool,
    ) -> TokenStream {
        // let field_stream = validate_versions_tokens(field_stream,
        //         &self.attrs.min_version,
        //         &self.attrs.max_version,
        //         None
        //     );
        if let Some(min_version) = &self.attrs.min_version {
            let min = prop_attrs_type_value(&min_version);
            if let Some(max_version) = &self.attrs.max_version {
                let max = prop_attrs_type_value(&max_version);
                let trace = if trace {
                    quote! {
                        else {
                            tracing::trace!("Field from tuple struct:is skipped because version: {} is outside min: {}, max: {}",version,#min,#max);
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    if (#min..=#max).contains(&version) {
                        #field_stream
                    }
                    #trace
                }
            } else {
                let trace = if trace {
                    quote! {
                        else {
                            tracing::trace!("Field from tuple struct: is skipped because version: {} is less than min: {}",version,#min);
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    if version >= #min {
                        #field_stream
                    }
                    #trace
                }
            }
        } else {
            quote! {}
        }
    }
}

// pub fn validate_versions_tokens(
//     token_stream: TokenStream,
//     min_prop: &PropAttrsType,
//     max_props: &Option<PropAttrsType>,
//     field: Option<&str>,
// ) -> TokenStream {
//     let min = prop_attrs_type_value(&min_prop);

//     let version_check_result = match max_props {
//         Some(max_values) => {
//             let max = prop_attrs_type_value(max_values);

//             dbg!(format!("min: {}, max: {}", &min, &max));
//             match field {
//                 Some(_) => quote! {
//                     if #min > #max {
//                         println!(format!("hereee min: {}, max: {}", #min, #max))
//                         //compile_error!("Field max_version is less than min version");
//                     }
//                 },
//                 None => quote! {
//                     if #min > #max {
//                         compile_error!("Max version is less than min version");
//                     }
//                 },
//             }
//         }
//         None => match field {
//             Some(_) => quote! {
//                 println!("test min_version{}, ", #min);
//                 // if #min < 0 {
//                 //     compile_error!("Field min_version must be positive.");
//                 // }
//             },
//             None => quote! {
//                 if #min < 0 {
//                     compile_error!("Min version must be positive.");
//                 }
//             },
//         },
//     };

    
//     quote! {
//         #version_check_result
//         #token_stream
//     }
// }
// pub fn validate_versions(min: i16, max: Option<i16>, field: Option<&str>) -> Option<String> {
//     match (max, field) {
//         // Print name in named fields
//         (Some(max), Some(field)) if min > max => Some(format!(
//             "On {field}, max version({max}) is less than min({min})."
//         )),
//         // No name to print in unnamed fields
//         (Some(max), None) if min > max => {
//             Some(format!("Max version({max}) is less than min({min})."))
//         }
//         (None, Some(field)) if min < 0 => {
//             Some(format!("On {field} min version({min}) must be positive."))
//         }
//         (None, None) if min < 0 => Some(format!("Min version({min}) must be positive.")),
//         _ => None,
//     }
// }
#[derive(Debug, Default, Clone)]
pub enum PropAttrsType {
    LitStr(Ident),
    LitFn(Ident),
    LitInt(i16),
    #[default]
    None,
}
#[derive(Debug, Default, Clone)]
pub(crate) struct PropAttrs {
    pub variant: bool,
    /// Will default to 0 if not specified.
    /// Note: `None` is encoded as "-1" so it's i16.
    pub min_version: Option<PropAttrsType>,
    /// Optional max version.
    /// The field won't be decoded from the buffer if it has a larger version than what is specified here.
    /// Note: `None` is encoded as "-1" so it's i16.
    pub max_version: Option<PropAttrsType>,
    /// Sets this value to the field when it isn't present in the buffer.
    /// Example: `#[fluvio(default = "-1")]`
    pub default_value: Option<String>,
    pub ignorable: Option<bool>,
}
impl PropAttrs {
    pub fn from_ast(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut prop_attrs = Self::default();

        parse_attributes!(attrs.iter(), "fluvio", meta,
            "min_version", prop_attrs.min_version => |expr: Option<syn::Expr>, attr_span, attr_name: &str| {
                let value = get_expr_value(&attr_name, &expr, attr_span)?;
                prop_attrs.min_version = Some(value);
                
                Ok(())
            }
            "max_version", prop_attrs.max_version => |expr: Option<syn::Expr>, attr_span, attr_name: &str| {
                let value = get_expr_value(&attr_name, &expr, attr_span)?;
                prop_attrs.max_version = Some(value);
                
                Ok(())
            }
            "default", prop_attrs.default_value => |expr: Option<syn::Expr>, attr_span, attr_name: &str| {
                let value = get_lit_str(&attr_name, &expr, attr_span)?;
                prop_attrs.default_value = Some(value.value());
                
                Ok(())
            }
            "ignorable", prop_attrs.ignorable => |_: Option<Expr>, _, _| {
                prop_attrs.ignorable = Some(true);
                
                Ok(())
            }
        );
        
        Ok(prop_attrs)
    }
}
