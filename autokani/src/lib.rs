use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, token::Mut, FieldsNamed, FnArg, Ident, Item, ItemFn, ItemStruct, Pat, Path,
    Receiver, Type, TypeArray, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple,
};

const ARR_LIMIT: usize = 16;
trait ArbitraryInit {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream;
}
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
        #[cfg(kani)]
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
    fn init_for_type(
        &self,
        arg_name: &str,
        mutability: &Option<Mut>,
    ) -> proc_macro2::TokenStream {
        let arg_ident = quote::format_ident!("{}", arg_name);
        let init_stmt = quote! {
            let #mutability #arg_ident: Self = kani::any();
        };
        // match self.reference {
        //     Some(_) => quote! {
        //         #init_stmt
        //         let #arg_ident = &#mutability #arg_ident;
        //     },
        //     None => init_stmt,
        // }
        init_stmt
    }
}

impl ArbitraryInit for TypePath {
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        // TODO: support Enum types
        let arg_ident = quote::format_ident!("{}", arg_name);
        if self.path.is_ident("u32") || self.path.is_ident("u64") {
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
                            quote! {
                                compile_error!("Unsupported Vec Type");
                            }
                        }
                    }
                    _ => quote! {
                        compile_error!("Unsupported Vec Pattern");
                    },
                }
            } else {
                // both typical types and user-defined structs are handled here
                let final_ident = &final_seg.ident;
                quote! {
                    let #mutability #arg_ident: #final_ident = kani::any();
                }
            }
        } else {
            quote! {
                compile_error!("Failed to get the final segment of the path.");
            }
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
    fn init_for_type(&self, arg_name: &str, mutability: &Option<Mut>) -> proc_macro2::TokenStream {
        let obj_name = quote::format_ident!("{}_obj", arg_name);
        let arg_ident = quote::format_ident!("{}", arg_name);
        let is_mut = matches!(mutability, Some(_));
        let obj_init = self.elem.init_for_type(&obj_name.to_string(), &mutability);
        match self.elem.as_ref() {
            Type::Slice(_) => {
                let slice_method = if is_mut {
                    "kani::slice::any_slice_of_array_mut"
                } else {
                    "kani::slice::any_slice_of_array"
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
            _ => quote! {
                compile_error!("Unsupported argument type for `kani_test` macro.");
            },
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
// Since some common structs do not impl `Arbitrary`, we do not derive `Arbitrary` for them.
// Instead, we provide a macro to generate the impl block for `Arbitrary`.
pub fn kani_arbitrary(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Item);
    let struct_def = match input {
        Item::Struct(ref struct_def) => struct_def,
        _ => {
            return quote! {
                compile_error!("`kani_arbitrary` can only be used on structs.");
            }
            .into();
        }
    };
    let impl_stmt = impl_arbitrary_for_struct(&struct_def);
    let output = quote! {
        #struct_def

        #impl_stmt
    };
    output.into()
}

fn impl_arbitrary_for_struct(struct_def: &ItemStruct) -> proc_macro2::TokenStream {
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
        #[cfg(kani)]
        impl kani::Arbitrary for #struct_name {
            fn any() -> Self {
                #init_stmt
            }
        }
    }
}
