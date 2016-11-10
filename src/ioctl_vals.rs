
#![allow(dead_code)]

use std::os::raw::c_ulong;

pub const DRM_IOCTL_VERSION: c_ulong = 3225445376;
pub const DRM_IOCTL_GET_UNIQUE: c_ulong = 3222299649;
pub const DRM_IOCTL_GET_MAGIC: c_ulong = 2147771394;
pub const DRM_IOCTL_IRQ_BUSID: c_ulong = 3222299651;
pub const DRM_IOCTL_GET_MAP: c_ulong = 3223872516;
pub const DRM_IOCTL_GET_CLIENT: c_ulong = 3223872517;
pub const DRM_IOCTL_GET_STATS: c_ulong = 2163762182;
pub const DRM_IOCTL_SET_VERSION: c_ulong = 3222299655;
pub const DRM_IOCTL_MODESET_CTL: c_ulong = 1074291720;
pub const DRM_IOCTL_GEM_CLOSE: c_ulong = 1074291721;
pub const DRM_IOCTL_GEM_FLINK: c_ulong = 3221775370;
pub const DRM_IOCTL_GEM_OPEN: c_ulong = 3222299659;
pub const DRM_IOCTL_GET_CAP: c_ulong = 3222299660;
pub const DRM_IOCTL_SET_CLIENT_CAP: c_ulong = 1074816013;

pub const DRM_IOCTL_SET_UNIQUE: c_ulong = 1074816016;
pub const DRM_IOCTL_AUTH_MAGIC: c_ulong = 1074029585;
pub const DRM_IOCTL_BLOCK: c_ulong = 3221513234;
pub const DRM_IOCTL_UNBLOCK: c_ulong = 3221513235;
pub const DRM_IOCTL_CONTROL: c_ulong = 1074291732;
pub const DRM_IOCTL_ADD_MAP: c_ulong = 3223872533;
pub const DRM_IOCTL_ADD_BUFS: c_ulong = 3223348246;
pub const DRM_IOCTL_MARK_BUFS: c_ulong = 1075864599;
pub const DRM_IOCTL_INFO_BUFS: c_ulong = 3222299672;
pub const DRM_IOCTL_MAP_BUFS: c_ulong = 3222823961;
pub const DRM_IOCTL_FREE_BUFS: c_ulong = 1074816026;

pub const DRM_IOCTL_RM_MAP: c_ulong = 1076388891;

pub const DRM_IOCTL_SET_SAREA_CTX: c_ulong = 1074816028;
pub const DRM_IOCTL_GET_SAREA_CTX: c_ulong = 3222299677;

pub const DRM_IOCTL_SET_MASTER: c_ulong = 25630;
pub const DRM_IOCTL_DROP_MASTER: c_ulong = 25631;

pub const DRM_IOCTL_ADD_CTX: c_ulong = 3221775392;
pub const DRM_IOCTL_RM_CTX: c_ulong = 3221775393;
pub const DRM_IOCTL_MOD_CTX: c_ulong = 1074291746;
pub const DRM_IOCTL_GET_CTX: c_ulong = 3221775395;
pub const DRM_IOCTL_SWITCH_CTX: c_ulong = 1074291748;
pub const DRM_IOCTL_NEW_CTX: c_ulong = 1074291749;
pub const DRM_IOCTL_RES_CTX: c_ulong = 3222299686;
pub const DRM_IOCTL_ADD_DRAW: c_ulong = 3221513255;
pub const DRM_IOCTL_RM_DRAW: c_ulong = 3221513256;
pub const DRM_IOCTL_DMA: c_ulong = 3225445417;
pub const DRM_IOCTL_LOCK: c_ulong = 1074291754;
pub const DRM_IOCTL_UNLOCK: c_ulong = 1074291755;
pub const DRM_IOCTL_FINISH: c_ulong = 1074291756;

pub const DRM_IOCTL_PRIME_HANDLE_TO_FD: c_ulong = 3222037549;
pub const DRM_IOCTL_PRIME_FD_TO_HANDLE: c_ulong = 3222037550;

