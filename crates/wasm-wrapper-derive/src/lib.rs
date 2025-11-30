//! Procedural macro for generating WASM wrappers around core types.
//!
//! # Usage
//!
//! ```rust,ignore
//! use wasm_wrapper_derive::wasm_wrapper;
//!
//! // Basic wrapper - generates struct, Deref, From, Serialize, Deserialize
//! wasm_wrapper! {
//!     pub struct StaticNode wraps alizarin_core::StaticNode;
//! }
//!
//! // With getters and setters
//! wasm_wrapper! {
//!     pub struct StaticNode wraps alizarin_core::StaticNode {
//!         get nodeid,
//!         get name,
//!         get set alias,
//!     }
//! }
//! ```
//!
//! Or use the attribute macro:
//!
//! ```rust,ignore
//! #[wasm_wrap(alizarin_core::StaticNode)]
//! pub struct StaticNode {
//!     #[get]
//!     nodeid: (),
//!     #[get] #[set]
//!     name: (),
//! }
//! ```

use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Path, ItemStruct, Attribute, parse::Parse, parse::ParseStream, Token, punctuated::Punctuated, Ident, Visibility};

/// Configuration for a single field getter/setter
#[derive(Debug, Clone, FromMeta)]
struct FieldConfig {
    /// Field name
    name: syn::Ident,
    /// Optional JS name override
    #[darling(default)]
    js_name: Option<String>,
    /// Field type (for complex conversions)
    #[darling(default)]
    field_type: Option<String>,
    /// Whether to wrap in Option
    #[darling(default)]
    optional: bool,
    /// Custom wrapper type for nested WASM types
    #[darling(default)]
    wrapper_type: Option<String>,
}

/// Main derive macro attributes
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wasm_wrapper), supports(struct_unit))]
struct WasmWrapperArgs {
    ident: syn::Ident,

    /// The core type to wrap (e.g., "alizarin_core::StaticNode")
    core_type: String,

    /// Optional JS class name override
    #[darling(default)]
    js_name: Option<String>,

    /// Fields to generate getters for
    #[darling(default, multiple)]
    getter: Vec<String>,

    /// Fields to generate setters for
    #[darling(default, multiple)]
    setter: Vec<String>,

    /// Whether to implement Clone
    #[darling(default = "default_true")]
    derive_clone: bool,

    /// Whether to implement Debug
    #[darling(default = "default_true")]
    derive_debug: bool,

    /// Whether to generate a constructor from JsValue
    #[darling(default = "default_true")]
    constructor: bool,

    /// Whether to generate a toJSON method
    #[darling(default = "default_true")]
    to_json: bool,

    /// Whether to generate copy() method
    #[darling(default = "default_true")]
    copy_method: bool,
}

fn default_true() -> bool {
    true
}

/// Parse a type path string into a syn::Path
fn parse_type_path(s: &str) -> syn::Path {
    syn::parse_str(s).expect("Invalid type path")
}

/// Generate the wrapper struct definition
fn generate_struct(args: &WasmWrapperArgs, core_path: &Path) -> TokenStream2 {
    let name = &args.ident;

    let derive_clone = if args.derive_clone {
        quote! { Clone, }
    } else {
        quote! {}
    };

    let derive_debug = if args.derive_debug {
        quote! { Debug, }
    } else {
        quote! {}
    };

    quote! {
        #[wasm_bindgen]
        #[repr(transparent)]
        #[derive(#derive_clone #derive_debug)]
        pub struct #name(#[wasm_bindgen(skip)] pub #core_path);
    }
}

