use context;
use enumflags::BitFlags;
use errors::{BufferError, MappingError};
use std::marker::PhantomData;
use traits::BackendApi;

pub enum HostVisible {}
pub enum DeviceLocal {}

pub trait HostVisibleBuffer<T, Backend: BackendApi>
where
    Self: Sized,
    T: Copy,
{
    fn from_slice(
        context: &context::Context<Backend>,
        usage: BitFlags<BufferUsage>,
        data: &[T],
    ) -> Result<Self, BufferError>;
    fn map_memory<R, F>(&mut self, f: F) -> Result<R, MappingError>
    where
        F: Fn(&mut [T]) -> R;
}

pub trait BufferApi<T, Backend: BackendApi>
where
    Self: Sized,
    T: Copy,
{
    fn copy_to_device_local(&self) -> ImplBuffer<T, DeviceLocal, Backend>;
}

pub struct ImplBuffer<T, Property, Backend: BackendApi> {
    pub buffer: Backend::Buffer,
    pub usage: BitFlags<BufferUsage>,
    pub _m: PhantomData<T>,
    pub _property: PhantomData<Property>,
}
pub struct Buffer<T, Property, Backend: BackendApi> {
    pub impl_buffer: ImplBuffer<T, Property, Backend>,
}

impl<T: Copy, Backend> Buffer<T, HostVisible, Backend>
where
    Backend: BackendApi,
    ImplBuffer<T, HostVisible, Backend>: HostVisibleBuffer<T, Backend>
{
    pub fn from_slice(
        context: &context::Context<Backend>,
        usage: BitFlags<BufferUsage>,
        data: &[T],
    ) -> Result<Self, BufferError> {
        <ImplBuffer<T, HostVisible, Backend> as HostVisibleBuffer<T, Backend>>::from_slice(context, usage, data).map(|impl_buffer| Buffer { impl_buffer })
    }

    pub fn map_memory<R, F>(&mut self, mut f: F) -> Result<R, MappingError>
    where
        F: Fn(&mut [T]) -> R,
    {
        ImplBuffer::map_memory(&mut self.impl_buffer, f)
    }
}

impl<T, Property, Backend> ImplBuffer<T, Property, Backend> where Backend: BackendApi {}

#[derive(Copy, Clone, EnumFlags)]
#[repr(u32)]
pub enum BufferUsage {
    Vertex = 1 << 0,
    Index = 1 << 1,
    Uniform = 1 << 2,
}
