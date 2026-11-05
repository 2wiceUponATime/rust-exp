use std::{fmt::Write, fs, io, path::PathBuf};

use convert_case::{Case, Casing};
use indoc::formatdoc;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, FnArg, ForeignItemFn, Ident, ImplItem, Item, LitStr, Meta, Pat, PatIdent,
    ReturnType, Type, TypeParen, TypePath, TypePtr,
};

#[macro_export]
macro_rules! debug {
    () => {
        println!("cargo::warning=");
    };
    ($($fmt:tt)*) => {
        for line in format!($($fmt)*).split('\n') {
            println!("cargo::warning={line}");
        }
    };
}

macro_rules! match_ident {
    ($path:expr, {
        $($($id:ident)|* => $value:expr),+ $(,)?
    }) => {
        {
            let path = $path;
            $(if $(path.is_ident(stringify!($id)))|* {
                $value;
            })*
        }
    };
}

macro_rules! idents_to_cpp {
    ($path:expr, $out:expr, { $($rust:ident => $cpp:tt),+ $(,)? }) => {
        match_ident!($path, {
            $($rust => {
                $out.push_str(idents_to_cpp!(@cpp $cpp));
                return Ok(());
            }),+
        });
    };
    (@cpp $cpp:literal) => {
        $cpp
    };
    (@cpp $cpp:ident) => {
        stringify!($cpp)
    };
}

pub fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|e| panic!("failed to read env var {key}: {e}"))
}

