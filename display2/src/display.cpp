#include "display.hpp"

#include <GLFW/glfw3.h>
#include <webgpu/webgpu_glfw.h>

#include <iostream>

void display::Application::ConfigureSurface() {
    wgpu::SurfaceCapabilities capabilities;
    surface.GetCapabilities(adapter, &capabilities);
    textureFormat = capabilities.formats[0];

    wgpu::SurfaceConfiguration config{
        .device = device,
        .format = textureFormat,
        .width = gWidth,
        .height = gHeight,
        .presentMode = wgpu::PresentMode::Fifo,
    };

    surface.Configure(&config);
}

void display::Application::CreateRenderPipeline() {
    // TODO: remove
    // example shader code (triangle)
    const char* shaderSource = R"(
    @vertex
    fn vs_main(@location(0) in_vertex_position: vec2f) -> @builtin(position) vec4f {
        return vec4f(in_vertex_position, 0.0, 1.0);
    }

    @fragment
    fn fs_main() -> @location(0) vec4f {
        return vec4f(0.0, 0.4, 1.0, 1.0);
    }
    )";

    wgpu::RenderPipelineDescriptor renderPipelineDescriptor;

    /// describe vertex pipeline state
    // load shader module for vertex pipeline state
    wgpu::ShaderSourceWGSL shaderSourceWgsl{
        wgpu::ShaderSourceWGSL::Init{.code = shaderSource}};
    wgpu::ShaderModuleDescriptor shaderModuleDescriptor{
        .nextInChain = &shaderSourceWgsl,
        .label = "Shader Source",
    };
    wgpu::ShaderModule shaderModule =
        device.CreateShaderModule(&shaderModuleDescriptor);
    wgpu::VertexState vertexState{
        .module = shaderModule,
        .entryPoint = "vs_main",
    };

    renderPipelineDescriptor.vertex = vertexState;

    /// describe primitive pipeline state
    wgpu::PrimitiveState primitiveState{
        .topology = wgpu::PrimitiveTopology::TriangleList,
        .frontFace = wgpu::FrontFace::CCW,
        .cullMode = wgpu::CullMode::None,
    };

    renderPipelineDescriptor.primitive = primitiveState;

    /// describe fragment pipeline state
    wgpu::BlendState blendState{
        .color =
            wgpu::BlendComponent{
                .operation = wgpu::BlendOperation::Add,
                .srcFactor = wgpu::BlendFactor::SrcAlpha,
                .dstFactor = wgpu::BlendFactor::OneMinusSrcAlpha,
            },
        .alpha = wgpu::BlendComponent{
            .operation = wgpu::BlendOperation::Add,
            .srcFactor = wgpu::BlendFactor::Zero,
            .dstFactor = wgpu::BlendFactor::One,
        }};
    wgpu::ColorTargetState colorTargetState{
        .format = textureFormat,
        .blend = &blendState,
        .writeMask = wgpu::ColorWriteMask::All,
    };
    wgpu::FragmentState fragmentState{
        .module = shaderModule,
        .entryPoint = "fs_main",
        .targetCount = 1, // only one target because render pass has only one
                          // output color attachment
        .targets = &colorTargetState,
    };

    renderPipelineDescriptor.fragment = &fragmentState;

    /// describe stencil/depth fragment state
    wgpu::DepthStencilState depthStencilState{
        .format = wgpu::TextureFormat::Depth24PlusStencil8,
        .depthWriteEnabled = wgpu::OptionalBool::True,
        .depthCompare = wgpu::CompareFunction::Less, // fragment is blended only if depth is less than current
        .stencilReadMask = 0,
        .stencilWriteMask = 0,
    };
    renderPipelineDescriptor.depthStencil = &depthStencilState;

    /// describe multi-sampling state
    wgpu::MultisampleState multisampleState;
    renderPipelineDescriptor.multisample = multisampleState;

    /// describe pipeline layout
    renderPipelineDescriptor.layout = nullptr;

    /// create render pipeline
    renderPipeline = device.CreateRenderPipeline(&renderPipelineDescriptor);

    /// create depth texture
    wgpu::TextureDescriptor depthTextureDesc{
        .label = "Depth Texture Descriptor",
        .usage = wgpu::TextureUsage::RenderAttachment,
        .dimension = wgpu::TextureDimension::e2D,
        .size = {WIDTH, HEIGHT, 0},
        .mipLevelCount = 1,
        .sampleCount = 1,
        .viewFormatCount = 1,
        .viewFormats = &depthStencilState.format,
    };
    wgpu::Texture depthTexture = device.CreateTexture(&depthTextureDesc);

    /// create depth texture view
    wgpu::TextureViewDescriptor depthTextureViewDesc{
        .label = "Depth Texture View",
        .format = depthStencilState.format,
        .dimension = wgpu::TextureViewDimension::e2D,
        .baseMipLevel = 0,
        .mipLevelCount = 1,
        .baseArrayLayer = 0,
        .arrayLayerCount = 1,
        .aspect = wgpu::TextureAspect::DepthOnly,
        .usage = wgpu::TextureUsage::RenderAttachment,
    };
    wgpu::TextureView depthTextureView = depthTexture.CreateView(&depthTextureViewDesc);


}