/// Generate Deref and DerefMut implementations
fn generate_deref_impls(args: &WasmWrapperArgs, core_path: &Path) -> TokenStream2 {
    let name = &args.ident;

    quote! {
        impl ::std::ops::Deref for #name {
            type Target = #core_path;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::std::ops::DerefMut for #name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    }
}

/// Generate From implementation
fn generate_from_impl(args: &WasmWrapperArgs, core_path: &Path) -> TokenStream2 {
    let name = &args.ident;

    quote! {
        impl From<#core_path> for #name {
            fn from(core: #core_path) -> Self {
                #name(core)
            }
        }

        impl From<#name> for #core_path {
            fn from(wrapper: #name) -> Self {
                wrapper.0
            }
        }
    }
}

/// Generate Serialize and Deserialize implementations
fn generate_serde_impls(args: &WasmWrapperArgs, core_path: &Path) -> TokenStream2 {
    let name = &args.ident;

    quote! {
        impl ::serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #core_path::deserialize(deserializer).map(#name)
            }
        }
    }
}

/// Generate wasm_bindgen constructor and methods
fn generate_wasm_methods(args: &WasmWrapperArgs, core_path: &Path) -> TokenStream2 {
    let name = &args.ident;

    let constructor = if args.constructor {
        quote! {
            #[wasm_bindgen(constructor)]
            pub fn new(json_data: ::wasm_bindgen::JsValue) -> Result<#name, ::wasm_bindgen::JsValue> {
                let data: #core_path = ::serde_wasm_bindgen::from_value(json_data)?;
                Ok(#name(data))
            }
        }
    } else {
        quote! {}
    };

    let copy_method = if args.copy_method {
        quote! {
            #[wasm_bindgen]
            pub fn copy(&self) -> #name {
                self.clone()
            }
        }
    } else {
        quote! {}
    };

    let to_json = if args.to_json {
        quote! {
            #[wasm_bindgen(js_name = toJSON)]
            pub fn to_json(&self) -> ::wasm_bindgen::JsValue {
                ::serde_wasm_bindgen::to_value(&self.0).unwrap_or(::wasm_bindgen::JsValue::NULL)
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #[wasm_bindgen]
        impl #name {
            #constructor
            #copy_method
            #to_json
        }
    }
}

/// Generate getter for a field
fn generate_getter(name: &syn::Ident, field_name: &str) -> TokenStream2 {
    let field_ident = format_ident!("{}", field_name);
    let getter_name = format_ident!("get_{}", field_name);
    let js_name = field_name;

    // For now, generate a simple clone-based getter
    // More sophisticated handling would require knowing the field type
    quote! {
        #[wasm_bindgen(getter = #js_name)]
        pub fn #getter_name(&self) -> ::wasm_bindgen::JsValue {
            ::serde_wasm_bindgen::to_value(&self.0.#field_ident).unwrap_or(::wasm_bindgen::JsValue::NULL)
        }
    }
}

/// Generate setter for a field
fn generate_setter(name: &syn::Ident, field_name: &str) -> TokenStream2 {
    let field_ident = format_ident!("{}", field_name);
    let setter_name = format_ident!("set_{}", field_name);
    let js_name = field_name;

    quote! {
        #[wasm_bindgen(setter = #js_name)]
        pub fn #setter_name(&mut self, value: ::wasm_bindgen::JsValue) {
            if let Ok(v) = ::serde_wasm_bindgen::from_value(value) {
                self.0.#field_ident = v;
            }
        }
    }
}

/// Generate getters and setters
fn generate_accessors(args: &WasmWrapperArgs) -> TokenStream2 {
    let name = &args.ident;

    let getters: Vec<_> = args.getter.iter()
        .map(|field| generate_getter(name, field))
        .collect();

    let setters: Vec<_> = args.setter.iter()
        .map(|field| generate_setter(name, field))
        .collect();

    if getters.is_empty() && setters.is_empty() {
        return quote! {};
    }

    quote! {
        #[wasm_bindgen]
        impl #name {
            #(#getters)*
            #(#setters)*
        }
    }
}

