#include "display.hpp"

#include <GLFW/glfw3.h>
#include <webgpu/webgpu_glfw.h>
#define GLM_ENABLE_EXPERIMENTAL
#include <glm/gtx/quaternion.hpp>
#define TINYGLTF_IMPLEMENTATION
#define STB_IMAGE_IMPLEMENTATION
#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "tiny_gltf.h"

#include <print>
#include <iostream>
#include <filesystem>

void display::Camera::Update() {
    // theta between 0 to pi
    theta = glm::clamp(theta, 0.1f, glm::pi<float>() - 0.1f);
    
    eye.x = target.x + rho * glm::sin(theta) * glm::cos(phi);
    eye.z = target.z + rho * glm::sin(theta) * glm::sin(phi);
    eye.y = target.y + rho * glm::cos(theta);
}

glm::mat4 display::Camera::BuildViewProjectionMatrix() const {
    glm::mat4 view = glm::lookAt(eye, target, up);
    glm::mat4 proj = glm::perspective(fovy, aspect, znear, zfar);
    return proj * view;
}

std::span<display::InstanceData> display::Instances::GetRawData() {
    return instanceData; // automatic conversion from vector to span
}

void display::Application::ProcessInput() {
    float currentFrame = static_cast<float>(glfwGetTime());
    deltaTime = currentFrame - lastFrame;
    lastFrame = currentFrame;

    float rotationSpeed{2.f};
    float zoomSpeed{5.f};

    // process keys
    if (glfwGetKey(window, GLFW_KEY_W) == GLFW_PRESS)
        camera.rho -= zoomSpeed * deltaTime; // zoom in
    if (glfwGetKey(window, GLFW_KEY_S) == GLFW_PRESS)
        camera.rho += zoomSpeed * deltaTime; // zoom out
    if (glfwGetKey(window, GLFW_KEY_A) == GLFW_PRESS)
        camera.phi -= rotationSpeed * deltaTime; // rotate left
    if (glfwGetKey(window, GLFW_KEY_D) == GLFW_PRESS)
        camera.phi += rotationSpeed * deltaTime; // rotate right

    // constraints
    if (camera.rho < 0.5f) camera.rho = 0.5f;
    if (camera.rho > 50.0f) camera.rho = 50.0f;
}

void display::Application::CreateCamera() {
    camera = Camera{
        .aspect = static_cast<float>(WIDTH) / static_cast<float>(HEIGHT),
        .fovy = glm::radians(70.f),
        .znear = 0.1,
        .zfar = 100,
    };

    // create camera buffer
    wgpu::BufferDescriptor cameraBufferDesc{
        .label = "Camera Uniform Buffer",
        .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Uniform,
        .size = sizeof(glm::mat4),
    };
    camera.cameraBuffer = this->device.CreateBuffer(&cameraBufferDesc);

    /// create bind group for camera so that data in camera buffer can be read as uniform in shader
    wgpu::BindGroupLayoutEntry cameraBindGroupLayoutEntry{
        .binding = 0,
        .visibility = wgpu::ShaderStage::Vertex,
        .buffer = {
            .type = wgpu::BufferBindingType::Uniform,
            .hasDynamicOffset = false,
            .minBindingSize = sizeof(glm::mat4),
        }
    };

    wgpu::BindGroupLayoutDescriptor cameraBindGroupLayoutDesc{
        .label = "Camera Bind Group Layout",
        .entryCount = 1,
        .entries = &cameraBindGroupLayoutEntry,
    };
    cameraBindGroupLayout = device.CreateBindGroupLayout(&cameraBindGroupLayoutDesc);

    wgpu::BindGroupEntry cameraBindGroupEntry{
        .binding = 0,
        .buffer = camera.cameraBuffer,
        .offset = 0,
        .size = sizeof(glm::mat4),
    };

    wgpu::BindGroupDescriptor cameraBindGroupDesc{
        .label = "Camera Bind Group",
        .layout = cameraBindGroupLayout,
        .entryCount = 1,
        .entries = &cameraBindGroupEntry,
    };
    cameraBindGroup = device.CreateBindGroup(&cameraBindGroupDesc);
}

