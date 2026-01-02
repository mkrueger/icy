//! Drag and Drop initiation for Windows.
//!
//! This module provides the ability to start drag operations from within your
//! application on Windows. It uses OLE Drag and Drop APIs to initiate drags.
//!
//! # How it works
//!
//! 1. Get the HWND from winit's window handle via `raw-window-handle`
//! 2. Create a [`DragSource`] with that window handle
//! 3. Call [`DragSource::start_drag`] during a mouse drag event
//!
//! # Example
//!
//! ```rust,ignore
//! use icy_ui_windows::DragSource;
//! use raw_window_handle::{HasWindowHandle, RawWindowHandle};
//!
//! // Get HWND from winit window
//! let handle = window.window_handle().unwrap();
//! if let RawWindowHandle::Win32(win32) = handle.as_raw() {
//!     let drag_source = DragSource::new(win32.hwnd).unwrap();
//!     
//!     // Start a drag with text data
//!     let result = drag_source.start_drag(
//!         b"Hello, World!",
//!         "text/plain",
//!         DragOperation::COPY,
//!     )?;
//! }
//! ```

use std::ptr::NonNull;

use windows::Win32::Foundation::{BOOL, E_UNEXPECTED, HWND, S_OK};
use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, CoInitializeEx, FORMATETC, IDataObject, IDataObject_Impl, STGMEDIUM,
    TYMED_HGLOBAL,
};
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GMEM_ZEROINIT, GlobalAlloc, GlobalLock, GlobalUnlock,
};
use windows::Win32::System::Ole::{
    DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_LINK, DROPEFFECT_MOVE, DROPEFFECT_NONE, DoDragDrop,
    IDropSource, IDropSource_Impl,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::core::{HRESULT, PCWSTR, implement};

// OLE DnD result codes - defined as HRESULT values
/// S_OK code returned from DoDragDrop - Drop was successful
const DRAGDROP_S_DROP: HRESULT = HRESULT(0x00040100u32 as i32);
/// S_OK code returned from DoDragDrop - Drop was cancelled  
const DRAGDROP_S_CANCEL: HRESULT = HRESULT(0x00040101u32 as i32);
/// S_OK code for using default cursors
const DRAGDROP_S_USEDEFAULTCURSORS: HRESULT = HRESULT(0x00040102u32 as i32);

// Windows CF_UNICODETEXT clipboard format
const CF_UNICODETEXT: u16 = 13;

/// Errors that can occur during drag operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragError {
    /// Failed to initialize COM.
    ComInitFailed,
    /// The provided window handle is invalid.
    InvalidWindow,
    /// Failed to create data object.
    DataObjectError,
    /// The drag operation is not supported on this platform.
    NotSupported,
    /// Windows API error.
    WindowsError(String),
}

impl std::fmt::Display for DragError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DragError::ComInitFailed => write!(f, "Failed to initialize COM"),
            DragError::InvalidWindow => write!(f, "Invalid window handle"),
            DragError::DataObjectError => write!(f, "Failed to create data object"),
            DragError::NotSupported => write!(f, "Drag operation not supported on this platform"),
            DragError::WindowsError(msg) => write!(f, "Windows error: {}", msg),
        }
    }
}

impl std::error::Error for DragError {}

/// Allowed drag operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragOperation {
    bits: u32,
}

impl DragOperation {
    /// No operation allowed.
    pub const NONE: Self = Self { bits: 0 };
    /// Copy the dragged data.
    pub const COPY: Self = Self {
        bits: DROPEFFECT_COPY.0,
    };
    /// Move the dragged data.
    pub const MOVE: Self = Self {
        bits: DROPEFFECT_MOVE.0,
    };
    /// Link to the dragged data.
    pub const LINK: Self = Self {
        bits: DROPEFFECT_LINK.0,
    };
    /// All operations allowed.
    pub const ALL: Self = Self {
        bits: DROPEFFECT_COPY.0 | DROPEFFECT_MOVE.0 | DROPEFFECT_LINK.0,
    };

    fn to_dropeffect(self) -> DROPEFFECT {
        DROPEFFECT(self.bits)
    }
}

impl std::ops::BitOr for DragOperation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

/// Result of a completed drag operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragResult {
    /// The drag was cancelled.
    Cancelled,
    /// The data was copied.
    Copied,
    /// The data was moved.
    Moved,
    /// The data was linked.
    Linked,
}

impl From<DROPEFFECT> for DragResult {
    fn from(effect: DROPEFFECT) -> Self {
        if effect.0 & DROPEFFECT_MOVE.0 != 0 {
            DragResult::Moved
        } else if effect.0 & DROPEFFECT_COPY.0 != 0 {
            DragResult::Copied
        } else if effect.0 & DROPEFFECT_LINK.0 != 0 {
            DragResult::Linked
        } else {
            DragResult::Cancelled
        }
    }
}

