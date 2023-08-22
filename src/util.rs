#[macro_export]
/// generates read_type functions.
macro_rules! forward_function {
    ($name:ident) => {
        paste! {
            pub static mut [<p $name>]: *const c_void = 0 as *const c_void;

            #[naked]
            #[no_mangle]
            pub extern "system" fn $name() {
                unsafe {
                    asm!(
                    "jmpq  *{}(%rip)",
                    sym [<p $name>],
                    options(noreturn, att_syntax),
                    );
                }
            }
        }
    };
}
