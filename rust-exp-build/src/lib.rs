use std::{
    fmt::Write,
    fs, io, mem,
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use indoc::formatdoc;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, FnArg, ForeignItemFn, GenericArgument, Ident, ImplItem, Item, ItemImpl, ItemMod,
    LitBool, LitStr, Meta, Pat, PatIdent, PathArguments, ReturnType, Type, TypeParen, TypePath,
    TypePtr, punctuated::Punctuated,
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
    ($path:expr, $out:expr, { $($($rust:ident)|* => $cpp:tt),+ $(,)? }) => {
        match_ident!($path, {
            $($($rust)|* => {
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

fn single<T, P>(p: &Punctuated<T, P>) -> Option<&T> {
    let mut it = p.iter();
    let first = it.next()?;
    it.next().is_none().then_some(first)
}

fn single_type_generic<'a>(path: &'a syn::Path, outer: &str) -> Option<&'a Type> {
    if let Some(seg) = single(&path.segments)
        && seg.ident == outer
        && let PathArguments::AngleBracketed(ab) = &seg.arguments
        && let Some(GenericArgument::Type(inner)) = single(&ab.args)
    {
        Some(inner)
    } else {
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(io::Error),
    #[error("{0}")]
    Syn(syn::Error),
    #[error("found multiple #[cpp(...)] attributes")]
    MultipleCppAttrs,
    #[error("unable to convert to C++ type: {:?}", .0.to_token_stream().to_string())]
    CannotConvertToCppType(Box<Type>),
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
struct CurrentImpl(Ident);

impl CurrentImpl {
    fn to_cpp(&self) -> String {
        let struct_name = self.0.to_string();
        return match struct_name.as_str() {
            "BrainBattery" => "brain::battery",
            "BrainButton" => "brain::button",
            "BrainLcd" | "BrainLcdGuard" => "brain::lcd",
            "MotorGroup" => "motor_group",
            "File" => "FIL",
            _ => return struct_name.to_case(Case::Camel),
        }
        .to_owned();
    }
}

#[derive(Debug)]
enum FfiTy {
    Unit,
    U8,
    U32,
    USize,
    F64,
    CStr,
    VelocityUnits,
    Passthrough(Box<Type>),
}

impl FfiTy {
    fn ty_to_cpp(ty: &Type, out: &mut String, current: &Option<CurrentImpl>) -> Result<(), Error> {
        match ty {
            Type::Paren(TypeParen { elem, .. }) => return Self::ty_to_cpp(elem, out, current),
            Type::Tuple(t) if t.elems.is_empty() => {
                out.push_str("void");
                return Ok(());
            }
            Type::Path(TypePath { path, .. }) => {
                if path.is_ident("Self")
                    && let Some(impl_) = current
                {
                    out.push_str(&impl_.to_cpp());
                    return Ok(());
                }
                if let Some(pointee) = single_type_generic(&path, "NonNull") {
                    Self::ty_to_cpp(pointee, out, current)?;
                    out.push('*');
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
                    c_char => char,
                    bool => bool,
                    Color => Color,
                    BrakeType => BrakeType,
                    Direction => Direction,
                    FontType => FontType,
                    FResult => FResult,
                    GearSetting => GearSetting,
                    TurnDirection => TurnDirection,
                    VelocityUnits => VelocityUnits,
                    Whence => Whence,
                    Brain => brain,
                    BrainBattery => "brain::battery",
                    BrainButton => "brain::button",
                    BrainLcd => "brain::lcd",
                    Motor => motor,
                    MotorGroup | RawMotorGroup => motor_group,
                    File => "FIL",
                });
            }
            Type::FnPtr(f) => {
                out.push_str("fn_ptr<");
                match &f.output {
                    ReturnType::Type(_, ty) => Self::ty_to_cpp(ty, out, current)?,
                    ReturnType::Default => {
                        out.push_str("void");
                    }
                }
                out.push('(');
                if f.inputs.is_empty() {
                    out.push_str("void");
                }
                for (i, arg) in f.inputs.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    Self::ty_to_cpp(&arg.ty, out, current)?;
                }
                out.push_str(")>");
                return Ok(());
            }
            Type::Ptr(TypePtr { elem, .. }) => {
                Self::ty_to_cpp(elem, out, current)?;
                out.push('*');
                return Ok(());
            }
            _ => {}
        };
        Err(Error::CannotConvertToCppType(Box::new(ty.clone())))
    }

    pub fn as_cpp(&self, current: &Option<CurrentImpl>) -> Result<String, Error> {
        Ok(match self {
            Self::Unit => "void".to_owned(),
            Self::U8 => "uint8_t".to_owned(),
            Self::U32 => "uint32_t".to_owned(),
            Self::USize => "size_t".to_owned(),
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
    fn map_cpp(&self, arg: String, dir: &str) -> String {
        match self {
            Self::VelocityUnits => return format!("{dir}_vex({arg})"),
            Self::Passthrough(ty) if let Type::Path(TypePath { path, .. }) = ty.as_ref() => {
                match_ident!(path, {
                    Color | BrakeType | Direction | FontType | FResult | GearSetting | VelocityUnits | Whence
                        => return format!("{dir}_vex({arg})"),
                })
            }
            _ => {}
        }
        arg
    }

    pub fn map_cpp_in(&self, arg: String) -> String {
        self.map_cpp(arg, "to")
    }

    pub fn map_cpp_out(&self, arg: String) -> String {
        self.map_cpp(arg, "from")
    }
}

impl ToTokens for FfiTy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Unit => tokens.extend(quote!(())),
            Self::U8 => tokens.extend(quote!(u8)),
            Self::U32 => tokens.extend(quote!(u32)),
            Self::USize => tokens.extend(quote!(usize)),
            Self::F64 => tokens.extend(quote!(f64)),
            Self::CStr => tokens.extend(quote!(*const core::ffi::c_char)),
            Self::VelocityUnits => tokens.extend(quote!(crate::units::VelocityUnits)),
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
    allow_method: bool,
}

#[derive(Default)]
pub struct Builder {
    tokens: TokenStream,
    ffi_funcs: TokenStream,
    mod_path: Vec<Ident>,
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
        self.tokens.extend(tokens);
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
        let mut allow_method = true;
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
                    },
                    method => {
                        let value = meta.value()?.parse::<LitBool>()?.value();
                        if value {
                            return Err(meta.error("only `method = false` is allowed"))
                        }
                        allow_method = false;
                        return Ok(())
                    },
                });
                Err(meta.error("unsupported attribute"))
            })?;
        }
        let attrs = Attrs {
            func_override,
            body_override,
            allow_method,
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
        match ty {
            Type::Path(TypePath { path, .. }) => {
                match_ident!(path, {
                    Port => return single(quote!(#name.index()), FfiTy::U8),
                    Angle => return single(quote!(#name.deg()), FfiTy::F64),
                    Current => return single(quote!(#name.amps()), FfiTy::F64),
                    Distance => return single(quote!(#name.mm()), FfiTy::F64),
                    Duration => return single(
                        quote!(#name.as_millis().try_into().expect("Duration in milliseconds exceeds u32::MAX")),
                        FfiTy::U32,
                    ),
                    Torque => return single(quote!(#name.newton_meters()), FfiTy::F64),
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
                    ],
                });
            }
            Type::Reference(type_ref)
                if let Type::Path(TypePath { path, .. }) = type_ref.elem.as_ref()
                    && path.is_ident("str") =>
            {
                return vec![
                    Lowered {
                        arg: quote!(#name.as_ptr()),
                        name: format_ident!("{name}_chars"),
                        ty: FfiTy::CStr,
                    },
                    Lowered {
                        arg: quote!(#name.len()),
                        name: format_ident!("{name}_len"),
                        ty: FfiTy::USize,
                    },
                ];
            }
            _ => {}
        }
        single(
            name.to_token_stream(),
            FfiTy::Passthrough(Box::new(ty.clone())),
        )
    }

    fn build_verbatim(&mut self, tokens: TokenStream) -> Result<(), Error> {
        let decl: ForeignItemFn = syn::parse2(tokens)?;
        let vis = decl.vis;
        let (attrs, rest) = Self::parse_attrs(&decl.attrs)?;
        let mut prefix: String = self.mod_path.iter().map(|m| format!("{m}_")).collect();
        if let Some(CurrentImpl(struct_name)) = &self.current_impl {
            prefix += &format!("{}_", struct_name.to_string().to_case(Case::Snake))
        }
        let name = &decl.sig.ident;
        let ffi_name = format_ident!("shim_{prefix}{name}");
        let output = &decl.sig.output;
        let mut has_self = false;
        let lowered_inputs = decl
            .sig
            .inputs
            .iter()
            .map(|arg| match arg {
                FnArg::Receiver(_) => {
                    has_self = true;
                    Ok(vec![])
                }
                FnArg::Typed(pat_ty) => {
                    let Pat::Ident(PatIdent { ident, .. }) = pat_ty.pat.as_ref() else {
                        return Err(syn::Error::new_spanned(
                            pat_ty,
                            "destructuring parameters are not allowed",
                        ));
                    };
                    Ok(Self::lower_input(&pat_ty.ty, ident))
                }
            })
            .try_fold(Vec::new(), |mut acc, item| {
                acc.extend(item?);
                Ok::<_, Error>(acc)
            })?;
        let is_method = has_self && attrs.allow_method;
        let (lowered_args, lowered_params, lowered_tys): (Vec<_>, Vec<_>, Vec<_>) =
            itertools::multiunzip(lowered_inputs.into_iter().map(|l| (l.arg, l.name, l.ty)));
        let (self_param, self_arg) = if has_self {
            let ident = &self.current_impl.as_ref().unwrap().0;
            let struct_name = match ident.to_string().as_str() {
                "BrainLcdGuard" => &format_ident!("BrainLcd"),
                _ => ident,
            };
            (quote!(self_: *mut #struct_name,), quote!(self.as_ptr(),))
        } else {
            (TokenStream::new(), TokenStream::new())
        };
        let sig = &decl.sig;
        self.push(quote! {
            #(#rest)*
            #vis #sig {
                unsafe { #ffi_name(#self_arg #(#lowered_args),*) }
            }
        });
        self.ffi_funcs.extend(quote! {
            fn #ffi_name(#self_param #(#lowered_params: #lowered_tys),*) #output;
        });
        let output = match decl.sig.output {
            ReturnType::Default => FfiTy::Unit,
            ReturnType::Type(_, ty) => FfiTy::Passthrough(ty.clone()),
        };
        let ret = output.as_cpp(&self.current_impl)?;
        let mut params = lowered_tys
            .iter()
            .zip(&lowered_params)
            .map(|(t, p)| Ok(format!("{} {p}", t.as_cpp(&self.current_impl)?)))
            .collect::<Result<Vec<_>, Error>>()?
            .join(", ");
        let mut args = lowered_tys
            .iter()
            .zip(&lowered_params)
            .map(|(t, p)| t.map_cpp_in(p.to_string()))
            .collect::<Vec<_>>()
            .join(", ");
        let mut func_prefix = "";
        if has_self && let Some(current) = &self.current_impl {
            params = format!(
                "{}* self{}{params}",
                current.to_cpp(),
                if !lowered_params.is_empty() { ", " } else { "" }
            );
            if is_method {
                func_prefix = "self->";
            } else {
                args = format!("self{}{args}", if !args.is_empty() { ", " } else { "" })
            }
        }
        let body = attrs.body_override.unwrap_or_else(|| {
            let func = format!(
                "{func_prefix}{}",
                attrs
                    .func_override
                    .unwrap_or_else(|| name.to_string().to_case(Case::Camel)),
            );
            format!(
                "{}{};",
                if ret == "void" { "" } else { "return " },
                output.map_cpp_out(format!("{func}({args})"))
            )
        });
        write!(
            self.cpp,
            "{}",
            formatdoc! {"
                {ret} {ffi_name}({params}) {{
                    {body}
                }}
            "}
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
            _ => {
                self.push(item.into_token_stream());
                Ok(())
            }
        }
    }

    fn build_item(&mut self, item: Item, dir: &Path) -> Result<(), Error> {
        match item {
            Item::Verbatim(tokens) => return self.build_verbatim(tokens),
            Item::Impl(impl_) => {
                let self_ty = impl_.self_ty.as_ref();
                let segment = match self_ty {
                    Type::Path(path) if let Some(segment) = single(&path.path.segments) => segment,
                    _ => {
                        self.push(impl_.to_token_stream());
                        return Ok(());
                    }
                };

                let name = &segment.ident;
                self.current_impl = Some(CurrentImpl(name.clone()));

                let outer = mem::take(&mut self.tokens);
                for item in &impl_.items {
                    self.build_impl_item(item.clone())?;
                }
                self.current_impl = None;
                let inner = mem::replace(&mut self.tokens, outer);

                let ItemImpl {
                    generics,
                    trait_,
                    unsafety,
                    ..
                } = &impl_;
                let trait_ = trait_
                    .as_ref()
                    .map(|(path, for_token)| quote!(#path #for_token));
                self.tokens.extend(quote! {
                    #unsafety impl #generics #trait_ #self_ty {
                        #inner
                    }
                });
            }
            Item::Mod(m) => self.build_mod(m, dir)?,
            _ => self.push(item.into_token_stream()),
        }
        Ok(())
    }

    fn build_mod(&mut self, m: ItemMod, dir: &Path) -> Result<(), Error> {
        let ItemMod {
            vis,
            ident,
            content,
            attrs,
            ..
        } = m;
        let items = match content {
            Some((_, items)) => items,
            None => {
                let file = dir.join(format!("{}.rs", ident));
                println!("cargo::rerun-if-changed={}", file.display());
                syn::parse_file(&fs::read_to_string(file)?)?.items
            }
        };
        let outer_tokens = mem::take(&mut self.tokens);
        let outer_ffi = mem::take(&mut self.ffi_funcs);

        self.mod_path.push(ident.clone());
        let child_dir = dir.join(ident.to_string());
        for item in items {
            self.build_item(item, &child_dir)?;
        }
        self.mod_path.pop();

        let inner_tokens = mem::replace(&mut self.tokens, outer_tokens);
        let inner_ffi = mem::replace(&mut self.ffi_funcs, outer_ffi);
        self.tokens.extend(quote! {
            #(#attrs)*
            #vis mod #ident {
                unsafe extern "C" { #inner_ffi }
                #inner_tokens
            }
        });
        Ok(())
    }

    pub fn build(&mut self) -> Result<(), Error> {
        println!("cargo::rerun-if-changed=bindings/mod.rs");
        let out_dir = env_path("OUT_DIR");
        let content = std::fs::read_to_string("bindings/mod.rs")?;
        let file = syn::parse_file(&content)?;
        self.cpp.push_str("extern \"C\"{\n");
        for item in file.items {
            self.build_item(item, Path::new("bindings"))?;
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

        let sdk_script = format!("{sdk}/lscript.ld");
        println!("cargo::rerun-if-changed={sdk_script}");
        let script = fs::read_to_string(&sdk_script)?
            .replace("\r\n", "\n")
            .replace(" . = 0x20;", " . = ADDR(.text) + 0x20;")
            .replace(
                "__ARM.attributes_end = .;\n} > SRAM_USER",
                "__ARM.attributes_end = .;\n}",
            );
        let script_path = out_dir.join("lscript.ld");
        fs::write(&script_path, script)?;
        println!("cargo::rustc-link-arg=-T{}", script_path.display());
        println!("cargo::rustc-link-search=native={sdk}");
        println!("cargo::rustc-link-search=native={sdk}/gcc/libs");
        for lib in ["exprt", "c_nano", "stdc++_nano", "m", "gcc"] {
            println!("cargo::rustc-link-lib=static={lib}");
        }

        let mut build = cc::Build::new();
        build
            .compiler("clang")
            .archiver("llvm-ar")
            .cpp(true)
            .cpp_link_stdlib(None)
            .std("gnu++17")
            .flags([
                "--target=thumbv7em-none-eabihf",
                "-nostdlibinc",
                "-mcpu=cortex-m7",
                "-mfloat-abi=hard",
                "-flto=thin",
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
