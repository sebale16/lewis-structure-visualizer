#include "display.hpp"

#include <GLFW/glfw3.h>
#include <webgpu/webgpu_glfw.h>
#include <iostream>

void display::Application::ConfigureSurface() {
    wgpu::SurfaceCapabilities capabilities;
    surface.GetCapabilities(adapter, &capabilities);
    format = capabilities.formats[0];
    
    wgpu::SurfaceConfiguration config {
            .device = device,
            .format = format,
            .width = gWidth,
            .height = gHeight,
            .presentMode = wgpu::PresentMode::Fifo,
    };

    surface.Configure(&config);
}

wgpu::TextureView display::Application::GetNextSurfaceTextureView() {
    // get next texture to be presented
    wgpu::SurfaceTexture surfaceTexture;
    surface.GetCurrentTexture(&surfaceTexture);

    // create view for this texture
    wgpu::TextureViewDescriptor textureViewDesc {
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
    wgpu::InstanceDescriptor instance_desc = {};
    instance_desc.nextInChain = nullptr;
    instance_desc.requiredFeatureCount = 1;
    instance_desc.requiredFeatures = &kTimedWaitAny;

    instance = wgpu::CreateInstance(&instance_desc);

    if (instance == nullptr) {
        std::cerr << "Could not initialize WebGPU!" << std::endl;
        return false;
    }

    // querying adapter
    // future to be passed into wgpuInstanceWaitAny (which is wrapped by WaitAny)
    wgpu::Future adapter_future = instance.RequestAdapter(
        nullptr, // adapter options
        wgpu::CallbackMode::WaitAnyOnly, // callbackMode
        // WaitAnyOnly fires when this future is passed into wgpuInstanceWaitAny, RequestAdapter is completed or has already completed
        [this](wgpu::RequestAdapterStatus status, wgpu::Adapter a, wgpu::StringView message) {
            if (status != wgpu::RequestAdapterStatus::Success) {
                std::cout << "RequestAdapter: " << message.data << "\n";
                exit(1);
            }
            adapter = std::move(a);
        }
    );
    instance.WaitAny(adapter_future, UINT64_MAX);

    wgpu::AdapterInfo adapter_info;
    adapter.GetInfo(&adapter_info);

    std::cout << "VendorID: " << std::hex << adapter_info.vendorID << std::dec << "\n";
    std::cout << "Vendor: " << adapter_info.vendor.data << "\n";
    std::cout << "Architecture: " << adapter_info.architecture.data << "\n";
    std::cout << "DeviceID: " << std::hex << adapter_info.deviceID << std::dec << "\n";
    std::cout << "Name: " << adapter_info.device.data << "\n";
    std::cout << "Driver description: " << adapter_info.description.data << "\n";

    // chosing the device
    wgpu::DeviceDescriptor device_desc = {};
    device_desc.SetUncapturedErrorCallback(
        [](const wgpu::Device&, wgpu::ErrorType error_type, wgpu::StringView message) {
            std::cout << "Error: " << static_cast<uint64_t>(error_type) << " - message: " << message.data << "\n";
        }
    );
    device_desc.label = "Device";
    device_desc.SetDeviceLostCallback(
        wgpu::CallbackMode::AllowProcessEvents,
        [](const wgpu::Device&, wgpu::DeviceLostReason reason, wgpu::StringView message) {
            std::cout << "Device lost. Reason: " << static_cast<uint32_t>(reason) << " - message: " << message.data << "\n";
        }
    );

    wgpu::Future device_future = adapter.RequestDevice(
        &device_desc,
        wgpu::CallbackMode::WaitAnyOnly,
        [this](wgpu::RequestDeviceStatus status, wgpu::Device d, wgpu::StringView message) {
            if (status != wgpu::RequestDeviceStatus::Success) {
                std::cout << "RequestDevice: " << message.data << "\n";
                exit(1);
            }
            device = std::move(d);
        }
    );
    instance.WaitAny(device_future, UINT64_MAX);

    queue = device.GetQueue();

    // callback when work is done
    auto onQueueWorkDone = [](wgpu::QueueWorkDoneStatus status, wgpu::StringView message) {
        std::cout << "Queue work finished with status: " << static_cast<uint64_t>(status) << " - message: " << message.data << "\n";
    };
    queue.OnSubmittedWorkDone(wgpu::CallbackMode::AllowProcessEvents, onQueueWorkDone);

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

    return true;
}

void display::Application::RenderPresent() {
    // which textures are the target of the render pass
    wgpu::RenderPassColorAttachment attachment {
            .view = GetNextSurfaceTextureView(),
            .loadOp = wgpu::LoadOp::Clear,
            .storeOp = wgpu::StoreOp::Store,
            .clearValue = wgpu::Color{ 1.0, 0.1, 0.1, 1.0 },
    };

    // command encoder for drawing
    wgpu::CommandEncoderDescriptor commandEncoderDesc {
            .nextInChain = nullptr,
            .label = "Command Encoder",
    };
    wgpu::CommandEncoder commandEncoder = device.CreateCommandEncoder(&commandEncoderDesc);

    // attach texture to the render pass
    wgpu::RenderPassDescriptor renderPassDesc {
            .colorAttachmentCount = 1,
            .colorAttachments = &attachment,
    };

    // record the render pass
    wgpu::RenderPassEncoder renderPass = commandEncoder.BeginRenderPass(&renderPassDesc);
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
