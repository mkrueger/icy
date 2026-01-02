//! Drag and Drop initiation for macOS.
//!
//! This module provides the ability to start drag operations from within your
//! application on macOS. It uses the native `NSDraggingSource` protocol to
//! initiate drags without requiring modifications to winit.
//!
//! # How it works
//!
//! 1. Get the `NSView` pointer from winit's window handle via `raw-window-handle`
//! 2. Create a [`DragSource`] with that pointer
//! 3. Call [`DragSource::start_drag`] during a mouse drag event
//!
//! # Example
//!
//! ```rust,ignore
//! use icy_ui_macos::DragSource;
//! use raw_window_handle::{HasWindowHandle, RawWindowHandle};
//!
//! // Get NSView from winit window
//! let handle = window.window_handle().unwrap();
//! if let RawWindowHandle::AppKit(appkit) = handle.as_raw() {
//!     let drag_source = DragSource::new(appkit.ns_view).unwrap();
//!     
//!     // Start a drag with text data
//!     drag_source.start_drag(
//!         b"Hello, World!",
//!         "text/plain",
//!         DragOperation::Copy,
//!     )?;
//! }
//! ```

use std::ptr::NonNull;
use std::sync::mpsc::{self, Receiver, Sender};

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, class, define_class, msg_send};
use objc2_app_kit::{
    NSApplication, NSDragOperation, NSDraggingContext, NSDraggingFormation, NSDraggingItem,
    NSDraggingSession, NSDraggingSource, NSEvent, NSPasteboardItem, NSPasteboardWriting, NSView,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSData, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize,
    NSString,
};

/// Errors that can occur during drag operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragError {
    /// Not running on the main thread (required for AppKit operations).
    NotMainThread,
    /// No current event available (drag must be initiated during mouse event processing).
    NoCurrentEvent,
    /// The provided view pointer is invalid.
    InvalidView,
    /// Failed to create pasteboard item.
    PasteboardError,
    /// The drag operation is not supported on this platform.
    NotSupported,
}

impl std::fmt::Display for DragError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DragError::NotMainThread => write!(f, "Not on main thread"),
            DragError::NoCurrentEvent => {
                write!(f, "No current event (call during mouse event processing)")
            }
            DragError::InvalidView => write!(f, "Invalid NSView pointer"),
            DragError::PasteboardError => write!(f, "Failed to create pasteboard item"),
            DragError::NotSupported => write!(f, "Drag operation not supported on this platform"),
        }
    }
}

impl std::error::Error for DragError {}

/// Allowed drag operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragOperation {
    bits: usize,
}

impl DragOperation {
    /// No operation allowed.
    pub const NONE: Self = Self { bits: 0 };
    /// Copy the dragged data.
    pub const COPY: Self = Self { bits: 1 };
    /// Link to the dragged data.
    pub const LINK: Self = Self { bits: 2 };
    /// Move the dragged data.
    pub const MOVE: Self = Self { bits: 16 };
    /// All operations allowed.
    pub const ALL: Self = Self {
        bits: 1 | 2 | 16, // Copy | Link | Move
    };
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
    /// The data was linked.
    Linked,
    /// The data was moved.
    Moved,
}

/// A drag source that can initiate drag operations from an NSView.
pub struct DragSource {
    view: Retained<NSView>,
    delegate: Retained<DragSourceDelegate>,
    result_receiver: Receiver<DragResult>,
}

