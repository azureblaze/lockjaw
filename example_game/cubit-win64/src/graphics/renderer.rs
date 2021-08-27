/*
Copyright 2021 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use crate::directx::check_result;
use crate::directx::command_list::CommandList;
use crate::directx::command_queue::DxCommandQueue;
use crate::directx::fence::Fence;
use crate::directx::swap_chain::SwapChain;
use cubit::graphics::{Color, RenderTarget};
use cubit::ApplicationComponent;
use lockjaw::{builder_modules, injectable, module, subcomponent,component_visible, Cl};
use std::cell::RefCell;
use std::ops::DerefMut;
use winapi::shared::windef::HWND;

lockjaw::prologue!("src/graphics/renderer.rs");

#[component_visible]
pub(crate) struct Renderer<'app_component> {
    component: Cl<'app_component, dyn RendererComponent<'app_component>>,
}

pub struct HwndModule {
    hwnd: HWND,
}

#[module]
impl HwndModule {
    #[provides]
    pub fn provide_hwnd(&self) -> HWND {
        self.hwnd
    }
}

#[builder_modules]
pub struct RendererBuilderModules {
    pub hwnd: HwndModule,
}

#[subcomponent(parent: ApplicationComponent, builder_modules: RendererBuilderModules)]
pub trait RendererComponent<'p> {
    fn command_queue(&self) -> &RefCell<DxCommandQueue>;

    fn command_list(&self) -> &RefCell<CommandList>;

    fn swap_chain(&self) -> &RefCell<SwapChain>;

    fn fence(&self) -> &RefCell<Fence>;
}

#[injectable]
impl<'app_component> Renderer<'app_component> {
    #[factory(visibility: "pub(crate)")]
    pub fn new<'a>(
        #[runtime] hwnd: HWND,
        renderer_component_builder: Cl<'a, dyn RendererComponentBuilder<'a>>,
    ) -> Renderer<'a> {
        let component: Cl<dyn RendererComponent> =
            renderer_component_builder.build(RendererBuilderModules {
                hwnd: HwndModule { hwnd },
            });

        Renderer {
            component
        }
    }

    pub fn initialize(&self) {
        self.component.command_list();
        self.update_render_target_views();
    }

    pub fn prepare_frame<'renderer>(&'renderer self) -> RenderTargetImpl<'app_component, 'renderer> {
        let mut swap_chain = self.component.swap_chain().borrow_mut();
        let mut frame = swap_chain.new_frame();

        self.component
            .command_list()
            .borrow()
            .reset(&frame.get_command_allocator());

        self.component
            .command_list()
            .borrow_mut()
            .render_target_barrier(frame.get_back_buffer());

        RenderTargetImpl {
            frame_index: frame.get_index(),
            renderer: self,
        }
    }

    pub fn clear(&self, index: usize, color: Color) {
        let mut swap_chain = self.component.swap_chain().borrow_mut();
        let mut frame = swap_chain.get_frame(index);
        frame.clear_render_target_view(&color);
    }

    pub fn present(&self, index: usize) {
        let mut swap_chain = self.component.swap_chain().borrow_mut();
        let mut frame = swap_chain.get_frame(index);

        self.component
            .command_list()
            .borrow_mut()
            .present_barrier(frame.get_back_buffer());

        check_result(self.component.command_list().borrow_mut().close()).unwrap();

        self.component
            .command_queue()
            .borrow_mut()
            .deref_mut()
            .execute_command_lists(
                &vec![self.component.command_list().borrow_mut().deref_mut()]
            );

        frame.present();
    }

    pub fn update_render_target_views(&self) {
        self.component
            .swap_chain()
            .borrow_mut()
            .update_render_target_views()
    }

    pub fn resize(&self, width: i32, height: i32) {
        self.component
            .command_queue()
            .borrow_mut()
            .deref_mut()
            .flush();

        self.component
            .swap_chain()
            .borrow_mut()
            .resize(width as u32, height as u32);
    }
}

pub struct RenderTargetImpl<'app_component, 'renderer>
    where
        'app_component: 'renderer,
{
    pub frame_index: usize,
    renderer: &'renderer Renderer<'app_component>,
}

impl<'app_component, 'renderer> RenderTarget for RenderTargetImpl<'app_component, 'renderer> {
    fn clear(&self, color: Color) {
        self.renderer
            .clear(self.frame_index, color);
    }

    fn present(&self) {
        self.renderer.present(self.frame_index);
    }
}
