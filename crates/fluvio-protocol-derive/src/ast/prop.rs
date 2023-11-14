use proc_macro::Span;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Expr, Field, LitInt, LitStr, Meta, Token, Type};

use crate::util::{get_expr_value, get_lit_int, get_lit_str};

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
        let min = prop_attrs_type_value(&self.attrs.min_version);
       
        let field_name = &self.field_name;

        if let Some(max_version_prop) = &self.attrs.max_version {
            let max = prop_attrs_type_value(&max_version_prop);
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
    }
}

pub fn prop_attrs_type_value(attrs_type: &PropAttrsType) -> TokenStream {
    match &attrs_type {
        PropAttrsType::LitStr(data) => {
            quote! {
                #data
            }
        }
        PropAttrsType::LitFn(data) => {
            quote! {
                #data()
            }
        }
        PropAttrsType::LitInt(data) => {
            quote! {
                #data
            }
        }
        PropAttrsType::None => quote! {
            1i16
        },
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
        let min = prop_attrs_type_value(&self.attrs.min_version);

        if let Some(max_version_prop) = &self.attrs.max_version {
            let max = prop_attrs_type_value(&max_version_prop);
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
    pub varint: bool,
    /// Will default to 0 if not specified.
    /// Note: `None` is encoded as "-1" so it's i16.
    pub min_version: PropAttrsType,
    /// Optional max version.
    /// The field won't be decoded from the buffer if it has a larger version than what is specified here.
    /// Note: `None` is encoded as "-1" so it's i16.
    pub max_version: Option<PropAttrsType>,
    /// Sets this value to the field when it isn't present in the buffer.
    /// Example: `#[fluvio(default = "-1")]`
    pub default_value: Option<String>,
}

impl PropAttrs {
    pub fn from_ast(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut prop_attrs = Self::default();

        // Find all supported field level attributes in one go.
        for attribute in attrs.iter() {
            if attribute.path().is_ident("varint") {
                prop_attrs.varint = true;
            } else if attribute.path().is_ident("fluvio") {
                if let Meta::List(list) = &attribute.meta {
                    if let Ok(list_args) =
                        list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    {
                        for args_meta in list_args.iter() {
                            if let Meta::NameValue(args_data) = args_meta {
                                let lit_expr = &args_data.value;

                                if let Some(args_name) = args_data.path.get_ident() {
                                    if args_name == "min_version" {
                                        // let value = get_lit_int("min_version", lit_expr)?;
                                        // prop_attrs.min_version = value.base10_parse::<i16>()?;
                                        let value = get_expr_value("min_version", lit_expr)?;
                                        prop_attrs.min_version = value;
                                    } else if args_name == "max_version" {
                                        // let value = get_lit_int("max_version", lit_expr)?;
                                        // prop_attrs.max_version = Some(value.base10_parse::<i16>()?);
                                        let value = get_expr_value("max_version", lit_expr)?;
                                        prop_attrs.max_version = Some(value);
                                    } else if args_name == "default" {
                                        let value = get_lit_str("default", lit_expr)?;
                                        prop_attrs.default_value = Some(value.value());
                                    } else {
                                        tracing::warn!(
                                            "#[fluvio({})] does nothing here.",
                                            args_name.to_token_stream().to_string(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(prop_attrs)
    }
}