void display::Application::CreateInstances(const std::vector<molecule::BondedAtom>& bondedAtoms) {
    std::print("Creating instances for...");
    for (auto& a : bondedAtoms) {
        std::print("{} ", a.wPtrAtom.lock()->name);
    }
    std::println();
    // create an instance data vector for each type of orbital: s, sp, p
    std::vector<InstanceData> sInstances;
    std::vector<InstanceData> spInstances;
    std::vector<InstanceData> pInstances;
    // build translation + rotation matrix for each atom that will be applied to its set of orbitals
    // translation + rotation are from locs + rots
    for (const auto& atom : bondedAtoms) {
        // model consists of each atom with its orbitals
        auto bondQuatPairs = atom.ToMatrix();
        // apply transforms to each of the atom's orbitals
        for (auto& bQPair : bondQuatPairs) {
            glm::mat4 model = glm::mat4(1.0f);

            // translate: move the whole thing to the atom's world position
            model = glm::translate(model, atom.loc);

            // rotate: apply the atom's overall rotation, then the specific orbital's rotation
            model = model * glm::toMat4(atom.rot);
            model = model * glm::toMat4(bQPair.second);

            // scale: scale the mesh at its local origin
            float scaleFactor = 1.0f;
            if (bQPair.first == molecule::OrbitalType::s) scaleFactor = S_ORBITAL_SCALE;
            else if (bQPair.first == molecule::OrbitalType::sp) scaleFactor = SP_ORBITAL_SCALE;
            else if (bQPair.first == molecule::OrbitalType::p) scaleFactor = P_ORBITAL_SCALE;

            model = glm::scale(model, glm::vec3(scaleFactor));

            // depending on the orbital type, add to corresponding vector
            switch (bQPair.first) {
                case molecule::OrbitalType::s:
                    sInstances.push_back(InstanceData { .modelMatrix = model });
                    break;
                case molecule::OrbitalType::sp:
                    spInstances.push_back(InstanceData { .modelMatrix = model });
                    break;
                case molecule::OrbitalType::p:
                    pInstances.push_back(InstanceData { .modelMatrix = model });
                    break;
            }
        }
    }

    // only create buffers and bind groups for which exist orbital instances
    if (sInstances.size() > 0) {
        wgpu::BufferDescriptor sOrbitalVertexBufferDesc = {
            .label = "s Orbital Vertex Buffer",
            .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Vertex,
            .size = sizeof(InstanceData) * sInstances.size(),
        };
        wgpu::BufferDescriptor sOrbitalColorBufferDesc = {
            .label = "s Orbital Color Buffer",
            .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Uniform,
            .size = sizeof(glm::vec4),
        };
        instances["s"] = Instances{
                .instanceBuffer = this->device.CreateBuffer(&sOrbitalVertexBufferDesc),
                .instanceData = sInstances,
                .colorBuffer = this->device.CreateBuffer(&sOrbitalColorBufferDesc),
                .color = glm::vec4(1.f, 0.f, 1.f, 1.f),
        };

        wgpu::BindGroupEntry colorBindGroupEntry{
            .binding = 0,
            .buffer = instances["s"].colorBuffer,
            .offset = 0,
            .size = sizeof(glm::vec4),
        };

        wgpu::BindGroupDescriptor colorBindGroupDesc{
            .label = "s Orbital Color Bind Group",
            .layout = colorBindGroupLayout,
            .entryCount = 1,
            .entries = &colorBindGroupEntry,
        };
        instances["s"].colorBindGroup = device.CreateBindGroup(&colorBindGroupDesc);
    }
    if (spInstances.size() > 0) {
        wgpu::BufferDescriptor spOrbitalVertexBufferDesc = {
            .label = "sp Orbital Instance Vertex Buffer",
            .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Vertex,
            .size = sizeof(InstanceData) * spInstances.size(),
        };
        wgpu::BufferDescriptor spOrbitalColorBufferDesc = {
            .label = "sp Orbital Color Buffer",
            .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Uniform,
            .size = sizeof(glm::vec4),
        };
        instances["sp"] = Instances {
                .instanceBuffer = this->device.CreateBuffer(&spOrbitalVertexBufferDesc),
                .instanceData = spInstances,
                .colorBuffer = this->device.CreateBuffer(&spOrbitalColorBufferDesc),
                .color = glm::vec4(1.f, 0.f, 0.f, 1.f),
        };
        wgpu::BindGroupEntry colorBindGroupEntry{
            .binding = 0,
            .buffer = instances["sp"].colorBuffer,
            .offset = 0,
            .size = sizeof(glm::vec4),
        };

        wgpu::BindGroupDescriptor colorBindGroupDesc{
            .label = "sp Orbital Color Bind Group",
            .layout = colorBindGroupLayout,
            .entryCount = 1,
            .entries = &colorBindGroupEntry,
        };
        instances["sp"].colorBindGroup = device.CreateBindGroup(&colorBindGroupDesc);
    }
    if (pInstances.size() > 0) {
        wgpu::BufferDescriptor pOrbitalVertexBufferDesc = {
            .label = "p Orbital Vertex Buffer",
            .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Vertex,
            .size = sizeof(InstanceData) * pInstances.size(),
        };
        wgpu::BufferDescriptor pOrbitalColorBufferDesc = {
            .label = "p Orbital Color Buffer",
            .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Uniform,
            .size = sizeof(glm::vec4),
        };
        instances["p"] = Instances{
                .instanceBuffer = this->device.CreateBuffer(&pOrbitalVertexBufferDesc),
                .instanceData = pInstances,
                .colorBuffer = this->device.CreateBuffer(&pOrbitalColorBufferDesc),
                .color = glm::vec4(0.f, 0.f, 1.f, 1.f),
        };
        wgpu::BindGroupEntry colorBindGroupEntry{
            .binding = 0,
            .buffer = instances["p"].colorBuffer,
            .offset = 0,
            .size = sizeof(glm::vec4),
        };

        wgpu::BindGroupDescriptor colorBindGroupDesc{
            .label = "p Orbital Color Bind Group",
            .layout = colorBindGroupLayout,
            .entryCount = 1,
            .entries = &colorBindGroupEntry,
        };
        instances["p"].colorBindGroup = device.CreateBindGroup(&colorBindGroupDesc);
    }

    std::println("s instance count: {}", sInstances.size());
    std::println("sp instance count: {}", spInstances.size());
    std::println("p instance count: {}", pInstances.size());
}

