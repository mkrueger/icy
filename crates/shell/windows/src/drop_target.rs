//! Windows OLE Drop Target integration.
//!
//! winit's built-in Windows drag-and-drop support is file-focused. This module
//! registers an OLE `IDropTarget` so we can receive generic payloads (text,
//! HTML, images, etc.) from other applications.

#![cfg(all(target_os = "windows", feature = "dnd"))]

use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::{Arc, Once, mpsc};

use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::System::Com::{FORMATETC, IDataObject, STGMEDIUM, TYMED_HGLOBAL};
use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{
    DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_LINK, DROPEFFECT_MOVE, DROPEFFECT_NONE, IDropTarget,
    IDropTarget_Impl, OleInitialize, RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
use windows::core::{PCWSTR, implement};

// Clipboard format constants (classic Win32 values).
const CF_UNICODETEXT: u16 = 13;
const CF_HDROP: u16 = 15;
const CF_DIB: u16 = 8;
const CF_DIBV5: u16 = 17;

fn ensure_ole_initialized() {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        // Initialize OLE for this thread (STA). Many OLE DnD APIs expect OLE
        // to be initialized (OleInitialize), not just COM.
        #[allow(unsafe_code)]
        unsafe {
            let _ = OleInitialize(None);
        }
    });
}

/// Events produced by a Windows OLE `IDropTarget`.
#[derive(Debug, Clone)]
pub enum DropEvent {
    /// A drag entered the window.
    DragEntered {
        /// Cursor position, relative to the window.
        position: (f32, f32),
        /// Best-effort list of detected content formats.
        formats: Vec<String>,
    },
    /// The cursor moved while a drag is over the window.
    DragMoved {
        /// Cursor position, relative to the window.
        position: (f32, f32),
    },
    /// The drag left the window without dropping.
    DragLeft,
    /// Data was dropped on the window.
    DragDropped {
        /// Cursor position at drop time, relative to the window.
        position: (f32, f32),
        /// Dropped payload bytes.
        data: Vec<u8>,
        /// Best-effort format identifier (e.g. "text/plain", "image/png").
        format: String,
        /// Selected drop action.
        action: DropAction,
    },
    /// A file is being hovered over the window.
    FileHovered(PathBuf),
    /// A file was dropped into the window.
    FileDropped(PathBuf),
    /// Hovered files have left the window.
    FilesHoveredLeft,
}

/// The selected drop action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropAction {
    /// No action.
    None,
    /// Copy.
    Copy,
    /// Move.
    Move,
    /// Link.
    Link,
}

impl DropAction {
    /// Convert a Win32 `DROPEFFECT` to a high-level action.
    pub fn from_effect(effect: DROPEFFECT) -> Self {
        if effect.0 & DROPEFFECT_MOVE.0 != 0 {
            DropAction::Move
        } else if effect.0 & DROPEFFECT_COPY.0 != 0 {
            DropAction::Copy
        } else if effect.0 & DROPEFFECT_LINK.0 != 0 {
            DropAction::Link
        } else {
            DropAction::None
        }
    }
}

/// A registered OLE drop target.
///
/// Keep this value alive for as long as you want to receive drag & drop events.
pub struct DropTarget {
    hwnd: HWND,
    _inner: IDropTarget,
}

impl DropTarget {
    /// Registers an OLE `IDropTarget` for the given window.
    ///
    /// Returns the drop target handle and a receiver for the produced events.
    pub fn register(
        hwnd: NonNull<std::ffi::c_void>,
        wakeup: Arc<dyn Fn() + Send + Sync>,
    ) -> windows::core::Result<(Self, mpsc::Receiver<DropEvent>)> {
        ensure_ole_initialized();

        let hwnd = HWND(hwnd.as_ptr() as *mut _);
        let (sender, receiver) = mpsc::channel();

        let inner: IDropTarget = DropTargetImpl::new(hwnd, sender, wakeup).into();

        #[allow(unsafe_code)]
        unsafe {
            RegisterDragDrop(hwnd, &inner)?;
        }

        Ok((
            Self {
                hwnd,
                _inner: inner,
            },
            receiver,
        ))
    }
}

impl Drop for DropTarget {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        unsafe {
            let _ = RevokeDragDrop(self.hwnd);
        }
    }
}

#[implement(IDropTarget)]
struct DropTargetImpl {
    hwnd: HWND,
    sender: mpsc::Sender<DropEvent>,
    wakeup: Arc<dyn Fn() + Send + Sync>,

    // Registered clipboard formats we want to recognize.
    html_format: u16,
    png_format: u16,
}

impl DropTargetImpl {
    fn new(
        hwnd: HWND,
        sender: mpsc::Sender<DropEvent>,
        wakeup: Arc<dyn Fn() + Send + Sync>,
    ) -> Self {
        let html_format = register_clipboard_format("HTML Format");
        let png_format = register_clipboard_format("PNG");

        Self {
            hwnd,
            sender,
            wakeup,
            html_format,
            png_format,
        }
    }