impl DragSource {
    /// Create a new drag source for the given NSView.
    ///
    /// # Arguments
    ///
    /// * `ns_view` - A pointer to the NSView (obtained from `raw-window-handle`)
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ns_view` is a valid pointer to an NSView
    /// that will remain valid for the lifetime of this `DragSource`.
    ///
    /// # Errors
    ///
    /// Returns an error if not called from the main thread or if the view is invalid.
    pub fn new(ns_view: NonNull<std::ffi::c_void>) -> Result<Self, DragError> {
        let mtm = MainThreadMarker::new().ok_or(DragError::NotMainThread)?;

        // SAFETY: The caller guarantees the pointer is valid
        #[allow(unsafe_code)]
        let view: Retained<NSView> =
            unsafe { Retained::retain(ns_view.as_ptr().cast()) }.ok_or(DragError::InvalidView)?;

        let (sender, receiver) = mpsc::channel();
        let delegate = DragSourceDelegate::new(mtm, sender);

        Ok(DragSource {
            view,
            delegate,
            result_receiver: receiver,
        })
    }

    /// Start a drag operation with the given data.
    ///
    /// This method must be called during mouse event processing (i.e., in response
    /// to a mouse drag event). It uses `NSApplication.currentEvent` to get the
    /// initiating event.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to drag
    /// * `format` - The format of the data (e.g., "text/plain", "text/uri-list")
    /// * `allowed_operations` - Which drag operations are allowed
    ///
    /// # Returns
    ///
    /// A receiver that will receive the drag result when the operation completes.
    ///
    /// # Errors
    ///
    /// Returns an error if there's no current event or if the pasteboard operation fails.
    pub fn start_drag(
        &self,
        data: &[u8],
        format: &str,
        allowed_operations: DragOperation,
    ) -> Result<&Receiver<DragResult>, DragError> {
        self.start_drag_impl(data, format, allowed_operations)?;
        Ok(&self.result_receiver)
    }

    fn start_drag_impl(
        &self,
        data: &[u8],
        format: &str,
        allowed_operations: DragOperation,
    ) -> Result<(), DragError> {
        let mtm = MainThreadMarker::new().ok_or(DragError::NotMainThread)?;

        // Get the current event from NSApplication
        let app = NSApplication::sharedApplication(mtm);
        let event = app.currentEvent().ok_or(DragError::NoCurrentEvent)?;

        // Update the delegate with allowed operations
        self.delegate.set_allowed_operations(allowed_operations);

        // Create a pasteboard item with the data
        let pasteboard_item = create_pasteboard_item(data, format)?;

        // Create a dragging item
        let dragging_item = create_dragging_item(&pasteboard_item, &event, mtm);

        // Create the items array
        let items: Retained<NSArray<NSDraggingItem>> =
            NSArray::from_retained_slice(&[dragging_item]);

        // Start the drag session
        let session = self.view.beginDraggingSessionWithItems_event_source(
            &items,
            &event,
            ProtocolObject::from_ref(&*self.delegate),
        );

        // Configure the session
        session.setAnimatesToStartingPositionsOnCancelOrFail(true);
        session.setDraggingFormation(NSDraggingFormation::Default);

        log::debug!("DragSource: Started drag session");
        Ok(())
    }

    /// Try to receive the drag result without blocking.
    ///
    /// Returns `Some(result)` if the drag has completed, `None` otherwise.
    pub fn try_recv_result(&self) -> Option<DragResult> {
        self.result_receiver.try_recv().ok()
    }
}

// === macOS Implementation Details ===

struct DragSourceDelegateIvars {
    result_sender: Sender<DragResult>,
    allowed_operations: std::cell::Cell<usize>,
}

define_class!(
    // SAFETY: DragSourceDelegate only uses main-thread APIs and
    // NSObject has no subclassing requirements
    #[unsafe(super(NSObject))]
    #[thread_kind = objc2::MainThreadOnly]
    #[name = "IcyUiDragSourceDelegate"]
    #[ivars = DragSourceDelegateIvars]

    /// Objective-C class implementing NSDraggingSource protocol.
    struct DragSourceDelegate;

    // SAFETY: NSObjectProtocol methods are inherited from NSObject
    unsafe impl NSObjectProtocol for DragSourceDelegate {}

    // SAFETY: We implement the NSDraggingSource protocol methods
    #[allow(non_snake_case)]
    unsafe impl NSDraggingSource for DragSourceDelegate {
        /// Return the allowed operations for the drag.
        #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
        fn draggingSession_sourceOperationMaskForDraggingContext(
            &self,
            _session: &NSDraggingSession,
            context: NSDraggingContext,
        ) -> NSDragOperation {
            let allowed = self.ivars().allowed_operations.get();

            // For drags within the same app, allow all requested operations
            // For drags to other apps, also allow all (can be customized)
            match context {
                NSDraggingContext::WithinApplication => NSDragOperation::from_bits_retain(allowed),
                NSDraggingContext::OutsideApplication => NSDragOperation::from_bits_retain(allowed),
                _ => NSDragOperation::None,
            }
        }

        /// Called when the drag session ends.
        #[unsafe(method(draggingSession:endedAtPoint:operation:))]
        fn draggingSession_endedAtPoint_operation(
            &self,
            _session: &NSDraggingSession,
            _point: NSPoint,
            operation: NSDragOperation,
        ) {
            let result = operation_to_result(operation);
            log::debug!("DragSource: Drag ended with result {:?}", result);

            // Send the result (ignore errors if receiver was dropped)
            let _ = self.ivars().result_sender.send(result);
        }
    }
);

impl DragSourceDelegate {
    fn new(mtm: MainThreadMarker, sender: Sender<DragResult>) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(DragSourceDelegateIvars {
            result_sender: sender,
            allowed_operations: std::cell::Cell::new(DragOperation::ALL.bits),
        });

        // SAFETY: Calling inherited init method from NSObject
        #[allow(unsafe_code)]
        unsafe {
            objc2::msg_send![super(this), init]
        }
    }

    fn set_allowed_operations(&self, ops: DragOperation) {
        self.ivars().allowed_operations.set(ops.bits);
    }
}

/// Convert NSDragOperation to our DragResult.
fn operation_to_result(operation: NSDragOperation) -> DragResult {
    if operation.contains(NSDragOperation::Move) {
        DragResult::Moved
    } else if operation.contains(NSDragOperation::Copy) {
        DragResult::Copied
    } else if operation.contains(NSDragOperation::Link) {
        DragResult::Linked
    } else {
        DragResult::Cancelled
    }
}

/// Create an NSPasteboardItem with the given data.
fn create_pasteboard_item(
    data: &[u8],
    format: &str,
) -> Result<Retained<NSPasteboardItem>, DragError> {
    // SAFETY: NSPasteboardItem creation
    #[allow(unsafe_code)]
    let item: Retained<NSPasteboardItem> = unsafe {
        let item: Retained<NSPasteboardItem> = msg_send![class!(NSPasteboardItem), new];
        item
    };

    // Convert format to NSPasteboardType (UTI)
    let pasteboard_type = format_to_uti(format);
    let type_string = NSString::from_str(&pasteboard_type);

    // Create NSData from the bytes
    // SAFETY: NSData creation from bytes
    #[allow(unsafe_code)]
    let ns_data = unsafe { NSData::dataWithBytes_length(data.as_ptr().cast(), data.len()) };

    // Set the data on the pasteboard item
    // SAFETY: Setting data with type
    #[allow(unsafe_code)]
    let success: bool = unsafe { msg_send![&item, setData: &*ns_data, forType: &*type_string] };

    if success {
        Ok(item)
    } else {
        Err(DragError::PasteboardError)
    }
}

/// Create an NSDraggingItem from a pasteboard item.
#[cfg(target_os = "macos")]
fn create_dragging_item(
    pasteboard_item: &NSPasteboardItem,
    event: &NSEvent,
    mtm: MainThreadMarker,
) -> Retained<NSDraggingItem> {
    use objc2::runtime::ProtocolObject;

    // Convert NSPasteboardItem to &ProtocolObject<dyn NSPasteboardWriting>
    let writer: &ProtocolObject<dyn NSPasteboardWriting> =
        ProtocolObject::from_ref(pasteboard_item);

    // NSDraggingItem creation with valid pasteboard writer
    let item: Retained<NSDraggingItem> =
        NSDraggingItem::initWithPasteboardWriter(mtm.alloc(), writer);

    // Set the dragging frame (a small rectangle at the mouse position)
    let location = event.locationInWindow();
    let frame = NSRect::new(
        NSPoint::new(location.x - 16.0, location.y - 16.0),
        NSSize::new(32.0, 32.0),
    );

    // Create a simple drag image (can be enhanced later)
    // SAFETY: Setting the dragging frame
    #[allow(unsafe_code)]
    unsafe {
        item.setDraggingFrame_contents(frame, None);
    }

    item
}

/// Convert format to macOS UTI (Uniform Type Identifier).
#[cfg(target_os = "macos")]
fn format_to_uti(format: &str) -> String {
    match format {
        "text/plain" | "text/plain;charset=utf-8" => "public.utf8-plain-text".to_string(),
        "text/html" => "public.html".to_string(),
        "text/uri-list" => "public.url".to_string(),
        "image/png" => "public.png".to_string(),
        "image/jpeg" => "public.jpeg".to_string(),
        "application/json" => "public.json".to_string(),
        // For unknown types, use the MIME type directly (macOS can often handle it)
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drag_operation_bitflags() {
        let ops = DragOperation::COPY | DragOperation::MOVE;
        assert_eq!(ops.bits, 17); // 1 | 16
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_format_conversion() {
        assert_eq!(format_to_uti("text/plain"), "public.utf8-plain-text");
        assert_eq!(format_to_uti("text/uri-list"), "public.url");
        assert_eq!(format_to_uti("custom/type"), "custom/type");
    }
}