std::expected<display::Mesh, std::string> display::Application::LoadMeshFromGLTF(const std::string& filePath) {
    tinygltf::Model model;
    tinygltf::TinyGLTF loader;
    std::string err;
    std::string warn;

    if (!loader.LoadASCIIFromFile(&model, &err, &warn, filePath)) {
        return std::unexpected(std::format("Failed to parse glTF: {}", filePath));
    }

    if (!warn.empty()) {
        std::println("Warn: {}", warn.c_str());
    }

    if (!err.empty()) {
        return std::unexpected(std::format("Err: {}", err.c_str()));
    }

    const auto& mesh = model.meshes[0];
    const auto& primitive = mesh.primitives[0];
    display::Mesh outputMesh;

    /// vertices
    // go to accessor at index POSITION
    const auto& posAccessor = model.accessors.at(primitive.attributes.at("POSITION"));
    // which bufferView to look at based on what index the accessor at index "POSITION" shows
    const auto& posView = model.bufferViews.at(posAccessor.bufferView);
    const auto& posBuffer = model.buffers.at(posView.buffer);
    
    // calculate size to create buffer
    size_t vertexDataSize = posAccessor.count * sizeof(float) * 3;
    // pointer to actual data
    const void* vertexDataPtr = &posBuffer.data[posView.byteOffset + posAccessor.byteOffset];
    // check that pointer contains data
    if (vertexDataPtr == nullptr) {
        return std::unexpected(std::format("{} has invalid vertex data", filePath));
    }

    // create vertex buffer
    wgpu::BufferDescriptor vertexBufferDesc{
        .label = std::format("{} Vertex Buffer Descriptor", filePath).c_str(),
        .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Vertex,
        .size = vertexDataSize,
    };
    outputMesh.vertexBuffer = this->device.CreateBuffer(&vertexBufferDesc);
    this->queue.WriteBuffer(outputMesh.vertexBuffer, 0, vertexDataPtr, vertexDataSize);

    /// indices
    // go to accessor at "indices"
    const auto& indAccessor = model.accessors.at(primitive.indices);
    const auto& indView = model.bufferViews.at(indAccessor.bufferView);
    const auto& indBuffer = model.buffers.at(indView.buffer);

    outputMesh.indexCount = static_cast<uint32_t>(indAccessor.count);
    size_t indexDataSize = indAccessor.count * tinygltf::GetComponentSizeInBytes(indAccessor.componentType);
    const void* indexDataPtr = &indBuffer.data[indView.byteOffset + indAccessor.byteOffset];
    // check that pointer contains data
    if (indexDataPtr == nullptr) {
        return std::unexpected(std::format("{} has invalid index data", filePath));
    }

    // map gltf type to webgpu index format
    if (indAccessor.componentType == TINYGLTF_COMPONENT_TYPE_UNSIGNED_SHORT) {
        outputMesh.indexFormat = wgpu::IndexFormat::Uint16;
    } else {
        outputMesh.indexFormat = wgpu::IndexFormat::Uint32;
    }

    // create index buffer
    wgpu::BufferDescriptor indexBufferDesc{
        .label = std::format("{} Index Buffer Descriptor", filePath).c_str(),
        .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Index,
        // ensure 4-byte alignment: round up next multiple of 4 and then mask last two bits to make a multiple of 4
        .size = (indexDataSize + 3) & ~3,
    };
    outputMesh.indexBuffer = this->device.CreateBuffer(&indexBufferDesc);
    this->queue.WriteBuffer(outputMesh.indexBuffer, 0, indexDataPtr, indexDataSize);

    return outputMesh;
}

