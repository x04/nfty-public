#[allow(dead_code)]
#[link(name="SecureEngineSDK64", kind="dylib")]
extern {
    #[link_name = "VMStart"]
    pub fn VM_START();

    #[link_name = "VMEnd"]
    pub fn VM_END();

    #[link_name = "StrEncryptStart"]
    pub fn STR_ENCRYPT_START();

    #[link_name = "StrEncryptEnd"]
    pub fn STR_ENCRYPT_END();

    #[link_name = "StrEncryptWStart"]
    pub fn STR_ENCRYPTW_START();

    #[link_name = "StrEncryptWEnd"]
    pub fn STR_ENCRYPTW_END();

    #[link_name = "UnprotectedStart"]
    pub fn UNPROTECTED_START();

    #[link_name = "UnprotectedEnd"]
    pub fn UNPROTECTED_END();

    #[link_name = "SECheckDebugger"]
    pub fn CHECK_DEBUGGER(var: &mut i32, val: i32);

    #[link_name = "SECheckProtection"]
    pub fn CHECK_PROTECTION(var: &mut i32, val: i32);

    #[link_name = "SECheckCodeIntegrity"]
    pub fn CHECK_CODE_INTEGRITY(var: &mut i32, val: i32);

    #[link_name = "SECheckRegistration"]
    pub fn CHECK_REGISTRATION(var: &mut i32, val: i32);

    #[link_name = "SECheckVirtualPC"]
    pub fn CHECK_VIRTUAL_PC(var: &mut i32, val: i32);
}