wgpu::TextureView display::Application::GetNextSurfaceTextureView() {
    // get next texture to be presented
    wgpu::SurfaceTexture surfaceTexture;
    surface.GetCurrentTexture(&surfaceTexture);

    // create view for this texture
    wgpu::TextureViewDescriptor textureViewDesc{
        .nextInChain = nullptr,
        .label = "Texture View",
        .format = surfaceTexture.texture.GetFormat(),
        .dimension = wgpu::TextureViewDimension::e2D,
        .baseMipLevel = 0,
        .mipLevelCount = 1,
        .baseArrayLayer = 0,
        .arrayLayerCount = 1,
        .aspect = wgpu::TextureAspect::All,
    };
    wgpu::TextureView targetView = surfaceTexture.texture.CreateView(&textureViewDesc);

    return targetView;
}

bool display::Application::Initialize(uint32_t width, uint32_t height) {
    static const auto kTimedWaitAny = wgpu::InstanceFeatureName::TimedWaitAny;
    wgpu::InstanceDescriptor instance_desc = {
        .nextInChain = nullptr,
        .requiredFeatureCount = 1,
        .requiredFeatures = &kTimedWaitAny,
    };

    instance = wgpu::CreateInstance(&instance_desc);

    if (instance == nullptr) {
        std::cerr << "Could not initialize WebGPU!" << std::endl;
        return false;
    }

    // querying adapter
    // future to be passed into wgpuInstanceWaitAny (which is wrapped by
    // WaitAny)
    wgpu::Future adapter_future = instance.RequestAdapter(
        nullptr,                         // adapter options
        wgpu::CallbackMode::WaitAnyOnly, // callbackMode
        // WaitAnyOnly fires when this future is passed into
        // wgpuInstanceWaitAny, RequestAdapter is completed or has already
        // completed
        [this](wgpu::RequestAdapterStatus status, wgpu::Adapter a,
               wgpu::StringView message) {
            if (status != wgpu::RequestAdapterStatus::Success) {
                std::cout << "RequestAdapter: " << message.data << "\n";
                exit(1);
            }
            adapter = std::move(a);
        });
    instance.WaitAny(adapter_future, UINT64_MAX);

    wgpu::AdapterInfo adapter_info;
    adapter.GetInfo(&adapter_info);

    std::cout << "VendorID: " << std::hex << adapter_info.vendorID << std::dec
              << "\n";
    std::cout << "Vendor: " << adapter_info.vendor.data << "\n";
    std::cout << "Architecture: " << adapter_info.architecture.data << "\n";
    std::cout << "DeviceID: " << std::hex << adapter_info.deviceID << std::dec
              << "\n";
    std::cout << "Name: " << adapter_info.device.data << "\n";
    std::cout << "Driver description: " << adapter_info.description.data
              << "\n";

    wgpu::Limits limits;
    adapter.GetLimits(&limits);
    std::cout << "adapter.maxVertexAttributes: " << limits.maxVertexAttributes << "\n";

    // choosing the device
    wgpu::DeviceDescriptor device_desc = {};
    device_desc.SetUncapturedErrorCallback([](const wgpu::Device &,
                                              wgpu::ErrorType error_type,
                                              wgpu::StringView message) {
        std::cout << "Error: " << static_cast<uint64_t>(error_type)
                  << " - message: " << message.data << "\n";
    });
    device_desc.label = "Device";
    device_desc.SetDeviceLostCallback(
        wgpu::CallbackMode::AllowProcessEvents,
        [](const wgpu::Device&, wgpu::DeviceLostReason reason,
           wgpu::StringView message) {
                std::cout << "Device lost. Reason: "
                      << static_cast<uint32_t>(reason)
                      << " - message: " << message.data << "\n";
        });

    wgpu::Future device_future = adapter.RequestDevice(
        &device_desc, wgpu::CallbackMode::WaitAnyOnly,
        [this](wgpu::RequestDeviceStatus status, wgpu::Device d,
               wgpu::StringView message) {
            if (status != wgpu::RequestDeviceStatus::Success) {
                std::cout << "RequestDevice: " << message.data << "\n";
                exit(1);
            }
            device = std::move(d);
        });
    instance.WaitAny(device_future, UINT64_MAX);

    device.GetLimits(&limits);
    std::cout << "device.maxVertexAttributes: " << limits.maxVertexAttributes << "\n";

    queue = device.GetQueue();

    // callback when work is done
    auto onQueueWorkDone = [](wgpu::QueueWorkDoneStatus status,
                              wgpu::StringView message) {
        std::cout << "Queue work finished with status: "
                  << static_cast<uint64_t>(status)
                  << " - message: " << message.data << "\n";
    };
    queue.OnSubmittedWorkDone(wgpu::CallbackMode::AllowProcessEvents,
                              onQueueWorkDone);

    // set up window + surface
    if (!glfwInit()) {
        return false;
    }

    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    window = glfwCreateWindow(width, height, "Display", nullptr, nullptr);
    surface = wgpu::glfw::CreateSurfaceForWindow(instance, window);
    gWidth = width;
    gHeight = height;
    ConfigureSurface();

    CreateRenderPipeline();

    return true;
}