    fn send(&self, event: DropEvent) {
        let _ = self.sender.send(event);
        (self.wakeup)();
    }

    fn window_position(&self, pt: &windows::Win32::Foundation::POINTL) -> (f32, f32) {
        // OLE provides screen coordinates. Convert to client (window-relative) coordinates
        // so that position checks like bounds.contains(position) work correctly.
        let mut point = POINT { x: pt.x, y: pt.y };

        #[allow(unsafe_code)]
        unsafe {
            // ScreenToClient modifies the point in-place
            let _ = ScreenToClient(self.hwnd, &mut point);
        }

        (point.x as f32, point.y as f32)
    }

    fn query_get_data(data: &IDataObject, cf: u16) -> bool {
        let formatetc = FORMATETC {
            cfFormat: cf,
            tymed: TYMED_HGLOBAL.0 as u32,
            ..Default::default()
        };

        #[allow(unsafe_code)]
        unsafe {
            data.QueryGetData(&formatetc).is_ok()
        }
    }

    fn enumerate_formats(&self, data: &IDataObject) -> Vec<String> {
        let mut results = Vec::new();

        if Self::query_get_data(data, CF_HDROP) {
            results.push("text/uri-list".to_string());
        }
        if Self::query_get_data(data, CF_UNICODETEXT) {
            results.push("text/plain".to_string());
        }
        if self.html_format != 0 && Self::query_get_data(data, self.html_format) {
            results.push("text/html".to_string());
        }
        if self.png_format != 0 && Self::query_get_data(data, self.png_format) {
            results.push("image/png".to_string());
        }
        if Self::query_get_data(data, CF_DIBV5) || Self::query_get_data(data, CF_DIB) {
            results.push("image/bmp".to_string());
        }

        results
    }

    fn get_stgmedium(data: &IDataObject, cf: u16) -> Option<STGMEDIUM> {
        let formatetc = FORMATETC {
            cfFormat: cf,
            tymed: TYMED_HGLOBAL.0 as u32,
            ..Default::default()
        };

        #[allow(unsafe_code)]
        unsafe {
            data.GetData(&formatetc).ok()
        }
    }

    fn get_hglobal_bytes(data: &IDataObject, cf: u16) -> Option<Vec<u8>> {
        let stg = Self::get_stgmedium(data, cf)?;

        let bytes = {
            let mut out = None;
            #[allow(unsafe_code)]
            unsafe {
                let hglobal = stg.u.hGlobal;
                if !hglobal.is_invalid() {
                    let ptr = GlobalLock(hglobal);
                    if !ptr.is_null() {
                        let size = GlobalSize(hglobal);
                        let slice = std::slice::from_raw_parts(ptr.cast::<u8>(), size as usize);
                        out = Some(slice.to_vec());
                        let _ = GlobalUnlock(hglobal);
                    }
                }
            }
            out
        };

        #[allow(unsafe_code)]
        unsafe {
            let mut stg = stg;
            ReleaseStgMedium(&mut stg);
        }

        bytes
    }

    fn get_unicode_text(data: &IDataObject) -> Option<Vec<u8>> {
        let stg = Self::get_stgmedium(data, CF_UNICODETEXT)?;

        let bytes = {
            let mut out = None;
            #[allow(unsafe_code)]
            unsafe {
                let hglobal = stg.u.hGlobal;
                if !hglobal.is_invalid() {
                    let ptr = GlobalLock(hglobal);
                    if !ptr.is_null() {
                        let size = GlobalSize(hglobal);
                        let u16_slice =
                            std::slice::from_raw_parts(ptr.cast::<u16>(), (size as usize) / 2);
                        let end = u16_slice
                            .iter()
                            .position(|c| *c == 0)
                            .unwrap_or(u16_slice.len());
                        let text = String::from_utf16_lossy(&u16_slice[..end]);
                        out = Some(text.into_bytes());
                        let _ = GlobalUnlock(hglobal);
                    }
                }
            }
            out
        };

        #[allow(unsafe_code)]
        unsafe {
            let mut stg = stg;
            ReleaseStgMedium(&mut stg);
        }

        bytes
    }

    fn get_hdrop_paths(data: &IDataObject) -> Option<Vec<PathBuf>> {
        let stg = Self::get_stgmedium(data, CF_HDROP)?;

        let paths = {
            let mut out = Vec::new();
            #[allow(unsafe_code)]
            unsafe {
                let hdrop = HDROP(stg.u.hGlobal.0);
                let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);

                for index in 0..count {
                    let len = DragQueryFileW(hdrop, index, None);
                    if len == 0 {
                        continue;
                    }

                    // `len` does not include trailing NUL.
                    let mut buffer = vec![0u16; (len as usize) + 1];
                    let copied = DragQueryFileW(hdrop, index, Some(buffer.as_mut_slice()));
                    if copied == 0 {
                        continue;
                    }

                    // `copied` is the number of UTF-16 chars copied, excluding NUL.
                    let s = String::from_utf16_lossy(&buffer[..(copied as usize)]);
                    out.push(PathBuf::from(s));
                }
            }
            out
        };

