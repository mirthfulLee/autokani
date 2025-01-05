use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, Ident, Item, ItemFn, Pat, Path, Type, TypeReference, TypePtr};

#[proc_macro_attribute]
pub fn kani_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Item);
    let func = match input {
        Item::Fn(func) => func,
        _ => {
            return quote! {
                compile_error!("`kani_test` can only be used on functions.");
            }
            .into();
        }
    };

    let func_name = &func.sig.ident;
    let inputs = &func.sig.inputs;
    // let output = &func.sig.output;

    let harness_name = quote::format_ident!("check_{}", func_name);

    let mut harness_body = Vec::new();
    let mut call_args: Vec<proc_macro2::TokenStream> = Vec::new();

    for arg in inputs {
        if let FnArg::Typed(pat_type) = arg {
            let arg_name = match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                _ => "arg".to_string(),
            };
            let arg_type = &pat_type.ty;
            let is_mut = matches!(&*pat_type.pat, syn::Pat::Ident(pat_ident) if pat_ident.mutability.is_some());
            let init_stmt = init_for_type(&arg_name, &arg_type, is_mut);
            harness_body.push(init_stmt);

            let arg_ident = quote::format_ident!("{}", arg_name);
            call_args.push(quote! { #arg_ident });
        }
    }

    let call_stmt = quote! {
        let _ = #func_name(#(#call_args),*);
    };
    harness_body.push(call_stmt);

    let output = quote! {
        #func

        #[cfg(kani)]
        #[kani::proof]
        #[kani::unwind(64)]
        pub fn #harness_name() {
            #(#harness_body)*
        }
    };

    output.into()
}

fn init_for_type(arg_name: &str, arg_type: &Type, is_mut: bool) -> proc_macro2::TokenStream {
    const ARR_LIMIT: usize = 16;
    let mutability = if is_mut { quote!(mut) } else { quote!() };
    let arg_ident = quote::format_ident!("{}", arg_name);
    let init_stmt = match arg_type {
        // Limit the range of u32 and u64 to avoid overflow errors
        Type::Path(ref type_path)
            if type_path.path.is_ident("u32") || type_path.path.is_ident("u64") =>
        {
            quote! {
                let #mutability #arg_ident = kani::any();
                kani::assume(#arg_ident < 100000000);
            }
        }
        Type::Path(ref type_path)
            if type_path.path.is_ident("i32") || type_path.path.is_ident("i64") =>
        {
            quote! {
                let #mutability #arg_ident = kani::any();
                kani::assume(#arg_ident < 100000000 && #arg_ident > -100000000);
            }
        }
        Type::Path(ref type_path)
            if type_path.path.is_ident("String") || type_path.path.is_ident("str") =>
        {
            init_for_string(arg_name, is_mut)
        }
        Type::Path(ref type_path) if matches!(type_path.path.segments.last(), Some(_)) => {
            let final_seg = type_path.path.segments.last().unwrap();
            let inner_type = &final_seg.arguments;
            // TODO: support more types, e.g., HashMap, HashSet, etc.
            if final_seg.ident == "Vec" {
                let vec_type = inner_type.clone();
                if let syn::PathArguments::AngleBracketed(args) = vec_type {
                    if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                        quote! {
                            let #mutability #arg_ident = kani::vec::any_vec::<#ty, #ARR_LIMIT>();
                        }
                    } else {
                        quote! {
                            compile_error!("Unsupported Vec Type");
                        }
                    }
                } else {
                    quote! {
                        compile_error!("Unsupported Vec Pattern");
                    }
                }
            } else {
                // FIXME: is this necessary?
                let final_ident = &final_seg.ident;
                quote! {
                    let #mutability #arg_ident: #final_ident = kani::any();
                }
            }
        }
        Type::Array(type_arr) => {
            let arr_type = &type_arr.elem;
            let arr_len = &type_arr.len;
            quote! {
                let #mutability #arg_ident = kani::any::<[#arr_type; #arr_len]>();
            }
        }
        Type::Slice(type_slice) => {
            let slice_type = &type_slice.elem;
            quote! {
                let #mutability #arg_ident = kani::any::<[#slice_type; #ARR_LIMIT]>();
            }
        }
        Type::Tuple(tuple) => {
            let tuple_elems = tuple.elems.iter().map(|elem| {
                let elem_name = quote::format_ident!("{}_elem", arg_ident);
                let elem_init = init_for_type(&elem_name.to_string(), elem, is_mut);
                quote! {
                    {
                        #elem_init
                        #elem_name
                    }
                }
            });
            quote! {
                let #mutability #arg_ident = (#(#tuple_elems),*);
            }
        }
        Type::Reference(TypeReference {
            elem, mutability, ..
        }) => {
            let obj_name = quote::format_ident!("{}_obj", arg_ident);
            let is_mut = matches!(mutability, Some(_));
            let obj_init =
                init_for_type(&obj_name.to_string(), elem, is_mut);
            match elem.as_ref() {
                Type::Slice(_) => {
                    let slice_method = if is_mut { "kani::slice::any_slice_of_array_mut" } else { "kani::slice::any_slice_of_array" };
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
        Type::Ptr(TypePtr {
            elem,const_token, mutability, ..
        }) => {
            quote! {
                let mut generator = kani::PointerGenerator::<{std::mem::size_of::<#elem>()}>::new();
                let #arg_ident: *#const_token #mutability #elem = generator.any_alloc_status().ptr;
            }
        }
        _ => quote! {
            compile_error!("Unsupported argument type for `kani_test` macro.");
        },
    };
    init_stmt
}

fn init_for_string(arg_name: &str, is_mut: bool) -> proc_macro2::TokenStream {
    const STRING_LIMIT: usize = 8;
    let mutability = if is_mut { quote!(mut) } else { quote!() };
    let arg_ident = quote::format_ident!("{}", arg_name);
    let arr_name = quote::format_ident!("{}_arr", arg_ident);
    quote! {
        let #arr_name = kani::any::<[char; #STRING_LIMIT]>();
        let #mutability #arg_ident = String::from_iter(#arr_name);
    }
}
