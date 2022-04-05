// //! # The GPU VirtIO Device
// //! The GPU device requires us to read a more up-to-date VirtIO specification.
// //! I’ll be reading from version 1.1, which you can get a copy here:
// //!
// //! https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html.
// //!
// //! Specifically, chapter 5.7 “GPU Device”. This is an unaccelerated 2D device,
// //! meaning that we must usethe CPU to actually form the framebuffer, then we
// //! transfer our CPU formulated memory location to the host GPU, which is then
// //! responsible for drawing it to the screen.

// use core::mem::size_of;

// /// Each request starts with a Command Header, which in Rust looks as follows:
// #[repr(C)]
// pub struct CtrlHeader {
//     ctrl_type: CtrlType,
//     flags: u32,
//     fence_id: u64,
//     ctx_id: u32,
//     padding: u32,
// }

// #[repr(u32)]
// enum CtrlType {
//     /* 2d commands */
//     CmdGetDisplayInfo = 0x0100,
//     CmdResourceCreate2d,
//     CmdResourceUref,
//     CmdSetScanout,
//     CmdResourceFlush,
//     CmdTransferToHost2d,
//     CmdResourceAttachBacking,
//     CmdResourceDetachBacking,
//     CmdGetCapsetInfo,
//     CmdGetCapset,
//     CmdGetEdid,
//     /* cursor commands */
//     CmdUpdateCursor = 0x0300,
//     CmdMoveCursor,
//     /* success responses */
//     RespOkNoData = 0x1100,
//     RespOkDisplayInfo,
//     RespOkCapsetInfo,
//     RespOkCapset,
//     RespOkEdid,
//     /* error responses */
//     RespErrUnspec = 0x1200,
//     RespErrOutOfMemory,
//     RespErrInvalidScanoutId,
//     RespErrInvalidResourceId,
//     RespErrInvalidContextId,
//     RespErrInvalidParameter,
// }

// #[repr(u32)]
// enum Formats {
//     B8G8R8A8Unorm = 1,
//     B8G8R8X8Unorm = 2,
//     A8R8G8B8Unorm = 3,
//     X8R8G8B8Unorm = 4,
//     R8G8B8A8Unorm = 67,
//     X8B8G8R8Unorm = 68,
//     A8B8G8R8Unorm = 121,
//     R8G8B8X8Unorm = 134,
// }

// pub struct Device {
//     queue: *mut Queue,
//     dev: *mut u32,
//     idx: u16,
//     ack_used_idx: u16,
//     framebuffer: *mut Pixel,
//     width: u32,
//     height: u32,
// }

// struct Request<RqT, RpT> {
//     request: RqT,
//     response: RpT,
// }
// impl<RqT, RpT> Request<RqT, RpT> {
//     pub fn new(request: RqT) -> Self {
//         Self {
//             request,
//             response: unsafe { core::mem::zeroed() },
//         }
//     }
// }
