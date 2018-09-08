use backend::BackendApi;
use buffer::GenericBuffer;
use context::Context;
use downcast::Downcast;
use parking_lot::{Mutex, MutexGuard};
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, Drop};
use std::sync::Arc;
pub trait CreateDescriptor {
    fn create_descriptor(
        &self,
        data: &[Binding<DescriptorType>],
        sizes: DescriptorSizes,
    ) -> InnerDescriptor;
}

pub trait CreatePool {
    fn create_pool(
        &self,
        alloc_size: u32,
        data: &[Binding<DescriptorType>],
        sizes: DescriptorSizes,
    ) -> InnerPool;
}
pub trait PoolApi {
    fn create_descriptor(&self) -> InnerDescriptor;
    fn reset(&mut self);
}

pub struct InnerPool {
    pub inner: Box<dyn PoolApi>,
}

pub struct LinearPoolAllocator {
    ctx: Context,
    block_size: usize,
    pools: Vec<InnerPool>,
    // Infos
    layout: Vec<Binding<DescriptorType>>,
    sizes: DescriptorSizes,
}

impl LinearPoolAllocator {
    pub fn new<T>(ctx: &Context) -> Self
    where
        T: DescriptorInfo,
    {
        LinearPoolAllocator {
            ctx: ctx.clone(),
            block_size: 50,
            pools: Vec::new(),
            layout: T::layout(),
            sizes: T::sizes(),
        }
    }

    pub fn allocate_additional_pool(&mut self) {
        let pool = self
            .ctx
            .create_pool(self.block_size as u32, &self.layout, self.sizes);
        self.pools.push(pool);
    }

    pub fn reset(&mut self) {
        for pool in &mut self.pools {
            pool.inner.reset();
        }
    }
}

pub struct Allocator<'pool, T: 'static> {
    allocator: MutexGuard<'pool, LinearPoolAllocator>,
    current_allocations: usize,
    _m: PhantomData<T>,
}

impl<'a, T> Drop for Allocator<'a, T> {
    fn drop(&mut self) {
        self.allocator.reset();
    }
}

impl<'pool, T> Allocator<'pool, T>
where
    T: DescriptorInfo,
{
    pub fn allocate<'alloc>(&'alloc mut self) -> Descriptor<'alloc, T> {
        let allocator = &mut self.allocator;
        let allocator_index = self.current_allocations / allocator.block_size;
        // If we don't have enough space, we need to allocate a new pool
        if allocator_index >= allocator.pools.len() {
            allocator.allocate_additional_pool();
        }
        let inner_descriptor = allocator.pools[allocator_index].inner.create_descriptor();
        self.current_allocations += 1;
        Descriptor {
            inner_descriptor,
            _m: PhantomData,
        }
    }
}

pub type PoolAllocator = Arc<Mutex<LinearPoolAllocator>>;
pub struct Pool<T> {
    ctx: Context,
    allocator: PoolAllocator,
    _m: PhantomData<T>,
}

impl<T> Pool<T>
where
    T: DescriptorInfo,
{
    pub fn new(ctx: &Context) -> Self {
        Pool {
            ctx: ctx.clone(),
            allocator: Arc::new(Mutex::new(LinearPoolAllocator::new::<T>(ctx))),
            _m: PhantomData,
        }
    }

    pub fn allocate<'a>(&'a self) -> Allocator<'a, T> {
        Allocator {
            allocator: self.allocator.lock(),
            current_allocations: 0,
            _m: PhantomData,
        }
    }
}

pub trait CreateLayout {
    fn create_layout(&self, data: &[Binding<DescriptorType>]) -> InnerLayout;
}
pub trait LayoutApi {}

pub struct InnerLayout {
    pub inner: Box<dyn LayoutApi>,
}

pub struct Layout<T: DescriptorInfo> {
    pub inner_layout: InnerLayout,
    _m: PhantomData<T>,
}
impl<T> Layout<T>
where
    T: DescriptorInfo,
{
    pub fn new(ctx: &Context) -> Self {
        Layout {
            inner_layout: ctx.create_layout(&T::layout()),
            _m: PhantomData,
        }
    }
}
pub trait DescriptorApi: Downcast {
    fn write(&mut self, data: &[Binding<DescriptorResource>]);
}
impl_downcast!(DescriptorApi);

pub struct InnerDescriptor {
    pub inner: Box<dyn DescriptorApi>,
}

#[derive(Debug, Copy, Clone)]
pub struct DescriptorSizes {
    pub buffer: u32,
    pub images: u32,
}

pub trait DescriptorInfo
where
    Self: 'static,
{
    fn descriptor_data(&self) -> Vec<Binding<DescriptorResource>>;
    fn sizes() -> DescriptorSizes;
    fn layout() -> Vec<Binding<DescriptorType>>;
}

pub enum DescriptorType {
    Uniform,
}
pub enum DescriptorResource<'a> {
    Uniform(&'a GenericBuffer),
}
pub struct Binding<T> {
    pub binding: u32,
    pub data: T,
}

pub struct Descriptor<'a, T: DescriptorInfo> {
    pub inner_descriptor: InnerDescriptor,
    _m: PhantomData<&'a T>,
}
impl<'a, T> Descriptor<'a, T>
where
    T: DescriptorInfo,
{
    pub fn update(&mut self, t: &'a T) {
        self.inner_descriptor.inner.write(&t.descriptor_data());
    }
}

impl<'a, T> Deref for Descriptor<'a, T>
where
    T: DescriptorInfo,
{
    type Target = DescriptorApi;
    fn deref(&self) -> &Self::Target {
        self.inner_descriptor.inner.as_ref()
    }
}

impl DescriptorApi {
    pub fn downcast<B: BackendApi>(&self) -> &B::Descriptor {
        self.downcast_ref::<B::Descriptor>()
            .expect("Downcast Descriptor Vulkan")
    }
}