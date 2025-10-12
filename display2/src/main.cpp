#include <GLFW/glfw3.h>
#include <webgpu/webgpu_cpp.h>

#include <iostream>

wgpu::Instance          g_instance;
wgpu::Adapter           g_adapter;
wgpu::Device            g_device;
wgpu::RenderPipeline    g_pipeline;

wgpu::Surface           g_surface;
wgpu::TextureFormat     g_format;

int main() {
    static const auto kTimedWaitAny = wgpu::InstanceFeatureName::TimedWaitAny;

    wgpu::InstanceDescriptor desc = {};
    desc.nextInChain = nullptr;
    desc.requiredFeatureCount = 1;
    desc.requiredFeatures = &kTimedWaitAny;

    g_instance = wgpu::CreateInstance(&desc);

    if (g_instance == nullptr) {
        std::cerr << "Could not initialize WebGPU!" << std::endl;
        return 1;
    }

    // future to be passed into wgpuInstanceWaitAny (which is wrapped by WaitAny)
    wgpu::Future adapter_future = g_instance.RequestAdapter(
        nullptr, // adapter options
        wgpu::CallbackMode::WaitAnyOnly, // callbackMode
        // WaitAnyOnly fires when this future is passed into wgpuInstanceWaitAny, RequestAdapter is completed or has already completed
        [](wgpu::RequestAdapterStatus status, wgpu::Adapter a, wgpu::StringView message) {
            if (status != wgpu::RequestAdapterStatus::Success) {
                std::cout << "RequestAdapter: " << message.data << "\n";
                exit(0);
            }
            g_adapter = std::move(a);
        }
    );
    g_instance.WaitAny(adapter_future, UINT64_MAX);

    wgpu::AdapterInfo adapter_info;
    g_adapter.GetInfo(&adapter_info);

    std::cout << "VendorID: " << std::hex << adapter_info.vendorID << std::dec << "\n";
    std::cout << "Vendor: " << adapter_info.vendor.data << "\n";
    std::cout << "Architecture: " << adapter_info.architecture.data << "\n";
    std::cout << "DeviceID: " << std::hex << adapter_info.deviceID << std::dec << "\n";
    std::cout << "Name: " << adapter_info.device.data << "\n";
    std::cout << "Driver description: " << adapter_info.description.data << "\n";
    return EXIT_SUCCESS;

}
