
#![allow(dead_code)]

macro_rules! fourcc_code {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        (($a as u32) | (($b as u32) << 8) |
	 (($c as u32) << 16) | (($d as u32) << 24))
    }
}

fn from_u32(u: u32) -> (char, char, char, char) {
    (((u >>  0) as u8) as char,
     ((u >>  8) as u8) as char,
     ((u >> 16) as u8) as char,
     ((u >> 24) as u8) as char)
}

macro_rules! decl_fourcc_list {
    ( $( $name:ident ($a:expr, $b:expr, $c:expr, $d:expr)),* ) => {
        #[repr(u32)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        pub enum FourCC {
            Unknown = 0,
            $(
                $name = fourcc_code!($a, $b, $c, $d),
            )*
        }

        impl From<u32> for FourCC {
            fn from(n: u32) -> FourCC {
                match from_u32(n) {
                    $(
                        ($a, $b, $c, $d) => FourCC::$name,
                    )*
                    _ => FourCC::Unknown
                }
            }
        }
    }
}

decl_fourcc_list! {
    /* color index */
    C8 ('C', '8', ' ', ' '), /* [7:0] C */

    /* 8 bpp RGB */
    RGB332 ('R', 'G', 'B', '8'), /* [7:0] R:G:B 3:3:2 */
    BGR233 ('B', 'G', 'R', '8'), /* [7:0] B:G:R 2:3:3 */

    /* 16 bpp RGB */
    XRGB4444 ('X', 'R', '1', '2'), /* [15:0] x:R:G:B 4:4:4:4 little endian */
    XBGR4444 ('X', 'B', '1', '2'), /* [15:0] x:B:G:R 4:4:4:4 little endian */
    RGBX4444 ('R', 'X', '1', '2'), /* [15:0] R:G:B:x 4:4:4:4 little endian */
    BGRX4444 ('B', 'X', '1', '2'), /* [15:0] B:G:R:x 4:4:4:4 little endian */

    ARGB4444 ('A', 'R', '1', '2'), /* [15:0] A:R:G:B 4:4:4:4 little endian */
    ABGR4444 ('A', 'B', '1', '2'), /* [15:0] A:B:G:R 4:4:4:4 little endian */
    RGBA4444 ('R', 'A', '1', '2'), /* [15:0] R:G:B:A 4:4:4:4 little endian */
    BGRA4444 ('B', 'A', '1', '2'), /* [15:0] B:G:R:A 4:4:4:4 little endian */

    XRGB1555 ('X', 'R', '1', '5'), /* [15:0] x:R:G:B 1:5:5:5 little endian */
    XBGR1555 ('X', 'B', '1', '5'), /* [15:0] x:B:G:R 1:5:5:5 little endian */
    RGBX5551 ('R', 'X', '1', '5'), /* [15:0] R:G:B:x 5:5:5:1 little endian */
    BGRX5551 ('B', 'X', '1', '5'), /* [15:0] B:G:R:x 5:5:5:1 little endian */

    ARGB1555 ('A', 'R', '1', '5'), /* [15:0] A:R:G:B 1:5:5:5 little endian */
    ABGR1555 ('A', 'B', '1', '5'), /* [15:0] A:B:G:R 1:5:5:5 little endian */
    RGBA5551 ('R', 'A', '1', '5'), /* [15:0] R:G:B:A 5:5:5:1 little endian */
    BGRA5551 ('B', 'A', '1', '5'), /* [15:0] B:G:R:A 5:5:5:1 little endian */

    RGB565 ('R', 'G', '1', '6'), /* [15:0] R:G:B 5:6:5 little endian */
    BGR565 ('B', 'G', '1', '6'), /* [15:0] B:G:R 5:6:5 little endian */

    /* 24 bpp RGB */
    RGB888 ('R', 'G', '2', '4'), /* [23:0] R:G:B little endian */
    BGR888 ('B', 'G', '2', '4'), /* [23:0] B:G:R little endian */

    /* 32 bpp RGB */
    XRGB8888 ('X', 'R', '2', '4'), /* [31:0] x:R:G:B 8:8:8:8 little endian */
    XBGR8888 ('X', 'B', '2', '4'), /* [31:0] x:B:G:R 8:8:8:8 little endian */
    RGBX8888 ('R', 'X', '2', '4'), /* [31:0] R:G:B:x 8:8:8:8 little endian */
    BGRX8888 ('B', 'X', '2', '4'), /* [31:0] B:G:R:x 8:8:8:8 little endian */

    ARGB8888 ('A', 'R', '2', '4'), /* [31:0] A:R:G:B 8:8:8:8 little endian */
    ABGR8888 ('A', 'B', '2', '4'), /* [31:0] A:B:G:R 8:8:8:8 little endian */
    RGBA8888 ('R', 'A', '2', '4'), /* [31:0] R:G:B:A 8:8:8:8 little endian */
    BGRA8888 ('B', 'A', '2', '4'), /* [31:0] B:G:R:A 8:8:8:8 little endian */

    XRGB2101010 ('X', 'R', '3', '0'), /* [31:0] x:R:G:B 2:10:10:10 little endian */
    XBGR2101010 ('X', 'B', '3', '0'), /* [31:0] x:B:G:R 2:10:10:10 little endian */
    RGBX1010102 ('R', 'X', '3', '0'), /* [31:0] R:G:B:x 10:10:10:2 little endian */
    BGRX1010102 ('B', 'X', '3', '0'), /* [31:0] B:G:R:x 10:10:10:2 little endian */

    ARGB2101010 ('A', 'R', '3', '0'), /* [31:0] A:R:G:B 2:10:10:10 little endian */
    ABGR2101010 ('A', 'B', '3', '0'), /* [31:0] A:B:G:R 2:10:10:10 little endian */
    RGBA1010102 ('R', 'A', '3', '0'), /* [31:0] R:G:B:A 10:10:10:2 little endian */
    BGRA1010102 ('B', 'A', '3', '0'), /* [31:0] B:G:R:A 10:10:10:2 little endian */

    /* packed YCbCr */
    YUYV ('Y', 'U', 'Y', 'V'), /* [31:0] Cr0:Y1:Cb0:Y0 8:8:8:8 little endian */
    YVYU ('Y', 'V', 'Y', 'U'), /* [31:0] Cb0:Y1:Cr0:Y0 8:8:8:8 little endian */
    UYVY ('U', 'Y', 'V', 'Y'), /* [31:0] Y1:Cr0:Y0:Cb0 8:8:8:8 little endian */
    VYUY ('V', 'Y', 'U', 'Y'), /* [31:0] Y1:Cb0:Y0:Cr0 8:8:8:8 little endian */

    AYUV ('A', 'Y', 'U', 'V'), /* [31:0] A:Y:Cb:Cr 8:8:8:8 little endian */

    /*
     * 2 plane YCbCr
     * index 0 = Y plane, [7:0] Y
     * index 1 = Cr:Cb plane, [15:0] Cr:Cb little endian
     * or
     * index 1 = Cb:Cr plane, [15:0] Cb:Cr little endian
     */
    NV12 ('N', 'V', '1', '2'), /* 2x2 subsampled Cr:Cb plane */
    NV21 ('N', 'V', '2', '1'), /* 2x2 subsampled Cb:Cr plane */
    NV16 ('N', 'V', '1', '6'), /* 2x1 subsampled Cr:Cb plane */
    NV61 ('N', 'V', '6', '1'), /* 2x1 subsampled Cb:Cr plane */

    /*
     * 3 plane YCbCr
     * index 0: Y plane, [7:0] Y
     * index 1: Cb plane, [7:0] Cb
     * index 2: Cr plane, [7:0] Cr
     * or
     * index 1: Cr plane, [7:0] Cr
     * index 2: Cb plane, [7:0] Cb
     */
    YUV410 ('Y', 'U', 'V', '9'), /* 4x4 subsampled Cb (1) and Cr (2) planes */
    YVU410 ('Y', 'V', 'U', '9'), /* 4x4 subsampled Cr (1) and Cb (2) planes */
    YUV411 ('Y', 'U', '1', '1'), /* 4x1 subsampled Cb (1) and Cr (2) planes */
    YVU411 ('Y', 'V', '1', '1'), /* 4x1 subsampled Cr (1) and Cb (2) planes */
    YUV420 ('Y', 'U', '1', '2'), /* 2x2 subsampled Cb (1) and Cr (2) planes */
    YVU420 ('Y', 'V', '1', '2'), /* 2x2 subsampled Cr (1) and Cb (2) planes */
    YUV422 ('Y', 'U', '1', '6'), /* 2x1 subsampled Cb (1) and Cr (2) planes */
    YVU422 ('Y', 'V', '1', '6'), /* 2x1 subsampled Cr (1) and Cb (2) planes */
    YUV444 ('Y', 'U', '2', '4'), /* non-subsampled Cb (1) and Cr (2) planes */
    YVU444 ('Y', 'V', '2', '4') /* non-subsampled Cr (1) and Cb (2) planes */
}
