use std::mem;
use wgpu::util::DeviceExt;

use crate::rendering::TextureView;
use super::BindingsDescriptorEntry;

pub struct Bindings<'d> {
    device: &'d wgpu::Device,
    entries: Vec<Option<BindingEntry>>,
    descriptor: Vec<BindingsDescriptorEntry>,
}

impl<'d> Bindings<'d> {
    pub(in crate::rendering) fn new(device: &'d wgpu::Device, descriptor: Vec<BindingsDescriptorEntry>) -> Self {
        let mut entries = Vec::with_capacity(descriptor.len());

        for _ in 0..descriptor.len() {
            entries.push(None);
        }

        Self {
            device,
            entries,
            descriptor,
        }
    }

    pub(in crate::rendering) fn texture_view(&mut self, texture_view: TextureView) {
        self.replace_entry(
            &BindingsDescriptorEntry::Texture,
            BindingEntry::TextureView(texture_view.view),
        ).unwrap();

        self.replace_entry(
            &BindingsDescriptorEntry::Sampler,
            BindingEntry::Sampler(self.device.create_sampler(&texture_view.sampler)),
        ).unwrap();
    }

    pub fn uniforms<U>(&mut self, uniforms: &Vec<U>) where
        U: bytemuck::Pod + bytemuck::Zeroable
    {
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniforms buffer"),
            contents: bytemuck::cast_slice(uniforms.as_slice()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.replace_entry(
            &BindingsDescriptorEntry::Uniform { size: mem::size_of::<U>() as _ },
            BindingEntry::Buffer(uniform_buffer),
        ).unwrap();
    }

    pub fn collect<'a>(&'a self) -> Result<Vec<wgpu::BindGroupEntry<'a>>, String> {
        for (i, e) in self.entries.iter().enumerate() {
            if e.is_none() {
                return Err(format!(
                    "Expecting a binding '{:?}' at index {}",
                    self.descriptor[i],
                    i
                ))
            }
        }

        Ok(self.entries
               .iter()
               .enumerate()
               .map(|(i, e)| wgpu::BindGroupEntry {
                   binding: i as u32,
                   resource: e.as_ref()
                              // safe to unwrap, it was checked already before
                              .unwrap()
                              .resource(),
               })
               .collect()
        )
    }

    fn find_entry(
        &mut self,
        descriptor: &BindingsDescriptorEntry,
    ) -> Option<&mut Option<BindingEntry>> {
        let mut index = None;

        for (i, d) in self.descriptor.iter().enumerate() {
            if d == descriptor {
                index = Some(i);
                break;
            }
        };

        match index {
            Some(i) => self.entries.get_mut(i),
            None => None
        }
    }

    fn replace_entry(
        &mut self,
        descriptor: &BindingsDescriptorEntry,
        entry: BindingEntry,
    ) -> Result<(), String> {
        match self.find_entry(descriptor) {
            Some(e) => {
                *e = Some(entry);
                Ok(())
            },
            None => Err(format!("Expecting a {:?} at bindings.", descriptor)),
        }
    }
}

enum BindingEntry {
    Buffer(wgpu::Buffer),
    TextureView(wgpu::TextureView),
    Sampler(wgpu::Sampler),
}

impl BindingEntry {
    pub fn resource<'b>(&'b self) -> wgpu::BindingResource<'b> {
        match self {
            Self::Buffer(ref buf) => buf.as_entire_binding(),
            Self::TextureView(ref tex_view) => wgpu::BindingResource::TextureView(tex_view),
            Self::Sampler(ref sampler) => wgpu::BindingResource::Sampler(sampler),
        }
    }
}
