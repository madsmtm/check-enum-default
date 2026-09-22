# Check C `enum` default integer type

A small script to check the default value of enums in C.

See Clang's logic in:
https://github.com/llvm/llvm-project/blob/llvmorg-23.1.0/clang/lib/Sema/SemaDecl.cpp#L18121-L18163

The result of this experiment is roughly:

```rust
cfg_select! {
    any(all(target_os = "windows", target_env = "msvc"), target_os = "uefi") => {
        type UnsignedEnum = core::ffi::c_int;
    }
    target_arch = "hexagon" => {
        type UnsignedEnum = core::ffi::c_uchar;
    }
    _ => {
        type UnsignedEnum = core::ffi::c_uint;
    }
}

cfg_select! {
    target_arch = "hexagon" => {
        type SignedEnum = core::ffi::c_schar;
    }
    _ => {
        type SignedEnum = core::ffi::c_int;
    }
}
```

(This can also be emulated with `#[repr(C)]` enums, though those aren't ABI stable).
