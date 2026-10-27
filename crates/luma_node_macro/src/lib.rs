use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, DeriveInput, FnArg, Item, ItemFn, ItemImpl, ItemStruct, Lit, Pat,
    ReturnType, Token, Type,
};

/// Attributes for `#[luma_node(...)]` and `#[luma_body_node(...)]`
#[derive(Default)]
struct LumaNodeAttrArgs {
    category: Option<String>,
    tag: Option<String>,
    title: Option<String>,
    kind: Option<String>,
    header_rgb: Option<syn::Expr>,
    system: Option<syn::Path>,
}

impl Parse for LumaNodeAttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = LumaNodeAttrArgs::default();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match ident.to_string().as_str() {
                "category" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.category = Some(s.value());
                    }
                }
                "tag" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.tag = Some(s.value());
                    }
                }
                "title" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.title = Some(s.value());
                    }
                }
                "kind" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.kind = Some(s.value());
                    }
                }
                "header_rgb" => {
                    let expr: syn::Expr = input.parse()?;
                    args.header_rgb = Some(expr);
                }
                "system" => {
                    let path: syn::Path = input.parse()?;
                    args.system = Some(path);
                }
                _ => {
                    let _expr: syn::Expr = input.parse()?;
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(args)
    }
}

/// Unified attribute macro converting a function or struct into an auto-registered Luma node.
///
/// # Examples
///
/// 1. **Pure Function Node**:
/// ```ignore
/// #[luma_node(category = "Math", title = "Multiply")]
/// pub fn multiply(a: f32, #[default = 2.0] b: f32) -> f32 {
///     a * b
/// }
/// ```
///
/// 2. **Custom / UI Node Struct**:
/// ```ignore
/// #[luma_node]
/// pub struct ImageViewerNode;
///
/// impl LumaNode for ImageViewerNode {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn luma_node(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as LumaNodeAttrArgs);
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Fn(func) => expand_fn_node(args, func),
        Item::Struct(st) => expand_struct_node(args, st),
        Item::Impl(item_impl) => expand_impl_node(args, item_impl),
        _ => syn::Error::new_spanned(
            item,
            "#[luma_node] can only be applied to functions, structs, or impl blocks",
        )
        .to_compile_error()
        .into(),
    }
}