pub const DRM_IOCTL_AGP_ACQUIRE: c_ulong = 25648;
pub const DRM_IOCTL_AGP_RELEASE: c_ulong = 25649;
pub const DRM_IOCTL_AGP_ENABLE: c_ulong = 1074291762;
pub const DRM_IOCTL_AGP_INFO: c_ulong = 2151179315;
pub const DRM_IOCTL_AGP_ALLOC: c_ulong = 3223348276;
pub const DRM_IOCTL_AGP_FREE: c_ulong = 1075864629;
pub const DRM_IOCTL_AGP_BIND: c_ulong = 1074816054;
pub const DRM_IOCTL_AGP_UNBIND: c_ulong = 1074816055;

pub const DRM_IOCTL_SG_ALLOC: c_ulong = 3222299704;
pub const DRM_IOCTL_SG_FREE: c_ulong = 1074816057;

pub const DRM_IOCTL_WAIT_VBLANK: c_ulong = 3222823994;

pub const DRM_IOCTL_UPDATE_DRAW: c_ulong = 1075340351;

pub const DRM_IOCTL_MODE_GETRESOURCES: c_ulong = 3225445536;
pub const DRM_IOCTL_MODE_GETCRTC: c_ulong = 3228066977;
pub const DRM_IOCTL_MODE_SETCRTC: c_ulong = 3228066978;
pub const DRM_IOCTL_MODE_CURSOR: c_ulong = 3223086243;
pub const DRM_IOCTL_MODE_GETGAMMA: c_ulong = 3223348388;
pub const DRM_IOCTL_MODE_SETGAMMA: c_ulong = 3223348389;
pub const DRM_IOCTL_MODE_GETENCODER: c_ulong = 3222561958;
pub const DRM_IOCTL_MODE_GETCONNECTOR: c_ulong = 3226494119;
pub const DRM_IOCTL_MODE_ATTACHMODE: c_ulong = 3225969832;
pub const DRM_IOCTL_MODE_DETACHMODE: c_ulong = 3225969833;

pub const DRM_IOCTL_MODE_GETPROPERTY: c_ulong = 3225445546;
pub const DRM_IOCTL_MODE_SETPROPERTY: c_ulong = 3222299819;
pub const DRM_IOCTL_MODE_GETPROPBLOB: c_ulong = 3222299820;
pub const DRM_IOCTL_MODE_GETFB: c_ulong = 3223086253;
pub const DRM_IOCTL_MODE_ADDFB: c_ulong = 3223086254;
pub const DRM_IOCTL_MODE_RMFB: c_ulong = 3221513391;
pub const DRM_IOCTL_MODE_PAGE_FLIP: c_ulong = 3222824112;
pub const DRM_IOCTL_MODE_DIRTYFB: c_ulong = 3222824113;

pub const DRM_IOCTL_MODE_CREATE_DUMB: c_ulong = 3223348402;
pub const DRM_IOCTL_MODE_MAP_DUMB: c_ulong = 3222299827;
pub const DRM_IOCTL_MODE_DESTROY_DUMB: c_ulong = 3221513396;
pub const DRM_IOCTL_MODE_GETPLANERESOURCES: c_ulong = 3222299829;
pub const DRM_IOCTL_MODE_GETPLANE: c_ulong = 3223348406;
pub const DRM_IOCTL_MODE_SETPLANE: c_ulong = 3224396983;
pub const DRM_IOCTL_MODE_ADDFB2: c_ulong = 3228067000;
pub const DRM_IOCTL_MODE_OBJ_GETPROPERTIES: c_ulong = 3223348409;
pub const DRM_IOCTL_MODE_OBJ_SETPROPERTY: c_ulong = 3222824122;
pub const DRM_IOCTL_MODE_CURSOR2: c_ulong = 3223610555;
pub const DRM_IOCTL_MODE_ATOMIC: c_ulong = 3224921276;
pub const DRM_IOCTL_MODE_CREATEPROPBLOB: c_ulong = 3222299837;
pub const DRM_IOCTL_MODE_DESTROYPROPBLOB: c_ulong = 3221513406;
