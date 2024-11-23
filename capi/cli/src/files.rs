pub const FILES: Files = Files::new();

macro_rules! files {
    ($($field:ident, $name:expr;)*) => {
        #[derive(Debug)]
        pub struct Files {
            $(
                $field: &'static [u8],
            )*
        }

        impl Files {
            pub const fn new() -> Self {
                Self {
                    $(
                        $field: include_bytes!(concat!(
                            env!("FILES"),
                            "/",
                            $name,
                        )),
                    )*
                }
            }

            pub fn list_invalid(&self) -> Vec<&'static str> {
                let mut invalid_files = Vec::new();

                let files = [
                    $(
                        (self.$field, $name),
                    )*
                ];

                for (file, name) in files {
                    if file.is_empty() {
                        invalid_files.push(name);
                    }
                }

                invalid_files
            }
        }
    };
}

files!(
    capi_debugger_bg_wasm, "capi-debugger_bg.wasm";
    capi_debugger_js, "capi-debugger.js";
    capi_host_wasm, "capi_host.wasm";
    index_html, "index.html";
    tailwind_js, "tailwind.js";
);
