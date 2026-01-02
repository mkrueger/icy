//! Drag and Drop support for Wayland (via smithay-clipboard) and macOS (via icy_ui_macos).
//!
//! This module provides the platform integration for DnD operations.

use crate::core::dnd::{DndAction, DragData, DragIcon, DropResult};
use crate::core::window::Id as WindowId;
use crate::futures::futures::channel::oneshot;
use crate::runtime::dnd::Action;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, SendError, TryRecvError};
use std::sync::Arc;
use winit::window::Window;

#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
use smithay_clipboard::dnd::{
    DndData as SmithayDndData, DndDestinationRectangle, DndEvent, OfferEvent, Rectangle, Sender,
    SourceEvent,
};
#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
use wayland_client::protocol::wl_data_device_manager::DndAction as WlDndAction;
#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
use wayland_client::protocol::wl_surface::WlSurface;
#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
use wayland_client::{Connection, Proxy, backend::ObjectId};

#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
use smithay_clipboard::Clipboard;

/// Manages drag and drop state for all windows.
pub struct DndManager {
    state: State,
}

#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
#[allow(dead_code)]
struct State {
    /// The smithay clipboard instance (used for DnD operations)
    clipboard: Option<Clipboard>,
    /// The Wayland connection (needed to create WlSurface from raw handles)
    connection: Option<Connection>,
    /// The WlSurface for the window (needed to start drags)
    surface: Option<WlSurface>,
    /// Receiver for DnD events from smithay-clipboard
    event_receiver: Option<Receiver<DndEvent<WlSurface>>>,
    /// Active drag source (if we initiated a drag)
    active_drag: Option<ActiveDrag>,
    /// The last action selected by the compositor
    last_action: WlDndAction,
    /// Pending channels for data requests
    data_requests: HashMap<String, oneshot::Sender<Option<Vec<u8>>>>,
    /// Whether DnD has been initialized
    dnd_initialized: bool,
    /// Track last known position for drop events
    last_position: (f32, f32),
    /// Whether a surface has been registered for DnD
    surface_registered: bool,
}

/// macOS-specific DnD state using icy_ui_macos.
#[cfg(target_os = "macos")]
struct State {
    /// The drag source for initiating drags
    drag_source: Option<icy_ui_macos::DragSource>,
    /// Active drag channel (if we initiated a drag)
    active_drag: Option<ActiveDrag>,
}

/// Windows-specific DnD state using icy_ui_windows.
#[cfg(target_os = "windows")]
struct State {
    /// The drag source for initiating drags
    drag_source: Option<icy_ui_windows::DragSource>,
    /// Active drag channel (if we initiated a drag)
    #[allow(dead_code)]
    active_drag: Option<ActiveDrag>,
}

#[cfg(not(any(
    all(feature = "wayland", unix, not(target_os = "macos")),
    target_os = "macos",
    target_os = "windows"
)))]
struct State {
    unavailable: (),
}

#[allow(dead_code)]
struct ActiveDrag {
    channel: oneshot::Sender<DropResult>,
}

/// A sender adapter for mpsc that implements smithay_clipboard's Sender trait.
/// Also wakes up the winit event loop when events are received.
#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
struct WakeupSender {
    sender: std::sync::mpsc::Sender<DndEvent<WlSurface>>,
    wakeup: Arc<dyn Fn() + Send + Sync>,
}

#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
impl Sender<WlSurface> for WakeupSender {
    fn send(&self, t: DndEvent<WlSurface>) -> Result<(), SendError<DndEvent<WlSurface>>> {
        let result = self.sender.send(t).map_err(|e| SendError(e.0));
        // Wake up the event loop so it polls for DnD events
        (self.wakeup)();
        result
    }
}