#[proc_macro_derive(WasmWrapper, attributes(wasm_wrapper))]
pub fn wasm_wrapper_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let args = match WasmWrapperArgs::from_derive_input(&input) {
        Ok(args) => args,
        Err(e) => return e.write_errors().into(),
    };

    let core_path = parse_type_path(&args.core_type);

    let struct_def = generate_struct(&args, &core_path);
    let deref_impls = generate_deref_impls(&args, &core_path);
    let from_impl = generate_from_impl(&args, &core_path);
    let serde_impls = generate_serde_impls(&args, &core_path);
    let wasm_methods = generate_wasm_methods(&args, &core_path);
    let accessors = generate_accessors(&args);

    let expanded = quote! {
        #struct_def
        #deref_impls
        #from_impl
        #serde_impls
        #wasm_methods
        #accessors
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Attribute macro version - replaces the struct entirely
// ============================================================================

/// Input for the wasm_wrap attribute macro
struct WasmWrapInput {
    vis: Visibility,
    name: Ident,
    core_type: Path,
    getters: Vec<Ident>,
    setters: Vec<Ident>,
    // Options
    no_constructor: bool,
    no_to_json: bool,
    no_copy: bool,
    impl_default: bool,
}

impl Parse for WasmWrapInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse: pub struct Name wraps core::Type { get field1, get set field2, ... }
        let vis: Visibility = input.parse()?;
        let _: Token![struct] = input.parse()?;
        let name: Ident = input.parse()?;

        // Parse "wraps"
        let wraps: Ident = input.parse()?;
        if wraps != "wraps" {
            return Err(syn::Error::new(wraps.span(), "expected 'wraps'"));
        }

        let core_type: Path = input.parse()?;

        let mut getters = Vec::new();
        let mut setters = Vec::new();
        let mut no_constructor = false;
        let mut no_to_json = false;
        let mut no_copy = false;
        let mut impl_default = false;

        // Optional block with accessors and options
        if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);

            while !content.is_empty() {
                let mut has_get = false;
                let mut has_set = false;

                // Parse keywords
                while content.peek(Ident) {
                    let kw: Ident = content.fork().parse()?;
                    if kw == "get" {
                        has_get = true;
                        let _: Ident = content.parse()?;
                    } else if kw == "set" {
                        has_set = true;
                        let _: Ident = content.parse()?;
                    } else if kw == "no_constructor" {
                        no_constructor = true;
                        let _: Ident = content.parse()?;
                        let _ = content.parse::<Token![,]>();
                        continue;
                    } else if kw == "no_to_json" {
                        no_to_json = true;
                        let _: Ident = content.parse()?;
                        let _ = content.parse::<Token![,]>();
                        continue;
                    } else if kw == "no_copy" {
                        no_copy = true;
                        let _: Ident = content.parse()?;
                        let _ = content.parse::<Token![,]>();
                        continue;
                    } else if kw == "impl_default" {
                        impl_default = true;
                        let _: Ident = content.parse()?;
                        let _ = content.parse::<Token![,]>();
                        continue;
                    } else {
                        break;
                    }
                }

                // If we have get/set, parse field name
                if has_get || has_set {
                    let field: Ident = content.parse()?;

                    if has_get {
                        getters.push(field.clone());
                    }
                    if has_set {
                        setters.push(field);
                    }

                    // Optional comma
                    let _ = content.parse::<Token![,]>();
                }
            }
        } else {
            // Just semicolon
            let _: Token![;] = input.parse()?;
        }

        Ok(WasmWrapInput {
            vis,
            name,
            core_type,
            getters,
            setters,
            no_constructor,
            no_to_json,
            no_copy,
            impl_default,
        })
    }
}

