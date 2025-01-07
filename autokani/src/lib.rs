use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, token::Mut, AngleBracketedGenericArguments, FieldsNamed, FnArg, Ident,
    ImplItem, ImplItemMethod, Item, ItemFn, ItemImpl, ItemStruct, Pat, Path, Receiver, ReturnType,
    Type, TypeArray, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple,
};

const ARR_LIMIT: usize = 16;
trait ArbitraryInit {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream;
}

fn error_msg(msg: &str) -> proc_macro2::TokenStream {
    quote! {
        compile_error!(#msg);
    }
}
#[proc_macro_attribute]
/// Automatedly generate Kani one test harness for target function.
/// The harness name is `check_{function_name}`.
///
/// # Example
/// ```rust
/// use autokani::kani_test;
/// #[kani_test]
/// pub fn multi_param(a: i16, b: u8, c: f32, d: bool) {
///     let _ = a;
///     let x = b as f32 + c ;
///     if d {
///         let y = b + c as u8;
///     }
/// }
/// ```
/// The above code will generate a test harness for the `multi_param` function.
/// Run the harness with `cargo kani --harness check_multi_param`.
/// The harness could find the possible arithmetic overflow in the function.
pub fn kani_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Item);
    let func = match input {
        Item::Fn(func) => func,
        _ => {
            return error_msg("`kani_test` can only be used on functions.").into();
        }
    };

    let func_name = &func.sig.ident;
    let inputs = &func.sig.inputs;
    let harness_name = quote::format_ident!("check_{}", func_name);
    let mut harness_body = Vec::new();
    let mut call_args: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut call_path: Vec<proc_macro2::TokenStream> = Vec::new();

    for arg in inputs {
        match arg {
            FnArg::Receiver(receiver) => {
                let arg_placeholder = "self_receiver";
                let init_stmt = receiver.init_for_type(arg_placeholder, &receiver.mutability);
                harness_body.push(init_stmt);
                let receiver_ident = quote::format_ident!("{}", arg_placeholder);
                call_path.push(quote! { #receiver_ident });
            }
            FnArg::Typed(pat_type) => {
                let arg_name = match &*pat_type.pat {
                    syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                    _ => "arg".to_string(),
                };
                let arg_type = &pat_type.ty;
                let mutability = match &*pat_type.pat {
                    Pat::Ident(pat_ident) => pat_ident.mutability,
                    _ => None,
                };
                let init_stmt = arg_type.init_for_type(&arg_name, &mutability);
                harness_body.push(init_stmt);

                let arg_ident = quote::format_ident!("{}", arg_name);
                call_args.push(quote! { #arg_ident });
            }
        }
    }
    call_path.push(quote! { #func_name });

    let call_stmt = quote! {
        let _ = #(#call_path).*(#(#call_args),*);
    };
    harness_body.push(call_stmt);
    let harness_code = quote! {
        #[cfg(any(kani, feature = "debug_log"))]
        #[kani::proof]
        #[kani::unwind(64)]
        pub fn #harness_name() {
            #(#harness_body)*
        }
    };
    let output = quote! {
        #func

        #harness_code
    };
    output.into()
}

impl ArbitraryInit for Receiver {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        let arg_ident = quote::format_ident!("{}", arg_name);
        let init_stmt = quote! {
            let #mutability #arg_ident: Self = kani::any();
        };
        init_stmt
    }
}

impl ArbitraryInit for TypePath {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        // TODO: support Enum types
        let arg_ident = quote::format_ident!("{}", arg_name);
        if self.path.is_ident("u32") || self.path.is_ident("u64") || self.path.is_ident("usize") {
            quote! {
                let #mutability #arg_ident = kani::any();
                kani::assume(#arg_ident < 100000000);
            }
        } else if self.path.is_ident("i32") || self.path.is_ident("i64") {
            quote! {
                let #mutability #arg_ident = kani::any();
                kani::assume(#arg_ident < 100000000 && #arg_ident > -100000000);
            }
        } else if self.path.is_ident("String") || self.path.is_ident("str") {
            init_for_string(arg_name, mutability)
        } else if self.path.segments.last().is_some() {
            let final_seg = self.path.segments.last().unwrap();
            let inner_type = &final_seg.arguments;
            // TODO: support more types, e.g., HashMap, HashSet, etc.
            if final_seg.ident == "Vec" {
                let vec_type = inner_type.clone();
                match vec_type {
                    syn::PathArguments::AngleBracketed(args) => {
                        if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                            quote! {
                                let #mutability #arg_ident = kani::vec::any_vec::<#ty, #ARR_LIMIT>();
                            }
                        } else {
                            error_msg("Unsupported Vec Type")
                        }
                    }
                    _ => error_msg("Unsupported Vec Pattern"),
                }
            } else if final_seg.ident == "Option" {
                let option_type = inner_type.clone();
                match option_type {
                    syn::PathArguments::AngleBracketed(args) => {
                        if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                            let init_stmt = ty.init_for_type(arg_name, mutability);
                            quote! {
                                let #mutability #arg_ident = if kani::any::<bool>() {
                                    #init_stmt
                                    Some(#arg_ident)
                                } else {
                                    None
                                };
                            }
                        } else {
                            error_msg("Unsupported Option Type")
                        }
                    }
                    _ => error_msg("Unsupported Option Pattern"),
                }
            } else if final_seg.ident == "Result" {
                let result_type = inner_type.clone();
                match result_type {
                    syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments{args,..}) => {
                        let ok_init = match args.first() {
                            Some(syn::GenericArgument::Type(ty)) => ty.init_for_type(arg_name, mutability),
                            _ => error_msg("Unsupported Result Type"),
                        };
                        let err_init = match args.last() {
                            Some(syn::GenericArgument::Type(ty)) => ty.init_for_type(arg_name, mutability),
                            _ => error_msg("Unsupported Result Type"),
                        };
                        quote! {
                            let #mutability #arg_ident = if kani::any::<bool>() {
                                #ok_init
                                Ok(#arg_ident)
                            } else {
                                #err_init
                                Err(#arg_ident)
                            };
                        }
                    }
                    _ => error_msg("Unsupported Result Pattern"),
                }
            } else {
                // both typical types and user-defined structs are handled here
                let final_ident = &final_seg.ident;
                quote! {
                    let #mutability #arg_ident: #final_ident = kani::any();
                }
            }
        } else {
            error_msg("Failed to get the final segment of the path.")
        }
    }
}

fn init_for_string(arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
    const STRING_LIMIT: usize = 8;
    let arg_ident = quote::format_ident!("{}", arg_name);
    let arr_name = quote::format_ident!("{}_arr", arg_ident);
    quote! {
        let #arr_name = kani::any::<[char; #STRING_LIMIT]>();
        let #mutability #arg_ident = String::from_iter(#arr_name);
    }
}

impl ArbitraryInit for TypeArray {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        let arr_type = &self.elem;
        let arr_len = &self.len;
        let arg_ident = quote::format_ident!("{}", arg_name);
        quote! {
            let #mutability #arg_ident = kani::any::<[#arr_type; #arr_len]>();
        }
    }
}

impl ArbitraryInit for TypeSlice {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        let slice_type = &self.elem;
        let arg_ident = quote::format_ident!("{}", arg_name);
        quote! {
            let #mutability #arg_ident = kani::any::<[#slice_type; #ARR_LIMIT]>();
        }
    }
}

