/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A headless window implementation.

use crate::window_trait::WindowPortsMethods;
use euclid::{Point2D, Rotation3D, Scale, Size2D, UnknownUnit, Vector3D};
use winit;
use servo::compositing::windowing::{AnimationState, WindowEvent};
use servo::compositing::windowing::{EmbedderCoordinates, WindowMethods};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::units::{DeviceIntRect, DeviceIntSize};
use servo_media::player::context as MediaPlayerCtxt;
use servo::webrender_traits::WebrenderSurfman;
use std::cell::Cell;
use std::rc::Rc;
use surfman::Connection;
use surfman::ContextAttributes;
use surfman::ContextAttributeFlags;
use surfman::Device;
use surfman::GLApi;
use surfman::GLVersion;
use surfman::NativeWidget;
use surfman::SurfaceType;

pub struct Window {
    webrender_surfman: WebrenderSurfman,
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    device_pixels_per_px: Option<f32>,
}

impl Window {
    pub fn new(
        size: Size2D<u32, DeviceIndependentPixel>,
        device_pixels_per_px: Option<f32>,
    ) -> Rc<dyn WindowPortsMethods> {
        // Initialize surfman
        let connection = Connection::new().expect("Failed to create connection");
        let adapter = connection.create_software_adapter().expect("Failed to create adapter");
        let flags = ContextAttributeFlags::ALPHA | ContextAttributeFlags::DEPTH;
        let version = match connection.gl_api() {
            GLApi::GLES => GLVersion { major: 3, minor: 0 },
            GLApi::GL => GLVersion { major: 3, minor: 2 }
        };
        let context_attributes = ContextAttributes { flags, version };
        let size = size.to_untyped().to_i32();
        let surface_type = SurfaceType::Generic { size };
        let webrender_surfman = WebrenderSurfman::create(
            &connection,
            &adapter,
            context_attributes,
            surface_type,
        ).expect("Failed to create WR surfman");

        let window = Window {
            webrender_surfman,
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            device_pixels_per_px,
        };

        Rc::new(window)
    }

    fn servo_hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        match self.device_pixels_per_px {
            Some(device_pixels_per_px) => Scale::new(device_pixels_per_px),
            _ => Scale::new(1.0),
        }
    }
}

impl WindowPortsMethods for Window {
    fn get_events(&self) -> Vec<WindowEvent> {
        vec![]
    }

    fn has_events(&self) -> bool {
        false
    }

    fn id(&self) -> winit::WindowId {
        unsafe { winit::WindowId::dummy() }
    }

    fn page_height(&self) -> f32 {
        let height = self.webrender_surfman.context_surface_info().unwrap_or(None).unwrap_or(0);
        let dpr = self.servo_hidpi_factor();
        height as f32 * dpr.get()
    }

    fn set_fullscreen(&self, state: bool) {
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        return self.fullscreen.get();
    }

    fn is_animating(&self) -> bool {
        self.animation_state.get() == AnimationState::Animating
    }

    fn winit_event_to_servo_event(&self, _event: winit::WindowEvent) {
        // Not expecting any winit events.
    }
}

impl WindowMethods for Window {
     fn get_coordinates(&self) -> EmbedderCoordinates {
        let dpr = self.servo_hidpi_factor();
        let size = self.webrender_surfman
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| Size2D::from_untyped(info.size));
            .unwrap_or(Size2D::new(0, 0));
        let viewport = DeviceIntRect::new(Point2D::zero(), size);
        let framebuffer = DeviceIntSize::from_untyped(info.size);
        EmbedderCoordinates {
            viewport,
            framebuffer,
            window: (size, Point2D::zero()),
            screen: size,
            screen_avail: size,
            hidpi_factor: dpr,
        }
    }

     fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

     fn get_gl_context(&self) -> MediaPlayerCtxt::GlContext {
        MediaPlayerCtxt::GlContext::Unknown
    }

    fn get_native_display(&self) -> MediaPlayerCtxt::NativeDisplay {
        MediaPlayerCtxt::NativeDisplay::Unknown
    }

    fn get_gl_api(&self) -> MediaPlayerCtxt::GlApi {
        MediaPlayerCtxt::GlApi::None
    }

    fn webrender_surfman(&self) -> WebrenderSurfman {
        self.webrender_surfman.clone()
    }
}

impl webxr::glwindow::GlWindow for Window {
    fn create_native_widget(&self, _device: &Device) -> NativeWidget {
        unimplemented!()
    }

    fn get_rotation(&self) -> Rotation3D<f32, UnknownUnit, UnknownUnit> {
        Rotation3D::identity()
    }

    fn get_translation(&self) -> Vector3D<f32, UnknownUnit> {
        Vector3D::zero()
    }
}
