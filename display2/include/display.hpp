#pragma once

#include "config.hpp"

#include <GLFW/glfw3.h>
#include <webgpu/webgpu_cpp.h>

#include <expected>
#include <map>

#ifndef WIDTH
#define WIDTH 1920
#endif

#ifndef HEIGHT
#define HEIGHT 1080
#endif

namespace display {

struct Mesh {
    wgpu::Buffer vertexBuffer;
    wgpu::Buffer indexBuffer;
    uint32_t indexCount;
    wgpu::IndexFormat indexFormat;
};

struct Camera {
    // location of camera
    glm::vec3 eye;
    // where camera is point to
    glm::vec3 target;
    glm::vec3 up;
    float aspect;
    float fovy;
    float znear;
    float zfar;

    wgpu::Buffer cameraBuffer;

    glm::mat4 BuildViewProjectionMatrix() const;
};

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
    wgpu::Texture depthTexture;
    wgpu::RenderPipeline renderPipeline;

    Camera camera;
    wgpu::BindGroupLayout cameraBindGroupLayout;
    wgpu::BindGroup cameraBindGroup;

    std::map<std::string, Mesh> meshes;

    /// helper to create a `Camera` and its buffer
    void CreateCamera();

    /// helper to configure surface
    void ConfigureSurface();

    /// helper to initialize models to be render
    void LoadMeshes(std::vector<std::string>& filePaths);

    /// loads a shader into a shader module
    wgpu::ShaderModule LoadShaderModule(const std::string& filePath);

    /// helper to initialize render pipeline
    void CreateRenderPipeline();

    /// returns the next wgpu::TextureView that can be drawn on
    wgpu::TextureView GetNextSurfaceTextureView();

public:
    /// initializes application and returns success or failure
    bool Initialize(uint32_t width, uint32_t height);

    /// render and present a frame
    void RenderPresent();

    /// returns true if app should keep rendering; necessary because window is a private member
    bool KeepRunning();

    /// constructs a mesh with a point and index buffer from a gltf file; takes first mesh of loaded model
    std::expected<Mesh, std::string> LoadMeshFromGLTF(std::string& filePath);
};

} // namespace display
