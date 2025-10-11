#include <GLFW/glfw3.h>
#include <webgpu/webgpu_cpp.h>

#include <iostream>

int main() {
    wgpu::InstanceDescriptor desc = {};
    desc.nextInChain = nullptr;

    wgpu::Instance instance = wgpu::CreateInstance(&desc);

    if (instance == nullptr) {
        std::cerr << "Could not initialize WebGPU!" << std::endl;
        return 1;
    }

    std::cout << "Instance: " << instance.Get() << std::endl;

    return 0;
}