impl ArbitraryInit for TypeReference {
    fn init_for_type(&self, arg_name: &str, _mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        let obj_name = quote::format_ident!("{}_obj", arg_name);
        let arg_ident = quote::format_ident!("{}", arg_name);
        let mutability = self.mutability;
        let obj_init = self.elem.init_for_type(&obj_name.to_string(), &mutability);
        match self.elem.as_ref() {
            Type::Slice(_) => {
                let slice_method = match mutability {
                    Some(_) => "kani::slice::any_slice_of_array_mut",
                    None => "kani::slice::any_slice_of_array",
                };
                let slice_method = syn::parse_str::<Path>(slice_method).unwrap();
                quote! {
                    #obj_init
                    let #arg_ident = #slice_method(&#mutability #obj_name);
                }
            }
            _ => quote! {
                #obj_init
                let #arg_ident = &#mutability #obj_name;
            },
        }
    }
}

impl ArbitraryInit for TypePtr {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        let mutability = if mutability.is_some() {
            quote!(mut)
        } else {
            quote!()
        };
        let arg_ident = quote::format_ident!("{}", arg_name);
        quote! {
            let mut generator = kani::PointerGenerator::<{std::mem::size_of::<#self.elem>()}>::new();
            let #arg_ident: *#self.const_token #mutability #self.elem = generator.any_alloc_status().ptr;
        }
    }
}

impl ArbitraryInit for Type {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        match self {
            Type::Path(type_path) => type_path.init_for_type(arg_name, mutability),
            Type::Array(type_arr) => type_arr.init_for_type(arg_name, mutability),
            Type::Slice(type_slice) => type_slice.init_for_type(arg_name, mutability),
            Type::Tuple(type_tuple) => type_tuple.init_for_type(arg_name, mutability),
            Type::Reference(type_ref) => type_ref.init_for_type(arg_name, mutability),
            Type::Ptr(type_ptr) => type_ptr.init_for_type(arg_name, mutability),
            _ => error_msg("Unsupported argument type for `kani_test` macro."),
        }
    }
}

