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
        wgpu::TextureFormat textureFormat;
        wgpu::RenderPipeline renderPipeline;

        /// @brief helper to configure surface
        void ConfigureSurface();

        /// @brief returns the next wgpu::TextureView that can be drawn on
        wgpu::TextureView GetNextSurfaceTextureView();

        /// @brief helper to initialize render pipeline
        void CreateRenderPipeline();

    public:
        /// @brief initializes application and returns success or failure
        bool Initialize(uint32_t width, uint32_t height);

        /// @brief render and present a frame
        void RenderPresent();

        /// @brief returns true if app should keep rendering; necessary because window is a private member
        bool KeepRunning();
};

}