        #[allow(unsafe_code)]
        unsafe {
            let mut stg = stg;
            ReleaseStgMedium(&mut stg);
        }

        Some(paths)
    }

    fn choose_payload(&self, data: &IDataObject) -> Option<(Vec<u8>, String)> {
        if Self::query_get_data(data, CF_UNICODETEXT) {
            if let Some(bytes) = Self::get_unicode_text(data) {
                return Some((bytes, "text/plain".to_string()));
            }
        }

        if self.html_format != 0 && Self::query_get_data(data, self.html_format) {
            if let Some(bytes) = Self::get_hglobal_bytes(data, self.html_format) {
                return Some((bytes, "text/html".to_string()));
            }
        }

        if self.png_format != 0 && Self::query_get_data(data, self.png_format) {
            if let Some(bytes) = Self::get_hglobal_bytes(data, self.png_format) {
                return Some((bytes, "image/png".to_string()));
            }
        }

        if Self::query_get_data(data, CF_DIBV5) {
            if let Some(bytes) = Self::get_hglobal_bytes(data, CF_DIBV5) {
                return Some((bytes, "image/bmp".to_string()));
            }
        }

        if Self::query_get_data(data, CF_DIB) {
            if let Some(bytes) = Self::get_hglobal_bytes(data, CF_DIB) {
                return Some((bytes, "image/bmp".to_string()));
            }
        }

        None
    }
}

impl IDropTarget_Impl for DropTargetImpl_Impl {
    fn DragEnter(
        &self,
        pdataobj: Option<&IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        pt: &windows::Win32::Foundation::POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        let Some(data) = pdataobj else {
            return Ok(());
        };

        // Prefer copy.
        #[allow(unsafe_code)]
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = DROPEFFECT_COPY;
            }
        }

        let formats = self.enumerate_formats(data);
        let position = self.window_position(pt);

        self.send(DropEvent::DragEntered { position, formats });

        if DropTargetImpl::query_get_data(data, CF_HDROP) {
            if let Some(paths) = DropTargetImpl::get_hdrop_paths(data) {
                for path in paths {
                    self.send(DropEvent::FileHovered(path));
                }
            }
        }

        Ok(())
    }

    fn DragOver(
        &self,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        pt: &windows::Win32::Foundation::POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        #[allow(unsafe_code)]
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = DROPEFFECT_COPY;
            }
        }

        let position = self.window_position(pt);
        self.send(DropEvent::DragMoved { position });

        Ok(())
    }

    fn DragLeave(&self) -> windows::core::Result<()> {
        self.send(DropEvent::DragLeft);
        self.send(DropEvent::FilesHoveredLeft);
        Ok(())
    }

    fn Drop(
        &self,
        pdataobj: Option<&IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        pt: &windows::Win32::Foundation::POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        let Some(data) = pdataobj else {
            return Ok(());
        };

        let position = self.window_position(pt);

        // If files are present, emit file drop events.
        if DropTargetImpl::query_get_data(data, CF_HDROP) {
            if let Some(paths) = DropTargetImpl::get_hdrop_paths(data) {
                for path in paths {
                    self.send(DropEvent::FileDropped(path));
                }
            }

            self.send(DropEvent::FilesHoveredLeft);

            #[allow(unsafe_code)]
            unsafe {
                if !pdweffect.is_null() {
                    *pdweffect = DROPEFFECT_COPY;
                }
            }

            return Ok(());
        }

        let Some((bytes, format)) = self.choose_payload(data) else {
            self.send(DropEvent::DragDropped {
                position,
                data: Vec::new(),
                format: "unknown".to_string(),
                action: DropAction::Copy,
            });

            #[allow(unsafe_code)]
            unsafe {
                if !pdweffect.is_null() {
                    *pdweffect = DROPEFFECT_NONE;
                }
            }

            return Ok(());
        };

        self.send(DropEvent::DragDropped {
            position,
            data: bytes,
            format,
            action: DropAction::Copy,
        });

        #[allow(unsafe_code)]
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = DROPEFFECT_COPY;
            }
        }

        Ok(())
    }
}

fn register_clipboard_format(name: &str) -> u16 {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    #[allow(unsafe_code)]
    unsafe {
        let id = RegisterClipboardFormatW(PCWSTR(wide.as_ptr()));
        id.try_into().unwrap_or(0)
    }
}
