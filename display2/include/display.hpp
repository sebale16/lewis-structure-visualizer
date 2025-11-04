#pragma once

#include <GLFW/glfw3.h>
#include <webgpu/webgpu_cpp.h>

namespace display {

class Application {
    private:
        GLFWwindow *window;
        wgpu::Instance instance;
        wgpu::Adapter adapter;
        wgpu::Device device;
        wgpu::Queue queue;
        uint32_t gWidth;
        uint32_t gHeight;
        wgpu::Surface surface;
        wgpu::TextureFormat format;

        /// helper to configure surface
        void ConfigureSurface();

        /// returns the next wgpu::TextureView that can be drawn on
        wgpu::TextureView GetNextSurfaceTextureView();

    public:
        /// initializes application and returns success or failure
        bool Initialize(uint32_t width, uint32_t height);

        /// terminate what was initialized
        void Terminate();

        /// render and present a frame
        void RenderPresent();

        /// returns true if app should keep rendering; necessary because window is a private member
        bool KeepRunning();
};

}