/// A drag source that can initiate drag operations from a window.
pub struct DragSource {
    // HWND is stored for potential future use (e.g., setting window as drag parent)
    // Currently OLE DnD doesn't require it after DoDragDrop is called
    #[allow(dead_code)]
    hwnd: HWND,
}

impl DragSource {
    /// Create a new drag source for the given window.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - A pointer to the HWND (obtained from `raw-window-handle`)
    ///
    /// # Errors
    ///
    /// Returns an error if COM initialization fails.
    pub fn new(hwnd: NonNull<std::ffi::c_void>) -> Result<Self, DragError> {
        // Initialize COM for this thread (STA mode required for OLE DnD)
        // It's OK if it's already initialized
        #[allow(unsafe_code)]
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        let hwnd = HWND(hwnd.as_ptr() as *mut _);

        Ok(DragSource { hwnd })
    }

    /// Start a drag operation with the given data.
    ///
    /// This method should be called during mouse event processing (i.e., in response
    /// to a mouse drag event).
    ///
    /// # Arguments
    ///
    /// * `data` - The data to drag
    /// * `format` - The format of the data (e.g., "text/plain", "text/uri-list")
    /// * `allowed_operations` - Which drag operations are allowed
    ///
    /// # Returns
    ///
    /// The result of the drag operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the drag operation fails to start.
    pub fn start_drag(
        &self,
        data: &[u8],
        format: &str,
        allowed_operations: DragOperation,
    ) -> Result<DragResult, DragError> {
        #[allow(unsafe_code)]
        unsafe {
            // Create our IDropSource implementation
            let drop_source: IDropSource = DropSourceImpl::new().into();

            // Create our IDataObject implementation
            let data_object: IDataObject = DataObjectImpl::new(data.to_vec(), format).into();

            // Perform the drag operation
            let mut effect = DROPEFFECT_NONE;
            let hr = DoDragDrop(
                &data_object,
                &drop_source,
                allowed_operations.to_dropeffect(),
                &mut effect,
            );

            log::debug!("DoDragDrop returned: {:?}, effect: {:?}", hr, effect);

            if hr == DRAGDROP_S_DROP {
                Ok(DragResult::from(effect))
            } else if hr == DRAGDROP_S_CANCEL {
                Ok(DragResult::Cancelled)
            } else if hr.is_ok() {
                Ok(DragResult::from(effect))
            } else {
                Err(DragError::WindowsError(format!(
                    "DoDragDrop failed: {:?}",
                    hr
                )))
            }
        }
    }
}

// === OLE IDropSource Implementation ===

#[implement(IDropSource)]
struct DropSourceImpl;

impl DropSourceImpl {
    fn new() -> Self {
        DropSourceImpl
    }
}

// MK_LBUTTON constant for mouse button state check
const MK_LBUTTON: u32 = 0x0001;

impl IDropSource_Impl for DropSourceImpl_Impl {
    fn QueryContinueDrag(&self, fescapepressed: BOOL, grfkeystate: MODIFIERKEYS_FLAGS) -> HRESULT {
        if fescapepressed.as_bool() {
            DRAGDROP_S_CANCEL
        } else if grfkeystate.0 & MK_LBUTTON == 0 {
            // Left mouse button released - complete the drop
            DRAGDROP_S_DROP
        } else {
            S_OK
        }
    }

    fn GiveFeedback(&self, _dweffect: DROPEFFECT) -> HRESULT {
        // Return DRAGDROP_S_USEDEFAULTCURSORS to use default cursors
        DRAGDROP_S_USEDEFAULTCURSORS
    }
}

// === OLE IDataObject Implementation ===

#[implement(IDataObject)]
struct DataObjectImpl {
    data: Vec<u8>,
    format: String,
    clipboard_format: u32,
}

impl DataObjectImpl {
    fn new(data: Vec<u8>, format: &str) -> Self {
        let clipboard_format = Self::get_clipboard_format(format);
        DataObjectImpl {
            data,
            format: format.to_string(),
            clipboard_format,
        }
    }

    fn get_clipboard_format(format: &str) -> u32 {
        // Map formats to Windows clipboard formats
        match format {
            "text/plain" | "text/plain;charset=utf-8" => CF_UNICODETEXT as u32,
            "text/uri-list" => {
                // Register custom format for file drops
                // Use RegisterClipboardFormatW from Win32_System_DataExchange
                #[allow(unsafe_code)]
                unsafe {
                    let format_name: Vec<u16> =
                        "UniformResourceLocatorW\0".encode_utf16().collect();
                    windows::Win32::System::DataExchange::RegisterClipboardFormatW(PCWSTR(
                        format_name.as_ptr(),
                    ))
                }
            }
            _ => {
                // Register custom format for other formats
                #[allow(unsafe_code)]
                unsafe {
                    let format_name: Vec<u16> = format!("{}\0", format).encode_utf16().collect();
                    windows::Win32::System::DataExchange::RegisterClipboardFormatW(PCWSTR(
                        format_name.as_ptr(),
                    ))
                }
            }
        }
    }
}

