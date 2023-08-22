use crate::dl_string::AllocatedDLWString;

#[repr(C)]
pub struct HashableString {
    pub string: *const u16,
    pub hash: u32,
    pub needs_hashing: bool,
    pub unk0xd: [u8;0x3],
}
#[repr(C)]
pub struct FD4ResCap {
    pub vtable: usize,
    pub resource_string: FD4BasicHashString,
}

#[repr(C)]
pub struct FD4BasicHashString {
    pub vtable: usize,
    pub allocated_string: AllocatedDLWString,
    pub hashable_string: HashableString,
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use crate::HashableString::{FD4BasicHashString, FD4ResCap, HashableString};

    #[test]
    pub fn test() {
        assert_eq!(size_of::<HashableString>(), 0x10);
        assert_eq!(size_of::<FD4BasicHashString>(), 0x40);
        assert_eq!(size_of::<FD4ResCap>(), 0x48);
    }
}