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
    NSWindow,
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

// =============================================================================
// DropTarget implementation for receiving drops
// =============================================================================

use std::path::PathBuf;
use std::sync::Arc;

use std::ffi::{CString, c_char, c_void};

/// Events produced by the macOS drop target.
#[derive(Debug, Clone)]
pub enum DropEvent {
    /// A drag entered the window.
    DragEntered {
        /// Cursor position, relative to the window (top-left origin).
        position: (f32, f32),
        /// List of detected content formats.
        formats: Vec<String>,
    },
    /// The cursor moved while a drag is over the window.
    DragMoved {
        /// Cursor position, relative to the window (top-left origin).
        position: (f32, f32),
    },
    /// The drag left the window without dropping.
    DragLeft,
    /// Data was dropped on the window.
    DragDropped {
        /// Cursor position at drop time, relative to the window (top-left origin).
        position: (f32, f32),
        /// Dropped payload bytes.
        data: Vec<u8>,
        /// Format identifier (e.g. "text/plain", "image/png").
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
    /// Convert an NSDragOperation to a high-level action.
    pub fn from_operation(operation: NSDragOperation) -> Self {
        if operation.contains(NSDragOperation::Move) {
            DropAction::Move
        } else if operation.contains(NSDragOperation::Copy) {
            DropAction::Copy
        } else if operation.contains(NSDragOperation::Link) {
            DropAction::Link
        } else {
            DropAction::None
        }
    }
}

/// A registered drop target for an NSView.
///
/// Keep this value alive for as long as you want to receive drag & drop events.
/// When dropped, the drop target is unregistered.
pub struct DropTarget {
    view: Retained<NSView>,
    /// Kept alive to ensure the delegate remains valid while the view is registered.
    #[allow(dead_code)]
    delegate: Retained<DropTargetDelegate>,
}

const DROP_TARGET_DELEGATE_KEY: &[u8] = b"IcyUiDropTargetDelegate\0";

impl DropTarget {
    /// Registers a drop target for the given NSView.
    ///
    /// # Arguments
    ///
    /// * `ns_view` - A pointer to the NSView (obtained from `raw-window-handle`)
    /// * `wakeup` - A callback to wake up the event loop when events are received
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ns_view` is a valid pointer to an NSView
    /// that will remain valid for the lifetime of this `DropTarget`.
    ///
    /// # Errors
    ///
    /// Returns an error if not called from the main thread or if the view is invalid.
    pub fn register(
        ns_view: NonNull<std::ffi::c_void>,
        wakeup: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<(Self, Receiver<DropEvent>), DragError> {
        let mtm = MainThreadMarker::new().ok_or(DragError::NotMainThread)?;

        // SAFETY: The caller guarantees the pointer is valid
        #[allow(unsafe_code)]
        let view: Retained<NSView> =
            unsafe { Retained::retain(ns_view.as_ptr().cast()) }.ok_or(DragError::InvalidView)?;

        let (sender, receiver) = mpsc::channel();
        let delegate = DropTargetDelegate::new(mtm, sender, wakeup);

        // Register the dragging types we accept
        // SAFETY: Objective-C method call
        #[allow(unsafe_code)]
        unsafe {
            // Create array of pasteboard types
            let types = [
                NSString::from_str("public.utf8-plain-text"),
                NSString::from_str("public.url"),
                NSString::from_str("public.file-url"),
                NSString::from_str("public.html"),
                NSString::from_str("public.png"),
                NSString::from_str("public.jpeg"),
                NSString::from_str("NSFilenamesPboardType"),
            ];
            let types_array: Retained<NSArray<NSString>> = NSArray::from_retained_slice(&types);

            // Register for dragging
            view.registerForDraggedTypes(&types_array);

            // Ensure the view actually receives NSDraggingDestination callbacks.
            // Winit's view class does not forward to an external delegate, so we
            // install a runtime subclass that forwards these methods to our
            // DropTargetDelegate stored as an associated object on the view.
            install_dragging_destination_hooks(view.as_ref());

            // Store the delegate in the view's associated objects so it stays alive
            // and receives the delegate calls. We use a custom key.
            use objc2::runtime::AnyObject;
            use std::ffi::c_void;

            let key_ptr = DROP_TARGET_DELEGATE_KEY.as_ptr().cast::<c_void>();

            // Set the delegate as an associated object (OBJC_ASSOCIATION_RETAIN_NONATOMIC = 1)
            #[allow(improper_ctypes)]
            unsafe extern "C" {
                fn objc_setAssociatedObject(
                    object: *const AnyObject,
                    key: *const c_void,
                    value: *const AnyObject,
                    policy: usize,
                );
            }
            objc_setAssociatedObject(
                view.as_ref() as *const NSView as *const AnyObject,
                key_ptr,
                delegate.as_ref() as *const DropTargetDelegate as *const AnyObject,
                1, // OBJC_ASSOCIATION_RETAIN_NONATOMIC
            );
        }

        Ok((Self { view, delegate }, receiver))
    }
}

impl Drop for DropTarget {
    fn drop(&mut self) {
        // Unregister dragging types
        #[allow(unsafe_code)]
        unsafe {
            self.view.unregisterDraggedTypes();

            // Remove the associated object
            use objc2::runtime::AnyObject;
            use std::ffi::c_void;

            let key_ptr = DROP_TARGET_DELEGATE_KEY.as_ptr().cast::<c_void>();

            #[allow(improper_ctypes)]
            unsafe extern "C" {
                fn objc_setAssociatedObject(
                    object: *const AnyObject,
                    key: *const c_void,
                    value: *const AnyObject,
                    policy: usize,
                );
            }
            objc_setAssociatedObject(
                self.view.as_ref() as *const NSView as *const AnyObject,
                key_ptr,
                std::ptr::null(),
                1,
            );
        }
    }
}

// === NSDraggingDestination hook installation ===

#[allow(unsafe_code)]
#[allow(improper_ctypes)]
#[allow(deprecated)]
unsafe extern "C" {
    fn objc_getAssociatedObject(
        object: *const objc2::runtime::AnyObject,
        key: *const c_void,
    ) -> *mut objc2::runtime::AnyObject;
    fn object_getClass(obj: *const objc2::runtime::AnyObject) -> *const objc2::runtime::Class;
    fn object_setClass(
        obj: *mut objc2::runtime::AnyObject,
        cls: *const objc2::runtime::Class,
    ) -> *const objc2::runtime::Class;
    fn objc_allocateClassPair(
        superclass: *const objc2::runtime::Class,
        name: *const c_char,
        extra_bytes: usize,
    ) -> *mut objc2::runtime::Class;
    fn objc_registerClassPair(cls: *mut objc2::runtime::Class);
    fn class_addMethod(
        cls: *mut objc2::runtime::Class,
        name: objc2::runtime::Sel,
        imp: *const c_void,
        types: *const c_char,
    ) -> objc2::runtime::Bool;
}

#[inline]
#[allow(unsafe_code)]
unsafe fn get_drop_delegate(
    this: *mut objc2::runtime::AnyObject,
) -> Option<Retained<DropTargetDelegate>> {
    // SAFETY: All operations here are valid unsafe calls within this unsafe fn.
    unsafe {
        let key_ptr = DROP_TARGET_DELEGATE_KEY.as_ptr().cast::<c_void>();
        let obj = objc_getAssociatedObject(this.cast_const(), key_ptr);
        // SAFETY: We stored a retained DropTargetDelegate under this key.
        Retained::retain(obj.cast())
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn view_dragging_entered(
    this: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    sender: *mut objc2::runtime::AnyObject,
) -> NSDragOperation {
    // SAFETY: Called from Objective-C runtime with valid pointers.
    unsafe {
        if let Some(delegate) = get_drop_delegate(this) {
            delegate.handle_dragging_entered_raw(sender)
        } else {
            NSDragOperation::empty()
        }
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn view_dragging_updated(
    this: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    sender: *mut objc2::runtime::AnyObject,
) -> NSDragOperation {
    // SAFETY: Called from Objective-C runtime with valid pointers.
    unsafe {
        if let Some(delegate) = get_drop_delegate(this) {
            delegate.handle_dragging_updated_raw(sender)
        } else {
            NSDragOperation::empty()
        }
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn view_dragging_exited(
    this: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    sender: *mut objc2::runtime::AnyObject,
) {
    // SAFETY: Called from Objective-C runtime with valid pointers.
    unsafe {
        if let Some(delegate) = get_drop_delegate(this) {
            delegate.handle_dragging_exited_raw(sender);
        }
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn view_prepare_for_drag_operation(
    this: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    sender: *mut objc2::runtime::AnyObject,
) -> objc2::runtime::Bool {
    // SAFETY: Called from Objective-C runtime with valid pointers.
    unsafe {
        if let Some(delegate) = get_drop_delegate(this) {
            delegate.handle_prepare_for_drag_operation_raw(sender)
        } else {
            objc2::runtime::Bool::NO
        }
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn view_perform_drag_operation(
    this: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    sender: *mut objc2::runtime::AnyObject,
) -> objc2::runtime::Bool {
    // SAFETY: Called from Objective-C runtime with valid pointers.
    unsafe {
        if let Some(delegate) = get_drop_delegate(this) {
            delegate.handle_perform_drag_operation_raw(sender)
        } else {
            objc2::runtime::Bool::NO
        }
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn view_conclude_drag_operation(
    this: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    sender: *mut objc2::runtime::AnyObject,
) {
    // SAFETY: Called from Objective-C runtime with valid pointers.
    unsafe {
        if let Some(delegate) = get_drop_delegate(this) {
            delegate.handle_conclude_drag_operation_raw(sender);
        }
    }
}

#[allow(unsafe_code)]
#[allow(deprecated)]
unsafe fn install_dragging_destination_hooks(view: &NSView) {
    // SAFETY: All operations here are valid unsafe calls within this unsafe fn.
    unsafe {
        let view_obj: *mut objc2::runtime::AnyObject =
            view as *const NSView as *mut objc2::runtime::AnyObject;
        let superclass = object_getClass(view_obj);

        if superclass.is_null() {
            return;
        }

        let superclass_ref: &objc2::runtime::Class = &*superclass;

        // Create a unique subclass name per superclass.
        // This avoids mutating winit's view class globally.
        let superclass_name = superclass_ref.name().to_string_lossy();
        let subclass_name_str = format!("IcyUiDropTargetView_{}", superclass_name);
        let subclass_name = CString::new(subclass_name_str).expect("valid objc class name");

        let subclass: *const objc2::runtime::Class =
            if let Some(existing) = objc2::runtime::Class::get(subclass_name.as_c_str()) {
                existing as *const objc2::runtime::Class
            } else {
                let cls = objc_allocateClassPair(superclass, subclass_name.as_ptr(), 0);
                if cls.is_null() {
                    return;
                }

                // NSDragOperation return: NSUInteger (Q on 64-bit)
                // BOOL return: signed char (c)
                const ENC_DRAG_OP: &[u8] = b"Q@:@\0";
                const ENC_VOID: &[u8] = b"v@:@\0";
                const ENC_BOOL: &[u8] = b"c@:@\0";

                let _ = class_addMethod(
                    cls,
                    objc2::sel!(draggingEntered:),
                    view_dragging_entered as *const c_void,
                    ENC_DRAG_OP.as_ptr().cast(),
                );
                let _ = class_addMethod(
                    cls,
                    objc2::sel!(draggingUpdated:),
                    view_dragging_updated as *const c_void,
                    ENC_DRAG_OP.as_ptr().cast(),
                );
                let _ = class_addMethod(
                    cls,
                    objc2::sel!(draggingExited:),
                    view_dragging_exited as *const c_void,
                    ENC_VOID.as_ptr().cast(),
                );
                let _ = class_addMethod(
                    cls,
                    objc2::sel!(prepareForDragOperation:),
                    view_prepare_for_drag_operation as *const c_void,
                    ENC_BOOL.as_ptr().cast(),
                );
                let _ = class_addMethod(
                    cls,
                    objc2::sel!(performDragOperation:),
                    view_perform_drag_operation as *const c_void,
                    ENC_BOOL.as_ptr().cast(),
                );
                let _ = class_addMethod(
                    cls,
                    objc2::sel!(concludeDragOperation:),
                    view_conclude_drag_operation as *const c_void,
                    ENC_VOID.as_ptr().cast(),
                );

                objc_registerClassPair(cls);
                cls
            };

        let _ = object_setClass(view_obj, subclass);
    }
}

// === Drop Target Delegate Implementation ===

use objc2_app_kit::{NSDraggingDestination, NSDraggingInfo, NSPasteboard};

struct DropTargetDelegateIvars {
    sender: Sender<DropEvent>,
    wakeup: Arc<dyn Fn() + Send + Sync>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = objc2::MainThreadOnly]
    #[name = "IcyUiDropTargetDelegate"]
    #[ivars = DropTargetDelegateIvars]

    /// Objective-C class implementing NSDraggingDestination protocol.
    struct DropTargetDelegate;

    // SAFETY: NSObjectProtocol methods are inherited from NSObject
    unsafe impl NSObjectProtocol for DropTargetDelegate {}

    // SAFETY: We implement the NSDraggingDestination protocol methods
    #[allow(non_snake_case)]
    unsafe impl NSDraggingDestination for DropTargetDelegate {
        /// Called when a drag enters the view.
        #[unsafe(method(draggingEntered:))]
        fn draggingEntered(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
            let position = self.get_position(sender);
            let formats = self.get_formats(sender);

            // Check for file URLs and emit FileHovered events
            if let Some(paths) = self.get_file_paths(sender) {
                for path in paths {
                    self.send(DropEvent::FileHovered(path));
                }
            }

            self.send(DropEvent::DragEntered { position, formats });

            // Accept copy by default
            NSDragOperation::Copy
        }

        /// Called when a drag moves within the view.
        #[unsafe(method(draggingUpdated:))]
        fn draggingUpdated(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
            let position = self.get_position(sender);
            self.send(DropEvent::DragMoved { position });

            // Continue accepting copy
            NSDragOperation::Copy
        }

        /// Called when a drag exits the view.
        #[unsafe(method(draggingExited:))]
        fn draggingExited(&self, _sender: Option<&ProtocolObject<dyn NSDraggingInfo>>) {
            self.send(DropEvent::DragLeft);
            self.send(DropEvent::FilesHoveredLeft);
        }

        /// Called to prepare for a drop.
        #[unsafe(method(prepareForDragOperation:))]
        fn prepareForDragOperation(
            &self,
            _sender: &ProtocolObject<dyn NSDraggingInfo>,
        ) -> objc2::runtime::Bool {
            objc2::runtime::Bool::YES
        }

        /// Called when a drop occurs.
        #[unsafe(method(performDragOperation:))]
        fn performDragOperation(
            &self,
            sender: &ProtocolObject<dyn NSDraggingInfo>,
        ) -> objc2::runtime::Bool {
            let position = self.get_position(sender);
            let operation = sender.draggingSourceOperationMask();

            // Check for file drops first
            if let Some(paths) = self.get_file_paths(sender) {
                for path in paths {
                    self.send(DropEvent::FileDropped(path));
                }
                self.send(DropEvent::FilesHoveredLeft);
                return objc2::runtime::Bool::YES;
            }

            // Try to get other data
            if let Some((data, format)) = self.get_payload(sender) {
                self.send(DropEvent::DragDropped {
                    position,
                    data,
                    format,
                    action: DropAction::from_operation(operation),
                });
                return objc2::runtime::Bool::YES;
            }

            // No data found
            self.send(DropEvent::DragDropped {
                position,
                data: Vec::new(),
                format: "unknown".to_string(),
                action: DropAction::None,
            });

            objc2::runtime::Bool::NO
        }

        /// Called after a drop completes.
        #[unsafe(method(concludeDragOperation:))]
        fn concludeDragOperation(&self, _sender: Option<&ProtocolObject<dyn NSDraggingInfo>>) {
            // Nothing to do here
        }
    }
);

impl DropTargetDelegate {
    fn new(
        mtm: MainThreadMarker,
        sender: Sender<DropEvent>,
        wakeup: Arc<dyn Fn() + Send + Sync>,
    ) -> Retained<Self> {
        let this = mtm
            .alloc::<Self>()
            .set_ivars(DropTargetDelegateIvars { sender, wakeup });

        // SAFETY: Calling inherited init method from NSObject
        #[allow(unsafe_code)]
        unsafe {
            objc2::msg_send![super(this), init]
        }
    }

    fn send(&self, event: DropEvent) {
        let _ = self.ivars().sender.send(event);
        (self.ivars().wakeup)();
    }

    #[allow(unsafe_code)]
    fn handle_dragging_entered_raw(
        &self,
        sender: *mut objc2::runtime::AnyObject,
    ) -> NSDragOperation {
        let position = self.get_position_raw(sender);
        let pasteboard = self.get_pasteboard_raw(sender);
        let formats = self.get_formats_from_pasteboard(&pasteboard);

        if let Some(paths) = self.get_file_paths_from_pasteboard(&pasteboard) {
            for path in paths {
                self.send(DropEvent::FileHovered(path));
            }
        }

        self.send(DropEvent::DragEntered { position, formats });
        NSDragOperation::Copy
    }

    #[allow(unsafe_code)]
    fn handle_dragging_updated_raw(
        &self,
        sender: *mut objc2::runtime::AnyObject,
    ) -> NSDragOperation {
        let position = self.get_position_raw(sender);
        self.send(DropEvent::DragMoved { position });
        NSDragOperation::Copy
    }

    #[allow(unsafe_code)]
    fn handle_dragging_exited_raw(&self, _sender: *mut objc2::runtime::AnyObject) {
        self.send(DropEvent::DragLeft);
        self.send(DropEvent::FilesHoveredLeft);
    }

    #[allow(unsafe_code)]
    fn handle_prepare_for_drag_operation_raw(
        &self,
        _sender: *mut objc2::runtime::AnyObject,
    ) -> objc2::runtime::Bool {
        objc2::runtime::Bool::YES
    }

    #[allow(unsafe_code)]
    fn handle_perform_drag_operation_raw(
        &self,
        sender: *mut objc2::runtime::AnyObject,
    ) -> objc2::runtime::Bool {
        let position = self.get_position_raw(sender);
        let operation = self.get_operation_mask_raw(sender);
        let pasteboard = self.get_pasteboard_raw(sender);

        if let Some(paths) = self.get_file_paths_from_pasteboard(&pasteboard) {
            for path in paths {
                self.send(DropEvent::FileDropped(path));
            }
            self.send(DropEvent::FilesHoveredLeft);
            return objc2::runtime::Bool::YES;
        }

        if let Some((data, format)) = self.get_payload_from_pasteboard(&pasteboard) {
            self.send(DropEvent::DragDropped {
                position,
                data,
                format,
                action: DropAction::from_operation(operation),
            });
            return objc2::runtime::Bool::YES;
        }

        self.send(DropEvent::DragDropped {
            position,
            data: Vec::new(),
            format: "unknown".to_string(),
            action: DropAction::None,
        });

        objc2::runtime::Bool::NO
    }

    #[allow(unsafe_code)]
    fn handle_conclude_drag_operation_raw(&self, _sender: *mut objc2::runtime::AnyObject) {
        // no-op
    }

    #[allow(unsafe_code)]
    fn get_pasteboard_raw(&self, sender: *mut objc2::runtime::AnyObject) -> Retained<NSPasteboard> {
        unsafe { msg_send![sender, draggingPasteboard] }
    }

    #[allow(unsafe_code)]
    fn get_operation_mask_raw(&self, sender: *mut objc2::runtime::AnyObject) -> NSDragOperation {
        unsafe { msg_send![sender, draggingSourceOperationMask] }
    }

    #[allow(unsafe_code)]
    fn get_position_raw(&self, sender: *mut objc2::runtime::AnyObject) -> (f32, f32) {
        let location: NSPoint = unsafe { msg_send![sender, draggingLocation] };
        let window: Option<Retained<NSWindow>> =
            unsafe { msg_send![sender, draggingDestinationWindow] };

        if let Some(window) = window {
            let frame = window.frame();
            let y = frame.size.height - location.y;
            (location.x as f32, y as f32)
        } else {
            (location.x as f32, location.y as f32)
        }
    }

    fn get_position(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> (f32, f32) {
        // Get the dragging location (in window coordinates, bottom-left origin)
        let location = sender.draggingLocation();

        // Get the destination window to convert coordinates
        if let Some(window) = sender.draggingDestinationWindow() {
            // Convert from bottom-left origin to top-left origin
            let frame = window.frame();
            let y = frame.size.height - location.y;
            (location.x as f32, y as f32)
        } else {
            (location.x as f32, location.y as f32)
        }
    }

    fn get_formats(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> Vec<String> {
        let pasteboard = sender.draggingPasteboard();
        self.get_formats_from_pasteboard(&pasteboard)
    }

    fn get_formats_from_pasteboard(&self, pasteboard: &NSPasteboard) -> Vec<String> {
        let mut formats = Vec::new();

        // Check for various types
        if self.has_type(pasteboard, "public.utf8-plain-text") {
            formats.push("text/plain".to_string());
        }
        if self.has_type(pasteboard, "public.url") || self.has_type(pasteboard, "public.file-url") {
            formats.push("text/uri-list".to_string());
        }
        if self.has_type(pasteboard, "public.html") {
            formats.push("text/html".to_string());
        }
        if self.has_type(pasteboard, "public.png") {
            formats.push("image/png".to_string());
        }
        if self.has_type(pasteboard, "public.jpeg") {
            formats.push("image/jpeg".to_string());
        }
        if self.has_type(pasteboard, "NSFilenamesPboardType") {
            formats.push("text/uri-list".to_string());
        }

        formats
    }

    fn has_type(&self, pasteboard: &NSPasteboard, uti: &str) -> bool {
        let type_string = NSString::from_str(uti);
        let types = pasteboard.types();
        if let Some(types) = types {
            for i in 0..types.count() {
                let t: Retained<NSString> = types.objectAtIndex(i);
                if t.isEqualToString(&type_string) {
                    return true;
                }
            }
        }
        false
    }

    fn get_file_paths(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> Option<Vec<PathBuf>> {
        let pasteboard = sender.draggingPasteboard();
        self.get_file_paths_from_pasteboard(&pasteboard)
    }

    fn get_file_paths_from_pasteboard(&self, pasteboard: &NSPasteboard) -> Option<Vec<PathBuf>> {
        // Try NSFilenamesPboardType first
        let filenames_type = NSString::from_str("NSFilenamesPboardType");
        #[allow(unsafe_code)]
        let property_list: Option<Retained<NSArray<NSString>>> =
            unsafe { msg_send![pasteboard, propertyListForType: &*filenames_type] };

        if let Some(paths_array) = property_list {
            let mut paths = Vec::new();
            for i in 0..paths_array.count() {
                let path_str: Retained<NSString> = paths_array.objectAtIndex(i);
                paths.push(PathBuf::from(path_str.to_string()));
            }
            if !paths.is_empty() {
                return Some(paths);
            }
        }

        // Try public.file-url
        let file_url_type = NSString::from_str("public.file-url");
        #[allow(unsafe_code)]
        let url_string: Option<Retained<NSString>> =
            unsafe { msg_send![pasteboard, stringForType: &*file_url_type] };

        if let Some(url) = url_string {
            let url_str = url.to_string();
            if url_str.starts_with("file://") {
                let path = url_str.strip_prefix("file://").unwrap_or(&url_str);
                // URL decode the path
                let decoded = percent_decode(path);
                return Some(vec![PathBuf::from(decoded)]);
            }
        }

        None
    }

    fn get_payload(
        &self,
        sender: &ProtocolObject<dyn NSDraggingInfo>,
    ) -> Option<(Vec<u8>, String)> {
        let pasteboard = sender.draggingPasteboard();
        self.get_payload_from_pasteboard(&pasteboard)
    }

    fn get_payload_from_pasteboard(&self, pasteboard: &NSPasteboard) -> Option<(Vec<u8>, String)> {
        // Try text first
        let text_type = NSString::from_str("public.utf8-plain-text");
        #[allow(unsafe_code)]
        let text: Option<Retained<NSString>> =
            unsafe { msg_send![pasteboard, stringForType: &*text_type] };
        if let Some(text) = text {
            return Some((text.to_string().into_bytes(), "text/plain".to_string()));
        }

        // Try URL
        let url_type = NSString::from_str("public.url");
        #[allow(unsafe_code)]
        let url: Option<Retained<NSString>> =
            unsafe { msg_send![pasteboard, stringForType: &*url_type] };
        if let Some(url) = url {
            return Some((url.to_string().into_bytes(), "text/uri-list".to_string()));
        }

        // Try HTML
        let html_type = NSString::from_str("public.html");
        #[allow(unsafe_code)]
        let html: Option<Retained<NSData>> =
            unsafe { msg_send![pasteboard, dataForType: &*html_type] };
        if let Some(html) = html {
            #[allow(unsafe_code)]
            let bytes: *const u8 = unsafe { msg_send![&html, bytes] };
            #[allow(unsafe_code)]
            let len: usize = unsafe { msg_send![&html, length] };
            #[allow(unsafe_code)]
            let slice = unsafe { std::slice::from_raw_parts(bytes, len) };
            return Some((slice.to_vec(), "text/html".to_string()));
        }

        // Try PNG
        let png_type = NSString::from_str("public.png");
        #[allow(unsafe_code)]
        let png: Option<Retained<NSData>> =
            unsafe { msg_send![pasteboard, dataForType: &*png_type] };
        if let Some(png) = png {
            #[allow(unsafe_code)]
            let bytes: *const u8 = unsafe { msg_send![&png, bytes] };
            #[allow(unsafe_code)]
            let len: usize = unsafe { msg_send![&png, length] };
            #[allow(unsafe_code)]
            let slice = unsafe { std::slice::from_raw_parts(bytes, len) };
            return Some((slice.to_vec(), "image/png".to_string()));
        }

        None
    }
}

/// Simple percent-decoding for file URLs.
fn percent_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let mut hex = String::new();
            if let Some(&h1) = chars.peek() {
                hex.push(h1);
                let _ = chars.next();
            }
            if let Some(&h2) = chars.peek() {
                hex.push(h2);
                let _ = chars.next();
            }
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }

    result
}