impl DndManager {
    /// Create a new DnD manager for the given window.
    ///
    /// The `wakeup` callback will be called whenever DnD events are received,
    /// allowing the event loop to wake up and process them.
    pub fn connect(window: Arc<Window>, wakeup: Arc<dyn Fn() + Send + Sync>) -> Self {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            use wayland_client::backend::Backend;
            use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

            if let Ok(display_handle) = window.display_handle() {
                use winit::raw_window_handle::RawDisplayHandle;

                if let RawDisplayHandle::Wayland(wayland) = display_handle.as_raw() {
                    // Create the clipboard with the display
                    // SAFETY: The display pointer is valid for the lifetime of the window
                    #[allow(unsafe_code)]
                    let clipboard = unsafe { Clipboard::new(wayland.display.as_ptr().cast()) };

                    // Create a Wayland connection from the display
                    #[allow(unsafe_code)]
                    let backend =
                        unsafe { Backend::from_foreign_display(wayland.display.as_ptr().cast()) };
                    let connection = Connection::from_backend(backend);

                    // Create a channel for receiving DnD events
                    let (sender, receiver): (
                        mpsc::Sender<DndEvent<WlSurface>>,
                        mpsc::Receiver<DndEvent<WlSurface>>,
                    ) = mpsc::channel();
                    let sender_box: Box<dyn Sender<WlSurface> + Send> =
                        Box::new(WakeupSender { sender, wakeup });
                    clipboard.init_dnd(sender_box);

                    // Try to register the window surface for DnD
                    let mut surface_registered = false;
                    let mut stored_surface = None;
                    if let Ok(window_handle) = window.window_handle() {
                        use winit::raw_window_handle::RawWindowHandle;
                        if let RawWindowHandle::Wayland(wl_window) = window_handle.as_raw() {
                            // Create WlSurface from the raw pointer
                            #[allow(unsafe_code)]
                            match unsafe {
                                ObjectId::from_ptr(
                                    WlSurface::interface(),
                                    wl_window.surface.as_ptr().cast(),
                                )
                            } {
                                Ok(object_id) => {
                                    match WlSurface::from_id(&connection, object_id) {
                                        Ok(surface) => {
                                            // Register the entire window as a drop target accepting common types
                                            let size = window.inner_size();
                                            let rectangles = vec![DndDestinationRectangle {
                                                id: 0,
                                                rectangle: Rectangle {
                                                    x: 0.0,
                                                    y: 0.0,
                                                    width: size.width as f64,
                                                    height: size.height as f64,
                                                },
                                                mime_types: vec![
                                                    "text/plain;charset=utf-8".into(),
                                                    "text/plain".into(),
                                                    "text/uri-list".into(),
                                                    "UTF8_STRING".into(),
                                                ],
                                                actions: WlDndAction::Copy | WlDndAction::Move,
                                                preferred: WlDndAction::Copy,
                                            }];
                                            clipboard.register_dnd_destination(
                                                surface.clone(),
                                                rectangles,
                                            );
                                            stored_surface = Some(surface);
                                            surface_registered = true;
                                        }
                                        Err(_e) => {
                                            log::warn!("DnD: Failed to create WlSurface: {:?}", _e);
                                        }
                                    }
                                }
                                Err(_e) => {
                                    log::warn!("DnD: Failed to create ObjectId: {:?}", _e);
                                }
                            }
                        }
                    }

                    return DndManager {
                        state: State {
                            clipboard: Some(clipboard),
                            connection: Some(connection),
                            surface: stored_surface,
                            event_receiver: Some(receiver),
                            active_drag: None,
                            last_action: WlDndAction::None,
                            data_requests: HashMap::new(),
                            dnd_initialized: true,
                            last_position: (0.0, 0.0),
                            surface_registered,
                        },
                    };
                }
            }
            DndManager {
                state: State {
                    clipboard: None,
                    connection: None,
                    surface: None,
                    event_receiver: None,
                    active_drag: None,
                    last_action: WlDndAction::None,
                    data_requests: HashMap::new(),
                    dnd_initialized: false,
                    last_position: (0.0, 0.0),
                    surface_registered: false,
                },
            }
        }

