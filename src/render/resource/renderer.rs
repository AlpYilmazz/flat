use bevy::prelude::{Resource, Deref};
use wgpu::util::DeviceExt;

use super::pipeline::BindGroupLayout;


#[derive(Resource, Deref)]
pub struct RenderInstance(pub wgpu::Instance);

#[derive(Resource, Deref)]
pub struct RenderAdapter(pub wgpu::Adapter);

#[derive(Resource, Deref)]
pub struct RenderQueue(pub wgpu::Queue);

#[derive(Resource)]
pub struct RenderDevice(pub wgpu::Device);

impl RenderDevice {
    #[inline]
    pub fn inner(&self) -> &wgpu::Device {
        &self.0
    }

    /// Check for resource cleanups and mapping callbacks.
    ///
    /// Return `true` if the queue is empty, or `false` if there are more queue
    /// submissions still in flight. (Note that, unless access to the [`Queue`] is
    /// coordinated somehow, this information could be out of date by the time
    /// the caller receives it. `Queue`s can be shared between threads, so
    /// other threads could submit new work at any time.)
    ///
    /// On the web, this is a no-op. `Device`s are automatically polled.
    pub fn poll(&self, maintain: wgpu::Maintain) -> bool {
        self.0.poll(maintain)
    }

    /// List all features that may be used with this device.
    ///
    /// Functions may panic if you use unsupported features.
    pub fn features(&self) -> wgpu::Features {
        self.0.features()
    }

    /// List all limits that were requested of this device.
    ///
    /// If any of these limits are exceeded, functions may panic.
    pub fn limits(&self) -> wgpu::Limits {
        self.0.limits()
    }

    /// Creates a shader module from either SPIR-V or WGSL source code.
    pub fn create_shader_module(&self, desc: wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule {
        self.0.create_shader_module(desc)
    }

    /// Creates a shader module from either SPIR-V or WGSL source code without runtime checks.
    ///
    /// # Safety
    /// In contrast with [`create_shader_module`](Self::create_shader_module) this function
    /// creates a shader module without runtime checks which allows shaders to perform
    /// operations which can lead to undefined behavior like indexing out of bounds, thus it's
    /// the caller responsibility to pass a shader which doesn't perform any of this
    /// operations.
    ///
    /// This has no effect on web.
    pub unsafe fn create_shader_module_unchecked(
        &self,
        desc: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::ShaderModule {
        self.0.create_shader_module_unchecked(desc)
    }

    /// Creates a shader module from SPIR-V binary directly.
    ///
    /// # Safety
    ///
    /// This function passes binary data to the backend as-is and can potentially result in a
    /// driver crash or bogus behaviour. No attempt is made to ensure that data is valid SPIR-V.
    ///
    /// See also [`include_spirv_raw!`] and [`util::make_spirv_raw`].
    pub unsafe fn create_shader_module_spirv(
        &self,
        desc: &wgpu::ShaderModuleDescriptorSpirV,
    ) -> wgpu::ShaderModule {
        self.0.create_shader_module_spirv(desc)
    }

    /// Creates an empty [`CommandEncoder`].
    pub fn create_command_encoder(
        &self,
        desc: &wgpu::CommandEncoderDescriptor,
    ) -> wgpu::CommandEncoder {
        self.0.create_command_encoder(desc)
    }

    /// Creates an empty [`RenderBundleEncoder`].
    pub fn create_render_bundle_encoder(
        &self,
        desc: &wgpu::RenderBundleEncoderDescriptor,
    ) -> wgpu::RenderBundleEncoder {
        self.0.create_render_bundle_encoder(desc)
    }

    /// Creates a new [`BindGroup`].
    pub fn create_bind_group(&self, desc: &wgpu::BindGroupDescriptor) -> wgpu::BindGroup {
        self.0.create_bind_group(desc)
    }

    /// Creates a [`BindGroupLayout`].
    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor,
    ) -> BindGroupLayout {
        BindGroupLayout::from(self.0.create_bind_group_layout(desc))
    }

    /// Creates a [`PipelineLayout`].
    pub fn create_pipeline_layout(
        &self,
        desc: &wgpu::PipelineLayoutDescriptor,
    ) -> wgpu::PipelineLayout {
        self.0.create_pipeline_layout(desc)
    }

    /// Creates a [`RenderPipeline`].
    pub fn create_render_pipeline(
        &self,
        desc: &wgpu::RenderPipelineDescriptor,
    ) -> wgpu::RenderPipeline {
        self.0.create_render_pipeline(desc)
    }

    /// Creates a [`ComputePipeline`].
    pub fn create_compute_pipeline(
        &self,
        desc: &wgpu::ComputePipelineDescriptor,
    ) -> wgpu::ComputePipeline {
        self.0.create_compute_pipeline(desc)
    }

    /// Creates a [`Buffer`].
    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> wgpu::Buffer {
        self.0.create_buffer(desc)
    }

    /// Creates a new [`Texture`].
    ///
    /// `desc` specifies the general format of the texture.
    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor) -> wgpu::Texture {
        self.0.create_texture(desc)
    }

    /// Creates a new [`Sampler`].
    ///
    /// `desc` specifies the behavior of the sampler.
    pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> wgpu::Sampler {
        self.0.create_sampler(desc)
    }

    /// Creates a new [`QuerySet`].
    pub fn create_query_set(&self, desc: &wgpu::QuerySetDescriptor) -> wgpu::QuerySet {
        self.0.create_query_set(desc)
    }

    /// Creates a [Buffer](crate::Buffer) with data to initialize it.
    pub fn create_buffer_init(&self, desc: &wgpu::util::BufferInitDescriptor) -> wgpu::Buffer {
        self.0.create_buffer_init(desc)
    }

    /// Upload an entire texture and its mipmaps from a source buffer.
    ///
    /// Expects all mipmaps to be tightly packed in the data buffer.
    ///
    /// If the texture is a 2DArray texture, uploads each layer in order, expecting
    /// each layer and its mips to be tightly packed.
    ///
    /// Example:
    /// Layer0Mip0 Layer0Mip1 Layer0Mip2 ... Layer1Mip0 Layer1Mip1 Layer1Mip2 ...
    ///
    /// Implicitly adds the `COPY_DST` usage if it is not present in the descriptor,
    /// as it is required to be able to upload the data to the gpu.
    pub fn create_texture_with_data(
        &self,
        queue: &RenderQueue,
        desc: &wgpu::TextureDescriptor,
        data: &[u8],
    ) -> wgpu::Texture {
        self.0.create_texture_with_data(&queue, desc, data)
    }

    /// Set a callback for errors that are not handled in error scopes.
    pub fn on_uncaptured_error(&self, handler: impl wgpu::UncapturedErrorHandler) {
        self.0.on_uncaptured_error(handler);
    }

    /// Push an error scope.
    pub fn push_error_scope(&self, filter: wgpu::ErrorFilter) {
        self.0.push_error_scope(filter);
    }

    /// Pop an error scope.
    pub fn pop_error_scope(&self) -> impl std::future::Future<Output = Option<wgpu::Error>> + Send {
        self.0.pop_error_scope()
    }

    /// Starts frame capture.
    pub fn start_capture(&self) {
        self.0.start_capture()
    }

    /// Stops frame capture.
    pub fn stop_capture(&self) {
        self.0.stop_capture()
    }
}