void display::Application::LoadMeshes(const std::vector<std::string>& filePaths) {
    // assuming each file path is one mesh
    for (const auto& fp : filePaths) {
        std::string fileName = std::filesystem::path(fp).stem().string();
        // check if already loaded
        if (meshes.contains(fileName)) {
            continue;
        }
        meshes[fileName] = LoadMeshFromGLTF(fp).value();
        std::println("Loaded {}...", fp);
    }
}

wgpu::ShaderModule display::Application::LoadShaderModule(const std::string& filePath) {
    std::ifstream file(filePath);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open shader file: " + filePath);
    }
    std::stringstream buffer;
    buffer << file.rdbuf();
    std::string shaderSource = buffer.str();

    // load shader module for vertex pipeline state
    wgpu::ShaderSourceWGSL shaderSourceWgsl{
        wgpu::ShaderSourceWGSL::Init{
            .code = shaderSource.c_str()
        }
    };
    wgpu::ShaderModuleDescriptor shaderModuleDescriptor{
        .nextInChain = &shaderSourceWgsl,
        .label = "Shader Source",
    };
    return device.CreateShaderModule(&shaderModuleDescriptor);
}

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
    wgpu::RenderPipelineDescriptor renderPipelineDescriptor;
    auto shaderModule = LoadShaderModule("res/shaders/shader.wgsl");

    /// describe vertex buffer layouts
    // define position attribute
    wgpu::VertexAttribute posAttribute{
        .format = wgpu::VertexFormat::Float32x3,
        .offset = 0,
        .shaderLocation = 0,
    };
    std::vector<wgpu::VertexAttribute> vecPosAttribute = { posAttribute };
    wgpu::VertexBufferLayout posBufferLayout{
        .stepMode = wgpu::VertexStepMode::Vertex,
        .arrayStride = sizeof(float) * 3, // total size of one vertex
        .attributeCount = vecPosAttribute.size(),
        .attributes = vecPosAttribute.data(),
    };

    // define instance attributes
    std::vector<wgpu::VertexAttribute> instanceAttributes(4);
    for (size_t i{0}; i < 4; i++) {
        instanceAttributes[i].format = wgpu::VertexFormat::Float32x4;
        instanceAttributes[i].offset = i * sizeof(glm::vec4);
        instanceAttributes[i].shaderLocation = i + 5;
    }
    wgpu::VertexBufferLayout instanceBufferLayout{
        .stepMode = wgpu::VertexStepMode::Instance,
        .arrayStride = sizeof(glm::mat4),
        .attributeCount = instanceAttributes.size(),
        .attributes = instanceAttributes.data(),
    };

    /// describe vertex pipeline state
    std::vector<wgpu::VertexBufferLayout> vertexBufferLayouts = { posBufferLayout, instanceBufferLayout };
    wgpu::VertexState vertexState{
        .module = shaderModule,
        .entryPoint = "vs_main",
        .bufferCount = 2,
        .buffers = vertexBufferLayouts.data(),
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
    std::vector<wgpu::BindGroupLayout> bindGroupLayouts = {
        cameraBindGroupLayout,
        colorBindGroupLayout,
    };
    wgpu::PipelineLayoutDescriptor pipelineLayoutDesc{
        .label = "Pipeline Layout Descriptor",
        .bindGroupLayoutCount = 2,
        .bindGroupLayouts = bindGroupLayouts.data(),
    };
    renderPipelineDescriptor.layout = device.CreatePipelineLayout(&pipelineLayoutDesc);

    /// create render pipeline
    renderPipeline = device.CreateRenderPipeline(&renderPipelineDescriptor);

    /// create depth texture
    wgpu::TextureDescriptor depthTextureDesc{
        .label = "Depth Texture Descriptor",
        .usage = wgpu::TextureUsage::RenderAttachment,
        .dimension = wgpu::TextureDimension::e2D,
        .size = {WIDTH, HEIGHT, 1},
        .format = wgpu::TextureFormat::Depth24PlusStencil8,
        .mipLevelCount = 1,
        .sampleCount = 1,
        .viewFormatCount = 1,
        .viewFormats = &depthStencilState.format,
    };
    depthTexture = device.CreateTexture(&depthTextureDesc);
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

bool display::Application::Initialize(uint32_t width, uint32_t height, std::string moleculePath) {
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

    std::vector<std::string> resPaths = {
        "res/orbitals/s.gltf",
        "res/orbitals/sp.gltf",
        "res/orbitals/p.gltf",
    };
    LoadMeshes(resPaths);

    CreateCamera();

    /// create bind group for color so that it can be read as uniform in shader; one per app
    wgpu::BindGroupLayoutEntry colorBindGroupLayoutEntry{
        .binding = 0,
        .visibility = wgpu::ShaderStage::Vertex,
        .buffer = {
            .type = wgpu::BufferBindingType::Uniform,
            .hasDynamicOffset = false,
            .minBindingSize = sizeof(glm::vec4),
        }
    };
    wgpu::BindGroupLayoutDescriptor colorBindGroupLayoutDesc{
        .label = "Color Bind Group Layout",
        .entryCount = 1,
        .entries = &colorBindGroupLayoutEntry,
    };
    colorBindGroupLayout = device.CreateBindGroupLayout(&colorBindGroupLayoutDesc);

    // create instances from solved molecule
    std::string jsonFilePath = "/home/seb/projects/lewis-structure-visualizer/solver/out/" + moleculePath;
    std::string csvFilePath = "/home/seb/projects/lewis-structure-visualizer/data/data.csv";
    molecule::Molecule molecule(jsonFilePath, csvFilePath);
    auto bondedAtoms = molecule.ComputeAtomLocsRots().value();

    CreateInstances(bondedAtoms);

    CreateRenderPipeline();

    return true;
}

void display::Application::RenderPresent() {
    // which textures are the target of the render pass
    wgpu::RenderPassColorAttachment attachment{
        .view = GetNextSurfaceTextureView(),
        .loadOp = wgpu::LoadOp::Clear,
        .storeOp = wgpu::StoreOp::Store,
        .clearValue = wgpu::Color{0.0, 0.0, 0.0, 1.0},
    };

    // command encoder for drawing
    wgpu::CommandEncoderDescriptor commandEncoderDesc{
        .nextInChain = nullptr,
        .label = "Command Encoder",
    };
    wgpu::CommandEncoder commandEncoder = device.CreateCommandEncoder(&commandEncoderDesc);

    /// create depth texture view
    wgpu::TextureViewDescriptor depthTextureViewDesc{
        .label = "Depth Texture View",
        .format = wgpu::TextureFormat::Depth24PlusStencil8,
        .dimension = wgpu::TextureViewDimension::e2D,
        .baseMipLevel = 0,
        .mipLevelCount = 1,
        .baseArrayLayer = 0,
        .arrayLayerCount = 1,
        .aspect = wgpu::TextureAspect::All,
        .usage = wgpu::TextureUsage::RenderAttachment,
    };
    wgpu::TextureView depthTextureView = depthTexture.CreateView(&depthTextureViewDesc);

    wgpu::RenderPassDepthStencilAttachment depthStencilAttachment{
        .view = depthTextureView,
        .depthLoadOp = wgpu::LoadOp::Clear,
        .depthStoreOp = wgpu::StoreOp::Store,
        // initial value of depth buffer, signifying the far value in this case
        .depthClearValue = 1.0f,
        .depthReadOnly = false,
        .stencilLoadOp = wgpu::LoadOp::Clear,
        .stencilStoreOp = wgpu::StoreOp::Store,
        .stencilClearValue = 0,
        .stencilReadOnly = false,
    };

    // attach textures to the render pass
    wgpu::RenderPassDescriptor renderPassDesc{
        .colorAttachmentCount = 1,
        .colorAttachments = &attachment,
        .depthStencilAttachment = &depthStencilAttachment,
    };

    // process user input
    ProcessInput();
    
    // update and write camera data to buffer
    camera.Update();
    glm::mat4 viewProjMat = camera.BuildViewProjectionMatrix();
    queue.WriteBuffer(camera.cameraBuffer, 0, &viewProjMat, sizeof(viewProjMat));

    // update and write instance data to buffer for each instances
    for (auto& [_, instance] : instances) {
        // vertex
        queue.WriteBuffer(instance.instanceBuffer, 0, instance.GetRawData().data(), instance.GetRawData().size_bytes());
        // color
        queue.WriteBuffer(instance.colorBuffer, 0, &instance.color, sizeof(instance.color));
    }

    // record the render pass
    wgpu::RenderPassEncoder renderPass = commandEncoder.BeginRenderPass(&renderPassDesc);
    renderPass.SetPipeline(renderPipeline);

    // bind camera bind group
    renderPass.SetBindGroup(0, cameraBindGroup);

    // draw meshes
    for (auto& [orbitalType, instance] : instances) {
        // vertex
        renderPass.SetVertexBuffer(0, meshes.at(orbitalType).vertexBuffer);
        // index
        renderPass.SetIndexBuffer(meshes.at(orbitalType).indexBuffer, meshes.at(orbitalType).indexFormat);
        // instance
        renderPass.SetVertexBuffer(1, instance.instanceBuffer);
        // color
        renderPass.SetBindGroup(1, instance.colorBindGroup);

        renderPass.DrawIndexed(meshes.at(orbitalType).indexCount, instance.instanceData.size());
    }

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