        #[cfg(target_os = "macos")]
        {
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

            let drag_source = if let Ok(window_handle) = window.window_handle() {
                if let RawWindowHandle::AppKit(appkit) = window_handle.as_raw() {
                    match icy_ui_macos::DragSource::new(appkit.ns_view) {
                        Ok(source) => {
                            log::debug!("DnD: Created macOS drag source");
                            Some(source)
                        }
                        Err(e) => {
                            log::warn!("DnD: Failed to create macOS drag source: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let _ = wakeup; // Not needed for macOS (yet)
            DndManager {
                state: State {
                    drag_source,
                    active_drag: None,
                },
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::ptr::NonNull;
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

            let drag_source = if let Ok(window_handle) = window.window_handle() {
                if let RawWindowHandle::Win32(win32) = window_handle.as_raw() {
                    // Convert NonZero<isize> to NonNull<c_void>
                    // Safety: win32.hwnd is a valid, non-zero handle
                    let hwnd_ptr = win32.hwnd.get() as *mut std::ffi::c_void;
                    #[allow(unsafe_code)]
                    let hwnd_non_null = unsafe { NonNull::new_unchecked(hwnd_ptr) };
                    match icy_ui_windows::DragSource::new(hwnd_non_null) {
                        Ok(source) => {
                            log::debug!("DnD: Created Windows drag source");
                            Some(source)
                        }
                        Err(e) => {
                            log::warn!("DnD: Failed to create Windows drag source: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let _ = wakeup; // Not needed for Windows (yet)
            DndManager {
                state: State {
                    drag_source,
                    active_drag: None,
                },
            }
        }

        #[cfg(not(any(
            all(feature = "wayland", unix, not(target_os = "macos")),
            target_os = "macos",
            target_os = "windows"
        )))]
        {
            let _ = window;
            let _ = wakeup;
            DndManager {
                state: State { unavailable: () },
            }
        }
    }

    /// Create an unconnected DnD manager.
    pub fn unconnected() -> Self {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            DndManager {
                state: State {
                    clipboard: None,
                    connection: None,
                    surface: None,
                    event_receiver: None,
                    active_drag: None,
                    last_action: WlDndAction::None,
                    data_requests: HashMap::new(),
                    dnd_initialized: false,
                    last_position: (0.0, 0.0),
                    surface_registered: false,
                },
            }
        }

        #[cfg(target_os = "macos")]
        {
            DndManager {
                state: State {
                    drag_source: None,
                    active_drag: None,
                },
            }
        }

        #[cfg(target_os = "windows")]
        {
            DndManager {
                state: State {
                    drag_source: None,
                    active_drag: None,
                },
            }
        }

        #[cfg(not(any(
            all(feature = "wayland", unix, not(target_os = "macos")),
            target_os = "macos",
            target_os = "windows"
        )))]
        {
            DndManager {
                state: State { unavailable: () },
            }
        }
    }

    /// Check if DnD is available.
    pub fn is_available(&self) -> bool {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            self.state.clipboard.is_some() && self.state.dnd_initialized
        }

        #[cfg(target_os = "macos")]
        {
            self.state.drag_source.is_some()
        }

        #[cfg(target_os = "windows")]
        {
            self.state.drag_source.is_some()
        }

        #[cfg(not(any(
            all(feature = "wayland", unix, not(target_os = "macos")),
            target_os = "macos",
            target_os = "windows"
        )))]
        {
            false
        }
    }

    /// Poll for pending DnD events and return them as icy events.
    ///
    /// This should be called in the event loop to process incoming DnD events.
    pub fn poll_events(&mut self) -> Vec<crate::core::Event> {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            // First, collect all pending events
            let mut pending_events = Vec::new();

            if let Some(ref receiver) = self.state.event_receiver {
                loop {
                    match receiver.try_recv() {
                        Ok(event) => {
                            pending_events.push(event);
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => break,
                    }
                }
            }

            // Then process them (now we can borrow self mutably)
            let mut icy_events = Vec::new();
            for event in pending_events {
                if let Some(icy_event) = self.handle_dnd_event(event) {
                    icy_events.push(icy_event);
                }
            }

            icy_events
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, check for completed drag operations
            if let Some(ref drag_source) = self.state.drag_source {
                if let Some(result) = drag_source.try_recv_result() {
                    // Convert macOS result to our DropResult
                    let drop_result = match result {
                        icy_ui_macos::dnd::DragResult::Copied => {
                            DropResult::Dropped(DndAction::Copy)
                        }
                        icy_ui_macos::dnd::DragResult::Moved => {
                            DropResult::Dropped(DndAction::Move)
                        }
                        icy_ui_macos::dnd::DragResult::Linked => {
                            DropResult::Dropped(DndAction::Link)
                        }
                        icy_ui_macos::dnd::DragResult::Cancelled => DropResult::Cancelled,
                    };

                    // Complete the active drag if there is one
                    if let Some(drag) = self.state.active_drag.take() {
                        let _ = drag.channel.send(drop_result);
                    }
                }
            }

            // macOS receives drops via winit's native events (DroppedFile, HoveredFile)
            // No additional polling needed here
            Vec::new()
        }

        #[cfg(not(any(
            all(feature = "wayland", unix, not(target_os = "macos")),
            target_os = "macos"
        )))]
        {
            Vec::new()
        }
    }

    /// Process a DnD action.
    pub fn process(&mut self, action: Action) {
        match action {
            Action::StartDrag {
                data,
                icon,
                allowed_actions,
                channel,
            } => {
                self.start_drag(data, icon, allowed_actions, channel);
            }
            Action::SetDropZones { window, zones } => {
                self.set_drop_zones(window, zones);
            }
            Action::AcceptDrag {
                window,
                formats,
                action,
            } => {
                self.accept_drag(window, formats, action);
            }
            Action::RejectDrag { window } => {
                self.reject_drag(window);
            }
            Action::RequestData {
                window,
                format,
                channel,
            } => {
                self.request_data(window, format, channel);
            }
        }
    }

    fn start_drag(
        &mut self,
        data: DragData,
        _icon: Option<DragIcon>,
        allowed_actions: DndAction,
        channel: oneshot::Sender<DropResult>,
    ) {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            if let (Some(clipboard), Some(surface)) = (&self.state.clipboard, &self.state.surface) {
                // Convert our DragData to smithay's DndData
                let smithay_data = SmithayDndData::new(
                    data.data,
                    data.formats.iter().map(|s| s.to_string()).collect(),
                );

                // Convert DndAction to Wayland DndAction
                let wayland_action = convert_dnd_action(allowed_actions);

                // Start the drag operation
                clipboard.start_dnd(
                    surface.clone(),
                    smithay_data,
                    wayland_action,
                    None, // No icon for now
                );

                // Store the channel to receive completion notification
                self.state.active_drag = Some(ActiveDrag { channel });
                return;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(ref drag_source) = self.state.drag_source {
                // Get the first format for the drag (use text/plain as fallback)
                let format = data
                    .formats
                    .first()
                    .map(|s| s.as_ref())
                    .unwrap_or("text/plain");

                // Convert our DndAction to macOS DragOperation
                let operations = match allowed_actions {
                    DndAction::Copy => icy_ui_macos::dnd::DragOperation::COPY,
                    DndAction::Move => icy_ui_macos::dnd::DragOperation::MOVE,
                    DndAction::Link => icy_ui_macos::dnd::DragOperation::LINK,
                    _ => icy_ui_macos::dnd::DragOperation::ALL,
                };

                match drag_source.start_drag(&data.data, format, operations) {
                    Ok(_) => {
                        // Store the channel to receive completion notification
                        self.state.active_drag = Some(ActiveDrag { channel });
                        return;
                    }
                    Err(e) => {
                        log::warn!("DnD: Failed to start macOS drag: {}", e);
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(ref drag_source) = self.state.drag_source {
                // Get the first format for the drag (use text/plain as fallback)
                let format = data
                    .formats
                    .first()
                    .map(|s| s.as_ref())
                    .unwrap_or("text/plain");

                // Convert our DndAction to Windows DragOperation
                let operations = match allowed_actions {
                    DndAction::Copy => icy_ui_windows::dnd::DragOperation::COPY,
                    DndAction::Move => icy_ui_windows::dnd::DragOperation::MOVE,
                    DndAction::Link => icy_ui_windows::dnd::DragOperation::LINK,
                    _ => icy_ui_windows::dnd::DragOperation::ALL,
                };

                match drag_source.start_drag(&data.data, format, operations) {
                    Ok(result) => {
                        // Windows DnD is synchronous, convert the result and send it
                        let drop_result = match result {
                            icy_ui_windows::dnd::DragResult::Copied => {
                                DropResult::Dropped(DndAction::Copy)
                            }
                            icy_ui_windows::dnd::DragResult::Moved => {
                                DropResult::Dropped(DndAction::Move)
                            }
                            icy_ui_windows::dnd::DragResult::Linked => {
                                DropResult::Dropped(DndAction::Link)
                            }
                            icy_ui_windows::dnd::DragResult::Cancelled => DropResult::Cancelled,
                        };
                        let _ = channel.send(drop_result);
                        return;
                    }
                    Err(e) => {
                        log::warn!("DnD: Failed to start Windows drag: {}", e);
                    }
                }
            }
        }

        #[cfg(not(any(
            all(feature = "wayland", unix, not(target_os = "macos")),
            target_os = "macos",
            target_os = "windows"
        )))]
        {
            let _ = (data, allowed_actions);
        }

        // If we can't start a drag, just send cancelled
        let _ = channel.send(DropResult::Cancelled);
    }

    fn set_drop_zones(&mut self, _window: WindowId, zones: Vec<crate::core::dnd::DropZone>) {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            if let Some(ref _clipboard) = self.state.clipboard {
                // Convert our DropZones to smithay's DndDestinationRectangles
                let rectangles: Vec<DndDestinationRectangle> = zones
                    .into_iter()
                    .map(|zone| DndDestinationRectangle {
                        id: zone.id,
                        rectangle: Rectangle {
                            x: zone.x as f64,
                            y: zone.y as f64,
                            width: zone.width as f64,
                            height: zone.height as f64,
                        },
                        mime_types: zone
                            .accepted_formats
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        actions: convert_dnd_action(zone.accepted_actions),
                        preferred: convert_dnd_action(zone.preferred_action),
                    })
                    .collect();

                // TODO: Need WlSurface to register destinations
                // clipboard.register_dnd_destination(surface, rectangles);
                log::debug!("DnD set_drop_zones: {} zones prepared", rectangles.len());
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, drop zones are handled by the native drag session
            // The entire window acts as a drop target via winit's HoveredFile/DroppedFile events
            log::debug!(
                "DnD set_drop_zones: {} zones (macOS uses native drop handling)",
                zones.len()
            );
            let _ = zones;
        }

        #[cfg(not(any(
            all(feature = "wayland", unix, not(target_os = "macos")),
            target_os = "macos"
        )))]
        {
            let _ = zones;
        }
    }

    fn accept_drag(
        &mut self,
        _window: WindowId,
        _formats: Vec<Cow<'static, str>>,
        action: DndAction,
    ) {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            if let Some(ref clipboard) = self.state.clipboard {
                let wayland_action = convert_dnd_action(action);
                clipboard.set_dnd_action(wayland_action);
                log::debug!("DnD accept_drag: action set to {:?}", wayland_action);
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, drag acceptance is handled automatically by the native drag session
            log::debug!("DnD accept_drag: {:?} (macOS uses native handling)", action);
            let _ = action;
        }

        #[cfg(not(any(
            all(feature = "wayland", unix, not(target_os = "macos")),
            target_os = "macos"
        )))]
        {
            let _ = action;
        }
    }

    fn reject_drag(&mut self, _window: WindowId) {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            if let Some(ref clipboard) = self.state.clipboard {
                clipboard.set_dnd_action(WlDndAction::None);
                log::debug!("DnD reject_drag called");
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, drag rejection is handled by the native drag session
            log::debug!("DnD reject_drag (macOS uses native handling)");
        }
    }

    fn request_data(
        &mut self,
        _window: WindowId,
        _format: String,
        channel: oneshot::Sender<Option<Vec<u8>>>,
    ) {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            if let Some(ref clipboard) = self.state.clipboard {
                // Try to peek at the DnD offer data
                match clipboard.peek_dnd_offer(&_format) {
                    Ok(data) => {
                        let _ = channel.send(Some(data.data));
                    }
                    Err(e) => {
                        log::debug!("DnD request_data failed for {}: {:?}", _format, e);
                        let _ = channel.send(None);
                    }
                }
                return;
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, data is delivered directly via winit's DroppedFile events
            // For in-app drags, we'd need to implement NSPasteboard reading
            log::debug!(
                "DnD request_data: {} (macOS - not yet implemented for in-app drags)",
                _format
            );
            let _ = _format;
        }

        // If we can't request data, send None
        let _ = channel.send(None);
    }

    /// Finish the current DnD operation (accept the drop).
    #[cfg(any(
        all(feature = "wayland", unix, not(target_os = "macos")),
        target_os = "macos"
    ))]
    pub fn finish_dnd(&mut self) {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        if let Some(ref clipboard) = self.state.clipboard {
            clipboard.finish_dnd();
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, the drag session handles completion automatically
            log::debug!("DnD finish_dnd (macOS)");
        }
    }

    /// Cancel/end the current DnD operation.
    #[cfg(any(
        all(feature = "wayland", unix, not(target_os = "macos")),
        target_os = "macos"
    ))]
    pub fn end_dnd(&mut self) {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        if let Some(ref clipboard) = self.state.clipboard {
            clipboard.end_dnd();
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, the drag session handles cancellation automatically
            log::debug!("DnD end_dnd (macOS)");
        }
    }

    /// Handle DnD events from smithay-clipboard.
    ///
    /// This should be called when DnD events are received from the Wayland compositor.
    #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
    fn handle_dnd_event(&mut self, event: DndEvent<WlSurface>) -> Option<crate::core::Event> {
        use crate::core::Point;
        use crate::core::window::Event as WindowEvent;

        match event {
            DndEvent::Offer(rect_id, offer_event) => match offer_event {
                OfferEvent::Enter {
                    x, y, mime_types, ..
                } => {
                    let _ = rect_id;
                    self.state.last_position = (x as f32, y as f32);
                    Some(crate::core::Event::Window(WindowEvent::DragEntered {
                        position: Point::new(x as f32, y as f32),
                        formats: mime_types,
                    }))
                }
                OfferEvent::Motion { x, y } => {
                    self.state.last_position = (x as f32, y as f32);
                    Some(crate::core::Event::Window(WindowEvent::DragMoved {
                        position: Point::new(x as f32, y as f32),
                    }))
                }
                OfferEvent::Leave | OfferEvent::LeaveDestination => {
                    Some(crate::core::Event::Window(WindowEvent::DragLeft))
                }
                OfferEvent::Drop => {
                    // The actual data comes in OfferEvent::Data
                    None
                }
                OfferEvent::SelectedAction(action) => {
                    // Track the selected action
                    self.state.last_action = action;
                    None
                }
                OfferEvent::Data { data, mime_type } => {
                    // Deliver the data to any pending requests
                    if let Some(channel) = self.state.data_requests.remove(&mime_type) {
                        let _ = channel.send(Some(data.clone()));
                    }

                    // Emit a drop event with tracked position and action
                    let (x, y) = self.state.last_position;
                    let action = convert_wayland_action(self.state.last_action);
                    Some(crate::core::Event::Window(WindowEvent::DragDropped {
                        position: Point::new(x, y),
                        data,
                        format: mime_type,
                        action,
                    }))
                }
            },
            DndEvent::Source(source_event) => {
                match source_event {
                    SourceEvent::Finished => {
                        if let Some(drag) = self.state.active_drag.take() {
                            let action = convert_wayland_action(self.state.last_action);
                            let _ = drag.channel.send(DropResult::Dropped(action));
                        }
                    }
                    SourceEvent::Cancelled => {
                        if let Some(drag) = self.state.active_drag.take() {
                            let _ = drag.channel.send(DropResult::Cancelled);
                        }
                    }
                    SourceEvent::Action(action) => {
                        // Track the selected action for when finish comes
                        self.state.last_action = action;
                    }
                    SourceEvent::Mime(_mime) => {
                        // Track accepted mime types (could store for debugging)
                    }
                    SourceEvent::Dropped => {
                        // Wait for Finished or Cancelled
                    }
                }
                None
            }
        }
    }

    /// Handle a drag source operation completing.
    pub fn on_drag_finished(&mut self, result: DropResult) {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            if let Some(drag) = self.state.active_drag.take() {
                let _ = drag.channel.send(result);
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(drag) = self.state.active_drag.take() {
                let _ = drag.channel.send(result);
            }
        }

        #[cfg(not(any(
            all(feature = "wayland", unix, not(target_os = "macos")),
            target_os = "macos"
        )))]
        {
            let _ = result;
        }
    }
}

/// Convert our DndAction to Wayland's DndAction.
#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
fn convert_dnd_action(
    action: DndAction,
) -> wayland_client::protocol::wl_data_device_manager::DndAction {
    use wayland_client::protocol::wl_data_device_manager::DndAction as WlDndAction;

    match action {
        DndAction::None => WlDndAction::None,
        DndAction::Copy => WlDndAction::Copy,
        DndAction::Move => WlDndAction::Move,
        DndAction::Link => WlDndAction::empty(), // Wayland doesn't have Link, use empty
        DndAction::Ask => WlDndAction::Ask,
    }
}

/// Convert Wayland's DndAction to our DndAction.
#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
#[allow(dead_code)]
fn convert_wayland_action(
    action: wayland_client::protocol::wl_data_device_manager::DndAction,
) -> DndAction {
    use wayland_client::protocol::wl_data_device_manager::DndAction as WlDndAction;

    if action.contains(WlDndAction::Copy) {
        DndAction::Copy
    } else if action.contains(WlDndAction::Move) {
        DndAction::Move
    } else if action.contains(WlDndAction::Ask) {
        DndAction::Ask
    } else {
        DndAction::None
    }
}