void display::Application::RenderPresent() {
    // which textures are the target of the render pass
    wgpu::RenderPassColorAttachment attachment{
        .view = GetNextSurfaceTextureView(),
        .loadOp = wgpu::LoadOp::Clear,
        .storeOp = wgpu::StoreOp::Store,
        .clearValue = wgpu::Color{1.0, 0.7, 0.3, 1.0},
    };

    // command encoder for drawing
    wgpu::CommandEncoderDescriptor commandEncoderDesc{
        .nextInChain = nullptr,
        .label = "Command Encoder",
    };
    wgpu::CommandEncoder commandEncoder =
        device.CreateCommandEncoder(&commandEncoderDesc);

    // attach texture to the render pass
    wgpu::RenderPassDescriptor renderPassDesc{
        .colorAttachmentCount = 1,
        .colorAttachments = &attachment,
    };

    // record the render pass
    wgpu::RenderPassEncoder renderPass =
        commandEncoder.BeginRenderPass(&renderPassDesc);
    renderPass.SetPipeline(renderPipeline);
    renderPass.Draw(3, 1, 0, 0);
    renderPass.End();

    // finish recording
    wgpu::CommandBufferDescriptor commandBufferDesc = {
        .nextInChain = nullptr,
        .label = "Command Buffer",
    };
    wgpu::CommandBuffer commands = commandEncoder.Finish(&commandBufferDesc);

    // submit to the queue
    queue.Submit(1, &commands);

    // present only if not on web since web already handles present
#ifndef __EMSCRIPTEN__
    surface.Present();
#endif

    // poll AllowProcessEvents callbacks
    instance.ProcessEvents();
}

bool display::Application::KeepRunning() {
    return !glfwWindowShouldClose(window);
}
