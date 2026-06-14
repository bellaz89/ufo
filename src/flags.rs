use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct PassFlags: u8 {
        const LINEAR = 0x1 << 0;
        const FIVED = 0x1 << 1;
        const EXACT = 0x1 << 2;
        const KICK = 0x1 << 3;
        const RADIATION = 0x1 << 4;
        const DOUBLE_PRECISION = 0x1 << 5;
        const ACHROMATIC = 0x1 << 6;
        const NO_APERTURE_CHECK = 0x1 << 7;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct TranLinearFlags: u8 {
        const VEC = 0x1 << 0;
        const MAT_XX = 0x1 << 1;
        const MAT_PXX = 0x1 << 2;
        const MAT_XPX = 0x1 << 3;
        const MAT_PXPX = 0x1 << 4;
        const NO_APERTURE_CHECK = 0x1 << 7;
    }
}