pub fn env_path(key: &str) -> PathBuf {
    PathBuf::from(env(key))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(io::Error),
    #[error("{0}")]
    Syn(syn::Error),
    #[error("unexpected item: {0:?}")]
    UnexpectedItem(Item),
    #[error("unexpected item: {:?}", .0.to_token_stream().to_string())]
    UnexpectedImplItem(ImplItem),
    #[error("unexpected type in impl: {}", .0.to_token_stream().to_string())]
    UnexpectedSelfTy(Type),
    #[error("found multiple #[cpp(...)] attributes")]
    MultipleCppAttrs,
    #[error("unable to convert to C++ type: {:?}", .0.to_token_stream().to_string())]
    CannotConvertToCppType(Type),
    #[error("string formatting error")]
    Fmt,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<syn::Error> for Error {
    fn from(err: syn::Error) -> Self {
        Self::Syn(err)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(_: std::fmt::Error) -> Self {
        Self::Fmt
    }
}

#[derive(Debug)]
struct CurrentImpl {
    struct_name: Ident,
    inner: TokenStream,
}

impl CurrentImpl {
    fn to_cpp(&self) -> String {
        let struct_name = self.struct_name.to_string();
        if struct_name == "Lcd" {
            return "brain::lcd".to_owned();
        }
        struct_name.to_case(Case::Camel)
    }
}

#[derive(Debug)]
enum FfiTy {
    Unit,
    F64,
    CStr,
    VelocityUnits,
    Passthrough(Type),
}

impl FfiTy {
    fn ty_to_cpp(ty: &Type, out: &mut String, current: &Option<CurrentImpl>) -> Result<(), Error> {
        match ty {
            Type::Paren(TypeParen { elem, .. }) => return Self::ty_to_cpp(elem, out, current),
            Type::Path(TypePath { path, .. }) => {
                if path.is_ident("Self")
                    && let Some(impl_) = current
                {
                    out.push_str(&impl_.to_cpp());
                    return Ok(());
                }
                idents_to_cpp!(path, out, {
                    u8 => uint8_t,
                    u16 => uint16_t,
                    u32 => uint32_t,
                    u64 => uint64_t,
                    usize => size_t,
                    i8 => int8_t,
                    i16 => int16_t,
                    i32 => int32_t,
                    i64 => int64_t,
                    f32 => float,
                    f64 => double,
                    bool => bool,
                    Direction => Direction,
                    TurnDirection => TurnDirection,
                    BrakeType => BrakeType,
                    VelocityUnits => VelocityUnits,
                    RawBrain => brain,
                    RawLcd => "brain::lcd",
                });
            }
            Type::Ptr(TypePtr { elem, .. }) => {
                Self::ty_to_cpp(elem, out, current)?;
                out.push('*');
                return Ok(());
            }
            _ => todo!(),
        };
        Err(Error::CannotConvertToCppType(ty.clone()))
    }

    pub fn as_cpp(&self, current: &Option<CurrentImpl>) -> Result<String, Error> {
        Ok(match self {
            Self::Unit => "void".to_owned(),
            Self::F64 => "double".to_owned(),
            Self::CStr => "const char*".to_owned(),
            Self::VelocityUnits => "VelocityUnits".to_owned(),
            Self::Passthrough(ty) => {
                let mut out = String::new();
                Self::ty_to_cpp(ty, &mut out, current)?;
                out
            }
        })
    }

    pub fn map_cpp_in(&self, arg: String) -> String {
        match self {
            Self::VelocityUnits => return format!("to_vex_enum({arg})"),
            Self::Passthrough(ty) if let Type::Path(TypePath { path, .. }) = ty => {
                match_ident!(path, {
                    VelocityUnits => return format!("to_vex_enum({arg})"),
                })
            }
            _ => {}
        }
        arg
    }

    pub fn map_cpp_out(&self, arg: String) -> String {
        match self {
            Self::VelocityUnits => return format!("return from_vex_enum({arg});"),
            Self::Passthrough(ty) if let Type::Path(TypePath { path, .. }) = ty => {
                match_ident!(path, {
                    VelocityUnits => return format!("return from_vex_enum({arg});"),
                })
            }
            _ => {}
        }
        format!("return {arg};")
    }
}

impl ToTokens for FfiTy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Unit => tokens.extend(quote!(())),
            Self::F64 => tokens.extend(quote!(f64)),
            Self::CStr => tokens.extend(quote!(*const core::ffi::c_char)),
            Self::VelocityUnits => tokens.extend(quote!(VelocityUnits)),
            Self::Passthrough(ty) => ty.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
struct Lowered {
    arg: TokenStream,
    name: Ident,
    ty: FfiTy,
}

#[derive(Debug)]
struct Attrs {
    func_override: Option<String>,
    body_override: Option<String>,
}

#[derive(Default)]
pub struct Builder {
    tokens: TokenStream,
    ffi_funcs: TokenStream,
    cpp: String,
    current_impl: Option<CurrentImpl>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cpp(&mut self, value: impl ToString) -> &mut Self {
        self.cpp = value.to_string();
        self
    }

    fn write_compile_commands(build: &cc::Build, file: &str) {
        let tool = build.get_compiler();

        let mut arguments = vec![tool.path().to_string_lossy().into_owned()];
        arguments.extend(
            tool.args()
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned()),
        );
        arguments.extend(["--target=arm-none-eabi", "-nostdlibinc", "-c", file].map(String::from));

        let db = serde_json::json!([{
            "directory": env("CARGO_MANIFEST_DIR"),
            "file": file,
            "arguments": arguments,
        }]);
        fs::write(
            "compile_commands.json",
            serde_json::to_string_pretty(&db).unwrap(),
        )
        .unwrap();
    }

    fn push(&mut self, tokens: TokenStream) {
        if let Some(CurrentImpl { inner: funcs, .. }) = &mut self.current_impl {
            funcs.extend(tokens);
        } else {
            self.tokens.extend(tokens);
        }
    }

    fn parse_attrs(attrs: &[Attribute]) -> Result<(Attrs, Vec<Attribute>), Error> {
        let mut meta_list = None;
        let mut rest = Vec::new();
        for attr in attrs {
            if let Meta::List(list) = &attr.meta
                && list.path.is_ident("cpp")
            {
                if meta_list.is_some() {
                    return Err(Error::MultipleCppAttrs);
                }
                meta_list = Some(list);
            } else {
                rest.push(attr.clone());
            }
        }
        let mut func_override = None;
        let mut body_override = None;
        if let Some(meta_list) = meta_list {
            meta_list.parse_nested_meta(|meta| {
                match_ident!(&meta.path, {
                    func => {
                        func_override = Some(meta.value()?.parse::<LitStr>()?.value());
                        return Ok(());
                    },
                    body => {
                        body_override = Some(meta.value()?.parse::<LitStr>()?.value());
                        return Ok(());
                    }
                });
                Err(meta.error("unsupported attribute"))
            })?;
        }
        let attrs = Attrs {
            func_override,
            body_override,
        };
        Ok((attrs, rest))
    }

    fn lower_input(ty: &Type, name: &Ident) -> Vec<Lowered> {
        let single = |arg: TokenStream, ty: FfiTy| {
            vec![Lowered {
                arg,
                name: name.clone(),
                ty,
            }]
        };
        if let Type::Path(TypePath { path, .. }) = ty {
            match_ident!(path, {
                CString => return single(quote!(#name.as_ptr()), FfiTy::CStr),
                Distance => return single(quote!(#name.mm()), FfiTy::F64),
                Angle => return single(quote!(#name.deg()), FfiTy::F64),
                Time => return single(quote!(#name.ms()), FfiTy::F64),
                Velocity => return vec![
                    Lowered {
                        arg: quote!(#name.value()),
                        name: format_ident!("{name}_value"),
                        ty: FfiTy::F64,
                    },
                    Lowered {
                        arg: quote!(#name.units()),
                        name: format_ident!("{name}_units"),
                        ty: FfiTy::VelocityUnits,
                    },
                ]
            });
        }
        single(name.to_token_stream(), FfiTy::Passthrough(ty.clone()))
    }

    fn build_verbatim(&mut self, tokens: TokenStream) -> Result<(), Error> {
        let decl: ForeignItemFn = syn::parse2(tokens)?;
        let vis = decl.vis;
        let (attrs, rest) = Self::parse_attrs(&decl.attrs)?;
        let prefix = match &self.current_impl {
            Some(CurrentImpl { struct_name, .. }) => {
                format!("{}_", struct_name.to_string().to_case(Case::Snake))
            }
            None => "".to_owned(),
        };
        let name = &decl.sig.ident;
        let ffi_name = format_ident!("shim_{prefix}{name}");
        let inputs = &decl.sig.inputs;
        let output = &decl.sig.output;
        let mut is_method = false;
        let lowered_inputs = decl
            .sig
            .inputs
            .iter()
            .map(|arg| match arg {
                FnArg::Receiver(_) => {
                    println!("method");
                    is_method = true;
                    Ok(vec![])
                }
                FnArg::Typed(pat_ty) => {
                    let Pat::Ident(PatIdent { ident, .. }) = pat_ty.pat.as_ref() else {
                        return Err(syn::Error::new_spanned(
                            pat_ty,
                            "destructuring parameters are not allowed",
                        ));
                    };
                    Ok(Self::lower_input(&pat_ty.ty, &ident))
                }
            })
            .try_fold(Vec::new(), |mut acc, item| {
                acc.extend(item?);
                Ok::<_, Error>(acc)
            })?;
        let (lowered_args, lowered_params, lowered_tys): (Vec<_>, Vec<_>, Vec<_>) =
            itertools::multiunzip(lowered_inputs.into_iter().map(|l| (l.arg, l.name, l.ty)));
        let (self_param, self_arg) = if is_method {
            let struct_name =
                format_ident!("Raw{}", self.current_impl.as_ref().unwrap().struct_name);
            (quote!(self_: *mut #struct_name,), quote!(self.0.as_ptr(),))
        } else {
            (TokenStream::new(), TokenStream::new())
        };
        let out = &decl.sig.output;
        self.push(quote! {
            #(#rest)*
            #vis fn #name(#inputs) #out {
                unsafe { #ffi_name(#self_arg #(#lowered_args),*) }
            }
        });
        self.ffi_funcs.extend(quote! {
            fn #ffi_name(#self_param #(#lowered_params: #lowered_tys),*) #output;
        });
        let output = match decl.sig.output {
            ReturnType::Default => FfiTy::Unit,
            ReturnType::Type(_, ty) => FfiTy::Passthrough(*ty.clone()),
        };
        let mut params = lowered_tys
            .iter()
            .zip(&lowered_params)
            .map(|(t, p)| Ok(format!("{} {p}", t.as_cpp(&self.current_impl)?)))
            .collect::<Result<Vec<_>, Error>>()?
            .join(", ");
        let mut func_prefix = "";
        if is_method && let Some(current) = &self.current_impl {
            params = format!(
                "{}* self{}{params}",
                current.to_cpp(),
                if lowered_params.len() > 0 { ", " } else { "" }
            );
            func_prefix = "self->";
        }
        let body = attrs.body_override.unwrap_or_else(|| {
            let func = format!(
                "{func_prefix}{}",
                attrs.func_override.unwrap_or_else(|| name.to_string()),
            );
            let args = lowered_tys
                .iter()
                .zip(&lowered_params)
                .map(|(t, p)| t.map_cpp_in(p.to_string()))
                .collect::<Vec<_>>()
                .join(", ");
            output.map_cpp_out(format!("{func}({args})"))
        });
        write!(
            self.cpp,
            "{}",
            formatdoc! {"
                {ret} {ffi_name}({params}) {{
                    {body}
                }}\n",
                ret = output.as_cpp(&self.current_impl)?,
            }
        )?;
        Ok(())
    }

    fn build_impl_item(&mut self, item: ImplItem) -> Result<(), Error> {
        match item {
            ImplItem::Fn(_) => {
                self.push(item.into_token_stream());
                Ok(())
            }
            ImplItem::Verbatim(tokens) => self.build_verbatim(tokens),
            _ => Err(Error::UnexpectedImplItem(item)),
        }
    }

    fn build_item(&mut self, item: Item) -> Result<(), Error> {
        match item {
            Item::Verbatim(tokens) => return self.build_verbatim(tokens),
            Item::Impl(impl_) => {
                let self_ty = impl_.self_ty.as_ref();
                let segments = match self_ty {
                    Type::Path(path) => &path.path.segments,
                    ty => return Err(Error::UnexpectedSelfTy(ty.clone())),
                };
                if impl_.trait_.is_some() || segments.len() != 1 {
                    self.push(impl_.into_token_stream());
                    return Ok(());
                }
                let name = &segments[0].ident;
                self.current_impl = Some(CurrentImpl {
                    struct_name: name.clone(),
                    inner: TokenStream::new(),
                });
                for item in impl_.items {
                    self.build_impl_item(item)?;
                }
                let CurrentImpl { inner, .. } = self.current_impl.take().unwrap();
                let generics = impl_.generics;
                self.tokens.extend(quote! {
                    impl #generics #self_ty {
                        #inner
                    }
                });
            }
            _ => self.push(item.into_token_stream()),
        }
        Ok(())
    }

    pub fn build(&mut self) -> Result<(), Error> {
        println!("cargo::rerun-if-changed=bindings.rs");
        let out_dir = env_path("OUT_DIR");
        let content = std::fs::read_to_string("bindings.rs")?;
        let file = syn::parse_file(&content)?;
        self.cpp.push_str("extern \"C\"{\n");
        for item in file.items {
            self.build_item(item)?;
        }
        self.cpp.push_str("}\n");
        let Self {
            tokens: funcs,
            ffi_funcs,
            ..
        } = self;
        let file = quote! {
            unsafe extern "C" {
                #ffi_funcs
            }

            #funcs
        };
        let shim_file = out_dir.join("shim.cpp");
        fs::write(out_dir.join("shim.cpp"), &self.cpp)?;
        fs::write(
            out_dir.join("bindings.rs"),
            prettyplease::unparse(&syn::parse2(file)?),
        )?;

        let sdk = env("VEX_SDK_PATH");

        let mut build = cc::Build::new();
        build
            .cpp(true)
            .cpp_link_stdlib(None)
            .std("gnu++17")
            .flags([
                "-mcpu=cortex-m7",
                "-fshort-enums",
                "-fno-rtti",
                "-fno-exceptions",
                "-fno-threadsafe-statics",
                "-Werror=return-type",
                "-U__INT32_TYPE__",
                "-U__UINT32_TYPE__",
            ])
            .flags(
                [
                    "include",
                    "gcc/include",
                    "gcc/include/c++/7.3.1",
                    "gcc/include/c++/7.3.1/arm-none-eabi",
                ]
                .map(|dir| format!("-isystem{sdk}/{dir}")),
            )
            .define("__INT32_TYPE__", "long")
            .define("__UINT32_TYPE__", "unsigned long");

        Self::write_compile_commands(&build, &out_dir.join("shim/prelude.cpp").to_string_lossy());

        build.file(shim_file).compile("rust_exp_shim");

        Ok(())
    }
}