impl IDataObject_Impl for DataObjectImpl_Impl {
    fn GetData(&self, pformatetc: *const FORMATETC) -> windows::core::Result<STGMEDIUM> {
        #[allow(unsafe_code)]
        unsafe {
            let formatetc = &*pformatetc;

            // Check if this is our format
            if formatetc.cfFormat as u32 != self.clipboard_format {
                return Err(windows::core::Error::new(
                    windows::Win32::Foundation::DV_E_FORMATETC,
                    "Format not supported",
                ));
            }

            if formatetc.tymed & TYMED_HGLOBAL.0 as u32 == 0 {
                return Err(windows::core::Error::new(
                    windows::Win32::Foundation::DV_E_TYMED,
                    "TYMED not supported",
                ));
            }

            // Allocate global memory for the data
            let data = if self.format.starts_with("text/plain") {
                // Convert to UTF-16 for text
                let text = String::from_utf8_lossy(&self.data);
                let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                let size = utf16.len() * 2;
                let hmem = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, size)
                    .map_err(|e| windows::core::Error::new(E_UNEXPECTED, e.to_string()))?;
                let ptr = GlobalLock(hmem);
                std::ptr::copy_nonoverlapping(utf16.as_ptr() as *const u8, ptr.cast(), size);
                let _ = GlobalUnlock(hmem);
                hmem
            } else {
                // Binary data
                let size = self.data.len();
                let hmem = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, size)
                    .map_err(|e| windows::core::Error::new(E_UNEXPECTED, e.to_string()))?;
                let ptr = GlobalLock(hmem);
                std::ptr::copy_nonoverlapping(self.data.as_ptr(), ptr.cast(), size);
                let _ = GlobalUnlock(hmem);
                hmem
            };

            Ok(STGMEDIUM {
                tymed: TYMED_HGLOBAL.0 as u32,
                u: std::mem::transmute(data.0),
                pUnkForRelease: std::mem::ManuallyDrop::new(None),
            })
        }
    }

    fn GetDataHere(
        &self,
        _pformatetc: *const FORMATETC,
        _pmedium: *mut STGMEDIUM,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            windows::Win32::Foundation::E_NOTIMPL,
            "GetDataHere not implemented",
        ))
    }

    fn QueryGetData(&self, pformatetc: *const FORMATETC) -> HRESULT {
        #[allow(unsafe_code)]
        unsafe {
            let formatetc = &*pformatetc;

            if formatetc.cfFormat as u32 == self.clipboard_format
                && formatetc.tymed & TYMED_HGLOBAL.0 as u32 != 0
            {
                S_OK
            } else {
                windows::Win32::Foundation::DV_E_FORMATETC
            }
        }
    }

    fn GetCanonicalFormatEtc(
        &self,
        _pformatectin: *const FORMATETC,
        pformatetcout: *mut FORMATETC,
    ) -> HRESULT {
        #[allow(unsafe_code)]
        unsafe {
            (*pformatetcout).ptd = std::ptr::null_mut();
        }
        // DATA_S_SAMEFORMATETC
        HRESULT(0x00040130)
    }

    fn SetData(
        &self,
        _pformatetc: *const FORMATETC,
        _pmedium: *const STGMEDIUM,
        _frelease: windows::Win32::Foundation::BOOL,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            windows::Win32::Foundation::E_NOTIMPL,
            "SetData not implemented",
        ))
    }

    fn EnumFormatEtc(
        &self,
        _dwdirection: u32,
    ) -> windows::core::Result<windows::Win32::System::Com::IEnumFORMATETC> {
        Err(windows::core::Error::new(
            windows::Win32::Foundation::E_NOTIMPL,
            "EnumFormatEtc not implemented",
        ))
    }

    fn DAdvise(
        &self,
        _pformatetc: *const FORMATETC,
        _advf: u32,
        _padvsink: Option<&windows::Win32::System::Com::IAdviseSink>,
    ) -> windows::core::Result<u32> {
        Err(windows::core::Error::new(
            windows::Win32::Foundation::OLE_E_ADVISENOTSUPPORTED,
            "DAdvise not supported",
        ))
    }

    fn DUnadvise(&self, _dwconnection: u32) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            windows::Win32::Foundation::OLE_E_ADVISENOTSUPPORTED,
            "DUnadvise not supported",
        ))
    }

    fn EnumDAdvise(&self) -> windows::core::Result<windows::Win32::System::Com::IEnumSTATDATA> {
        Err(windows::core::Error::new(
            windows::Win32::Foundation::OLE_E_ADVISENOTSUPPORTED,
            "EnumDAdvise not supported",
        ))
    }
}