impl ArbitraryInit for TypeTuple {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        let tuple_elems = self.elems.iter().map(|elem| {
            let elem_name = quote::format_ident!("{}_elem", arg_name);
            let elem_init = elem.init_for_type(&elem_name.to_string(), mutability);
            quote! {
                {
                    #elem_init
                    #elem_name
                }
            }
        });
        let arg_ident = quote::format_ident!("{}", arg_name);
        quote! {
            let #mutability #arg_ident = (#(#tuple_elems),*);
        }
    }
}

#[proc_macro_attribute]
/// Impl `Arbitrary` for a struct by generating the `any` method based on the fields of the struct.
/// 
/// Since some common types (e.g., Vec) do not impl `Arbitrary`, it's inpractical to derive `Arbitrary`.
/// Instead, this macro generates the impl block for `Arbitrary`.
pub fn kani_arbitrary(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Item);
    let struct_def = match input {
        Item::Struct(ref struct_def) => struct_def,
        _ => {
            return error_msg("`kani_arbitrary` can only be used on structs.").into();
        }
    };
    let impl_stmt = impl_arbitrary_via_fields(&struct_def);
    let output = quote! {
        #struct_def

        #impl_stmt
    };
    output.into()
}

fn impl_arbitrary_via_fields(struct_def: &ItemStruct) -> proc_macro2::TokenStream {
    let mutability: Option<Mut> = None;
    let struct_name = &struct_def.ident;
    let init_stmt = match &struct_def.fields {
        syn::Fields::Named(fields) => {
            let mut fields_init: Vec<proc_macro2::TokenStream> = Vec::new();
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                let obj = field_type.init_for_type(&field_name.to_string(), &mutability);
                let field_init = quote! {
                    #field_name: {
                        #obj
                        #field_name
                    }
                };
                fields_init.push(field_init);
            }

            quote! {
                Self {
                    #(#fields_init),*
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            todo!("Fields::Unnamed");
        }
        syn::Fields::Unit => {
            todo!("Fields::Unit");
        }
    };
    quote! {
        #[cfg(any(kani, feature = "debug_log"))]
        impl kani::Arbitrary for #struct_name {
            fn any() -> Self {
                #init_stmt
            }
        }
    }
}

/// Extend the `Arbitrary` trait for target struct based on its constructor(e.g., `new` method).
/// Add this attribute to the impl block of the struct.
#[proc_macro_attribute]
pub fn extend_arbitrary(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Item);
    let impl_block = match input {
        Item::Impl(impl_block) => impl_block,
        _ => {
            return error_msg("`extend_arbitrary` can only be used on impl blocks.").into();
        }
    };
    let impl_stmt = impl_arbitrary_via_constructor(&impl_block);
    let output = quote! {
        #impl_block
        #impl_stmt
    };
    output.into()
}

fn find_constructor(impl_block: &ItemImpl, struct_name: &Ident) -> Option<ImplItemMethod> {
    for item in &impl_block.items {
        if let ImplItem::Method(method) = item {
            // if method.sig.ident == "new" {
            //     return method.clone();
            // }
            if let ReturnType::Type(_, return_type) = &method.sig.output {
                if let Type::Path(type_path) = &**return_type {
                    if type_path.path.is_ident(struct_name) || type_path.path.is_ident("Self") {
                        return Some(method.clone());
                    }
                }
            }
        }
    }
    None
}

fn impl_arbitrary_via_constructor(impl_block: &ItemImpl) -> Option<proc_macro2::TokenStream> {
    // This function should generate the `Arbitrary` impl block based on the `new` method.
    // You need to parse the `new` method and generate the corresponding `Arbitrary` impl block.
    let struct_name = match &*impl_block.self_ty {
        Type::Path(type_path) => type_path.path.get_ident()?,
        _ => {
            return error_msg("`extend_arbitrary` can only be used on impl blocks of structs.")
                .into();
        }
    };
    let constructor = find_constructor(impl_block, struct_name)?;
    let inputs = &constructor.sig.inputs;
    let func_name = &constructor.sig.ident;
    let mut init_code = Vec::new();
    let mut call_args: Vec<proc_macro2::TokenStream> = Vec::new();

    for arg in inputs {
        match arg {
            FnArg::Receiver(_) => unreachable!(),
            FnArg::Typed(pat_type) => {
                let arg_name = match &*pat_type.pat {
                    syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                    _ => "arg".to_string(),
                };
                let arg_type = &pat_type.ty;
                let mutability = match &*pat_type.pat {
                    Pat::Ident(pat_ident) => pat_ident.mutability,
                    _ => None,
                };
                let init_stmt = arg_type.init_for_type(&arg_name, &mutability);
                init_code.push(init_stmt);

                let arg_ident = quote::format_ident!("{}", arg_name);
                call_args.push(quote! { #arg_ident });
            }
        }
    }
    Some(quote! {
        #[cfg(any(kani, feature = "debug_log"))]
        impl kani::Arbitrary for #struct_name {
            fn any() -> Self {
                #(#init_code)*
                Self::#func_name(#(#call_args),*)
            }
        }
    })
}
