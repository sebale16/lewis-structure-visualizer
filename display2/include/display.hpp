#pragma once

#include "config.hpp"
#include "molecule.hpp"

#include <GLFW/glfw3.h>
#include <webgpu/webgpu_cpp.h>

#include <expected>
#include <unordered_map>
#include <span>

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
    // where camera is pointing to
    glm::vec3 target{glm::zero<glm::vec3>()};
    glm::vec3 up{glm::vec3(0.f, 1.f, 0.f)};
    float aspect;
    float fovy;
    float znear;
    float zfar;

    wgpu::Buffer cameraBuffer;

    float phi{glm::pi<float>() / 4.f}; // azimuth
    float theta{glm::pi<float>() / 4.f}; // elevation
    float rho{5.f};

    void Update();

    glm::mat4 BuildViewProjectionMatrix() const;
};

// holds actual data that varies per instance
struct InstanceData {
    glm::mat4 modelMatrix;
};

// holds a single instance buffer
struct Instances {
    wgpu::Buffer instanceBuffer;
    std::vector<InstanceData> instanceData;

    wgpu::Buffer colorBuffer;
    wgpu::BindGroup colorBindGroup;
    glm::vec4 color;

    // returns view of raw instance data
    std::span<InstanceData> GetRawData();
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
    float deltaTime{0.f};
    float lastFrame{0.f};

    wgpu::BindGroupLayout colorBindGroupLayout;

    std::unordered_map<std::string, Mesh> meshes;

    std::unordered_map<std::string, Instances> instances;

    /// processes wasd user input to update `camera`
    void ProcessInput();

    /// helper to create a `Camera`, its buffer, and its bind group
    void CreateCamera();

    /// creates a vector of `Instances` and their buffers based on `molecule`
    void CreateInstances(const std::vector<molecule::BondedAtom>& bondedAtoms);

    /// helper to configure surface
    void ConfigureSurface();

    /// helper to initialize models to be render
    void LoadMeshes(const std::vector<std::string>& filePaths);

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
    std::expected<Mesh, std::string> LoadMeshFromGLTF(const std::string& filePath);
};

} // namespace display