fn expand_struct_node(args: LumaNodeAttrArgs, st: ItemStruct) -> TokenStream {
    let st_ident = &st.ident;
    let construct_expr = match &st.fields {
        syn::Fields::Unit => quote! { #st_ident },
        syn::Fields::Named(named) if named.named.is_empty() => quote! { #st_ident {} },
        _ => quote! { <#st_ident as ::std::default::Default>::default() },
    };

    let system_submission = if let Some(sys_path) = args.system {
        quote! {
            ::luma_ui::inventory::submit! {
                ::luma_ui::RegisteredLumaSystem {
                    register: |app| {
                        app.add_systems(::bevy::app::Update, #sys_path);
                    },
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #st

        ::luma_ui::inventory::submit! {
            ::luma_ui::RegisteredLumaNode {
                constructor: || Box::new(#construct_expr),
            }
        }

        #system_submission
    };

    expanded.into()
}

/// Attribute macro for UI body nodes.
///
/// Can be placed on a struct implementing `LumaNode`.
/// Optionally binds a Bevy update system via `#[luma_body_node(system = my_sync_fn)]`.
#[proc_macro_attribute]
pub fn luma_body_node(args: TokenStream, input: TokenStream) -> TokenStream {
    luma_node(args, input)
}

/// Attribute macro auto-registering any Bevy system into the UI update loop.
///
/// # Example
/// ```ignore
/// #[luma_system]
/// fn sync_my_widget(...) {
///     // runs in Bevy Update schedule automatically
/// }
/// ```
#[proc_macro_attribute]
pub fn luma_system(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    let fn_ident = &func.sig.ident;
    let expanded = quote! {
        #func

        ::luma_ui::inventory::submit! {
            ::luma_ui::RegisteredLumaSystem {
                register: |app| {
                    app.add_systems(::bevy::app::Update, #fn_ident);
                },
            }
        }
    };
    expanded.into()
}

fn expand_impl_node(_args: LumaNodeAttrArgs, item_impl: ItemImpl) -> TokenStream {
    let self_ty = &item_impl.self_ty;
    let expanded = quote! {
        #item_impl

        ::luma_ui::inventory::submit! {
            ::luma_ui::RegisteredLumaNode {
                constructor: || Box::new(<#self_ty as ::std::default::Default>::default()),
            }
        }
    };

    expanded.into()
}

fn expand_fn_node(args: LumaNodeAttrArgs, func: ItemFn) -> TokenStream {
    let fn_vis = &func.vis;
    let fn_ident = &func.sig.ident;
    let fn_name_str = fn_ident.to_string();

    let node_kind = args.kind.unwrap_or_else(|| fn_name_str.clone());
    let node_title = args.title.unwrap_or_else(|| {
        fn_name_str
            .split('_')
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    });
    let category = args.category.unwrap_or_else(|| "Custom".into());
    let tag = args.tag.unwrap_or_else(|| "NODE".into());

    let companion_struct_ident = format_ident!("__LumaNode_{}", fn_ident);

    // Clean parameter attributes (e.g. `#[default = ...]`) so emitted function is valid Rust
    let mut clean_func = func.clone();
    let mut input_names = Vec::new();
    let mut input_types = Vec::new();
    let mut input_defaults = Vec::new();

    for input_arg in &mut clean_func.sig.inputs {
        if let FnArg::Typed(pat_type) = input_arg {
            let mut default_expr = None;
            pat_type.attrs.retain(|attr| {
                if attr.path().is_ident("default") {
                    if let Ok(syn::MetaNameValue { value, .. }) = attr.meta.require_name_value() {
                        default_expr = Some(value.clone());
                    } else if let Ok(syn::MetaList { tokens, .. }) = attr.meta.require_list() {
                        if let Ok(expr) = syn::parse2::<syn::Expr>(tokens.clone()) {
                            default_expr = Some(expr);
                        }
                    }
                    false
                } else {
                    true
                }
            });

            let name_str = match &*pat_type.pat {
                Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                _ => "arg".to_string(),
            };
            input_names.push(name_str);
            input_types.push((*pat_type.ty).clone());
            input_defaults.push(default_expr);
        }
    }

    // Extract return type (output)
    let has_return = match &func.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(t) if t.elems.is_empty() => false,
            _ => true,
        },
    };

    let return_ty = match &func.sig.output {
        ReturnType::Type(_, ty) => (*ty).clone(),
        _ => Box::new(syn::parse_quote!(())),
    };

    let pin_inputs_tokens = input_names
        .iter()
        .zip(&input_types)
        .zip(&input_defaults)
        .map(|((name, ty), def_opt)| {
            match def_opt {
                Some(expr) => quote! {
                    {
                        let mut pin = ::luma_ui::PinInfo::new(
                            #name,
                            <#ty as ::luma_dataflow::FromNodeValue>::node_data_type(),
                        );
                        let val = ::luma_dataflow::IntoNodeValue::into_node_value(#expr);
                        if let ::luma_dataflow::Value::Primitive(prim) = val {
                            pin.default_value = prim;
                        }
                        pin
                    }
                },
                None => quote! {
                    ::luma_ui::PinInfo::new(
                        #name,
                        <#ty as ::luma_dataflow::FromNodeValue>::node_data_type(),
                    )
                },
            }
        });

    let pin_outputs_tokens = if has_return {
        quote! {
            vec![
                ::luma_ui::PinInfo::new(
                    "result",
                    <#return_ty as ::luma_dataflow::IntoNodeValue>::node_data_type(),
                )
            ]
        }
    } else {
        quote! { Vec::new() }
    };

    let eval_args_pass = (0..input_types.len()).map(|idx| {
        let arg_ident = format_ident!("arg_{}", idx);
        quote! { #arg_ident }
    });

    let eval_inputs_let = input_types
        .iter()
        .zip(&input_defaults)
        .enumerate()
        .map(|(idx, (ty, def_opt))| {
            let arg_ident = format_ident!("arg_{}", idx);
            let fallback = match def_opt {
                Some(expr) => quote! { #expr },
                None => quote! { Default::default() },
            };
            quote! {
                let #arg_ident: #ty = inputs
                    .get(#idx)
                    .and_then(::luma_dataflow::FromNodeValue::from_node_value)
                    .unwrap_or_else(|| #fallback);
            }
        });

    let eval_body = if has_return {
        quote! {
            #(#eval_inputs_let)*
            let res = #fn_ident(#(#eval_args_pass),*);
            if let Some(out_slot) = outputs.get_mut(0) {
                *out_slot = ::luma_dataflow::IntoNodeValue::into_node_value(res);
            }
        }
    } else {
        quote! {
            #(#eval_inputs_let)*
            #fn_ident(#(#eval_args_pass),*);
        }
    };

    let style_fn = if let Some(rgb_expr) = &args.header_rgb {
        quote! {
            fn style(&self) -> ::luma_ui::NodeStyle {
                let rgb: [f32; 3] = #rgb_expr;
                ::luma_ui::NodeStyle::header_rgb(rgb[0], rgb[1], rgb[2])
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #clean_func

        #[allow(non_camel_case_types)]
        #fn_vis struct #companion_struct_ident;

        impl ::luma_ui::LumaNode for #companion_struct_ident {
            fn kind(&self) -> &'static str {
                #node_kind
            }

            fn title(&self) -> &str {
                #node_title
            }

            fn category(&self) -> (&'static str, &'static str) {
                (#category, #tag)
            }

            #style_fn

            fn inputs(&self) -> Vec<::luma_ui::PinInfo> {
                vec![#(#pin_inputs_tokens),*]
            }

            fn outputs(&self) -> Vec<::luma_ui::PinInfo> {
                #pin_outputs_tokens
            }

            fn evaluate(&self, inputs: &[::luma_dataflow::Value], outputs: &mut [::luma_dataflow::Value]) {
                #eval_body
            }
        }

        ::luma_ui::inventory::submit! {
            ::luma_ui::RegisteredLumaNode {
                constructor: || Box::new(#companion_struct_ident),
            }
        }
    };

    expanded.into()
}

/// Derive macro allowing any user struct to automatically become an engine resource.
///
/// # Example
/// ```ignore
/// #[derive(Clone, Debug, LumaResource)]
/// #[resource("CustomMesh")]
/// pub struct CustomMesh {
///     pub name: String,
///     pub color: LinearColor,
/// }
/// ```
#[proc_macro_derive(LumaResource, attributes(resource))]
pub fn derive_luma_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input.ident;
    let mut custom_name = struct_ident.to_string();

    for attr in &input.attrs {
        if attr.path().is_ident("resource") {
            if let Ok(lit) = attr.parse_args::<Lit>() {
                if let Lit::Str(s) = lit {
                    custom_name = s.value();
                }
            }
        }
    }

    let expanded = quote! {
        impl ::luma_dataflow::CustomNodeResource for #struct_ident {
            fn type_name() -> &'static str {
                #custom_name
            }
        }
    };

    expanded.into()
}