/// Declarative-style macro for creating WASM wrappers.
///
/// # Usage
///
/// ```rust,ignore
/// wasm_wrapper! {
///     pub struct StaticNode wraps alizarin_core::StaticNode {
///         get nodeid,
///         get name,
///         get set alias,
///     }
/// }
///
/// // With options to disable auto-generated methods
/// wasm_wrapper! {
///     pub struct StaticTranslatableString wraps alizarin_core::StaticTranslatableString {
///         no_constructor,
///         no_to_json,
///         impl_default,
///         get lang,
///     }
/// }
/// ```
#[proc_macro]
pub fn wasm_wrapper(input: TokenStream) -> TokenStream {
    let WasmWrapInput {
        vis, name, core_type, getters, setters,
        no_constructor, no_to_json, no_copy, impl_default
    } = parse_macro_input!(input as WasmWrapInput);

    // Generate getters
    let getter_impls: Vec<_> = getters.iter().map(|field| {
        let getter_name = format_ident!("get_{}", field);
        let js_name = field.to_string();
        quote! {
            #[wasm_bindgen(getter = #js_name)]
            pub fn #getter_name(&self) -> ::wasm_bindgen::JsValue {
                ::serde_wasm_bindgen::to_value(&self.0.#field).unwrap_or(::wasm_bindgen::JsValue::NULL)
            }
        }
    }).collect();

    // Generate setters
    let setter_impls: Vec<_> = setters.iter().map(|field| {
        let setter_name = format_ident!("set_{}", field);
        let js_name = field.to_string();
        quote! {
            #[wasm_bindgen(setter = #js_name)]
            pub fn #setter_name(&mut self, value: ::wasm_bindgen::JsValue) {
                if let Ok(v) = ::serde_wasm_bindgen::from_value(value) {
                    self.0.#field = v;
                }
            }
        }
    }).collect();

    let accessors_impl = if getter_impls.is_empty() && setter_impls.is_empty() {
        quote! {}
    } else {
        quote! {
            #[wasm_bindgen]
            impl #name {
                #(#getter_impls)*
                #(#setter_impls)*
            }
        }
    };

    // Conditionally generate constructor
    let constructor_impl = if no_constructor {
        quote! {}
    } else {
        quote! {
            #[wasm_bindgen(constructor)]
            pub fn new(json_data: ::wasm_bindgen::JsValue) -> Result<#name, ::wasm_bindgen::JsValue> {
                let data: #core_type = ::serde_wasm_bindgen::from_value(json_data)?;
                Ok(#name(data))
            }
        }
    };

    // Conditionally generate copy
    let copy_impl = if no_copy {
        quote! {}
    } else {
        quote! {
            #[wasm_bindgen]
            pub fn copy(&self) -> #name {
                self.clone()
            }
        }
    };

    // Conditionally generate toJSON
    let to_json_impl = if no_to_json {
        quote! {}
    } else {
        quote! {
            #[wasm_bindgen(js_name = toJSON)]
            pub fn to_json(&self) -> ::wasm_bindgen::JsValue {
                ::serde_wasm_bindgen::to_value(&self.0).unwrap_or(::wasm_bindgen::JsValue::NULL)
            }
        }
    };

    // Conditionally generate Default impl
    let default_impl = if impl_default {
        quote! {
            impl Default for #name {
                fn default() -> Self {
                    #name(<#core_type as Default>::default())
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #[wasm_bindgen]
        #[repr(transparent)]
        #[derive(Clone, Debug)]
        #vis struct #name(#[wasm_bindgen(skip)] pub #core_type);

        impl ::std::ops::Deref for #name {
            type Target = #core_type;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::std::ops::DerefMut for #name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<#core_type> for #name {
            fn from(core: #core_type) -> Self {
                #name(core)
            }
        }

        impl From<#name> for #core_type {
            fn from(wrapper: #name) -> Self {
                wrapper.0
            }
        }

        #default_impl

        impl ::serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #core_type::deserialize(deserializer).map(#name)
            }
        }

        #[wasm_bindgen]
        impl #name {
            #constructor_impl
            #copy_impl
            #to_json_impl
        }

        #accessors_impl
    };

    TokenStream::from(expanded)
}
