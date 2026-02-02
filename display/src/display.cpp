#include "display.hpp"

#include <GLFW/glfw3.h>
#include <webgpu/webgpu_glfw.h>

#define GLM_ENABLE_EXPERIMENTAL
#include <glm/gtx/quaternion.hpp>
#include <glm/gtc/type_ptr.hpp>
#define TINYGLTF_IMPLEMENTATION
#define STB_IMAGE_IMPLEMENTATION
#define STB_IMAGE_WRITE_IMPLEMENTATION

#include "tiny_gltf.h"

#include <imgui.h>
#include <backends/imgui_impl_wgpu.h>
#include <backends/imgui_impl_glfw.h>

#include <print>
#include <iostream>
#include <filesystem>
#include <random>

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

glm::mat4 display::Camera::BuildProjMatrix() const {
    return glm::perspective(fovy, aspect, znear, zfar);
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
    // sphere representing atoms will be rendered as an s orbital instance with different color
    std::vector<InstanceData> sInstances;
    std::vector<InstanceData> spInstances;
    std::vector<InstanceData> pInstances;
    // build translation + rotation matrix for each atom that will be applied to its set of orbitals
    // translation + rotation are from locs + rots
    for (const auto& atom : bondedAtoms) {
        // model consists of each atom with its orbitals and sphere
        auto bondQuatPairs = atom.ToMatrix();

        // construct model matrix
        glm::mat4 atomModelMatrix = glm::mat4(1.0f);

        // translate: move the whole thing to the atom's world position
        atomModelMatrix = glm::translate(atomModelMatrix, atom.loc);

        // rotate: apply the atom's overall rotation, then the specific orbital's rotation
        atomModelMatrix = atomModelMatrix * glm::toMat4(atom.rot);

        // since spheres are s orbitals prepend to sIntances vector
        sInstances.insert(
                sInstances.begin(),
                InstanceData{
                    .modelMatrix = glm::scale(atomModelMatrix, glm::vec3(S_ORBITAL_SCALE / 1.5)),
                    .color = glm::vec4(.5, .5, .5, 1)
                }
        );

        // apply transforms to each of the atom's orbitals
        for (auto& bQPair : bondQuatPairs.orbitals) {
            auto orbitalModelMatrix = atomModelMatrix * glm::toMat4(bQPair.second);

            // scale: scale the mesh at its local origin
            float scaleFactor = 1.0f;
            if (bQPair.first == molecule::OrbitalType::s) scaleFactor = S_ORBITAL_SCALE;
            else if (bQPair.first == molecule::OrbitalType::sp) scaleFactor = SP_ORBITAL_SCALE;
            else if (bQPair.first == molecule::OrbitalType::p) scaleFactor = P_ORBITAL_SCALE;

            orbitalModelMatrix = glm::scale(orbitalModelMatrix, glm::vec3(scaleFactor));

            // depending on the orbital type, add to corresponding vector
            switch (bQPair.first) {
                case molecule::OrbitalType::s:
                    sInstances.push_back(InstanceData { .modelMatrix = orbitalModelMatrix, .color = glm::vec4(0.3f, 0.f, 0.3f, 1.f)});
                    break;
                case molecule::OrbitalType::sp:
                    spInstances.push_back(InstanceData { .modelMatrix = orbitalModelMatrix, .color = glm::vec4(0.f, 0.3f, 0.45f, 1.f) });
                    break;
                case molecule::OrbitalType::p:
                    pInstances.push_back(InstanceData { .modelMatrix = orbitalModelMatrix, .color = glm::vec4(0.45f, 0.f, 0.2f, 1.f) });
                    break;
            }
        }
    }

    // only create buffers for which exist orbital instances
    if (sInstances.size() > 0) {
        wgpu::BufferDescriptor sOrbitalVertexBufferDesc {
            .label = "s Orbital Vertex Buffer",
            .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Vertex,
            .size = sizeof(InstanceData) * sInstances.size(),
        };
        instances["s"] = Instances{
                .instanceBuffer = this->device.CreateBuffer(&sOrbitalVertexBufferDesc),
                .instanceData = sInstances,
        };
    }
    if (spInstances.size() > 0) {
        wgpu::BufferDescriptor spOrbitalVertexBufferDesc {
            .label = "sp Orbital Instance Vertex Buffer",
            .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Vertex,
            .size = sizeof(InstanceData) * spInstances.size(),
        };
        instances["sp"] = Instances {
                .instanceBuffer = this->device.CreateBuffer(&spOrbitalVertexBufferDesc),
                .instanceData = spInstances,
        };
    }
    if (pInstances.size() > 0) {
        wgpu::BufferDescriptor pOrbitalVertexBufferDesc {
            .label = "p Orbital Vertex Buffer",
            .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Vertex,
            .size = sizeof(InstanceData) * pInstances.size(),
        };
        instances["p"] = Instances{
                .instanceBuffer = this->device.CreateBuffer(&pOrbitalVertexBufferDesc),
                .instanceData = pInstances,
        };
    }

    std::println("s instance count: {}", sInstances.size());
    std::println("sp instance count: {}", spInstances.size());
    std::println("p instance count: {}", pInstances.size());

    // write instance data to buffer for each instance
    for (auto& [_, instance] : instances) {
        // vertex + color
        queue.WriteBuffer(instance.instanceBuffer, 0, instance.GetRawData().data(), instance.GetRawData().size_bytes());
    }
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

    /// vertices + normals
    // go to accessor at index POSITION
    const auto& posAccessor = model.accessors.at(primitive.attributes.at("POSITION"));
    // go to accessor at index NORMAL
    const auto& normAccessor = model.accessors.at(primitive.attributes.at("NORMAL"));

    // which bufferView to look at based on what index the accessor at index "POSITION" shows
    const auto& posView = model.bufferViews.at(posAccessor.bufferView);
    const auto& posBuffer = model.buffers.at(posView.buffer);
    // which bufferView to look at based on what index the accessor at index "NORMAL" shows
    const auto& normView = model.bufferViews.at(normAccessor.bufferView);
    const auto& normBuffer = model.buffers.at(normView.buffer);
    
    // calculate size to create buffer
    size_t posDataSize = posAccessor.count * sizeof(float) * 3;
    // pointer to actual data
    const float* posDataPtr = reinterpret_cast<const float *>(&posBuffer.data[posView.byteOffset + posAccessor.byteOffset]);
    // check that pointer contains data
    if (posDataPtr == nullptr) {
        return std::unexpected(std::format("{} has invalid pos data", filePath));
    }

    // calculate size to create buffer
    size_t normDataSize = normAccessor.count * sizeof(float) * 3;
    // pointer to actual data
    const float* normDataPtr = reinterpret_cast<const float *>(&normBuffer.data[normView.byteOffset + normAccessor.byteOffset]);
    // check that pointer contains data
    if (normDataPtr == nullptr) {
        return std::unexpected(std::format("{} has invalid normal data", filePath));
    }

    // stores each vertex's position and normal
    std::vector<Vertex> vertices;
    for (size_t i = 0; i < posAccessor.count; i++) {
        vertices.emplace_back(
            glm::make_vec3(&posDataPtr[i * 3]),
            glm::make_vec3(&normDataPtr[i * 3])
        );
    }

    // create vertex buffer
    wgpu::BufferDescriptor vertexBufferDesc{
        .label = std::format("{} Position + Normal Buffer", filePath).c_str(),
        .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Vertex,
        .size = vertices.size() * sizeof(Vertex),
    };
    outputMesh.vertexBuffer = this->device.CreateBuffer(&vertexBufferDesc);
    this->queue.WriteBuffer(outputMesh.vertexBuffer, 0, vertices.data(), vertices.size() * sizeof(Vertex));

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

void display::Application::CreateGeometryRenderPipeline() {
    wgpu::RenderPipelineDescriptor geoRenderPipelineDescriptor { .label = "Geometry Render Pipeline" };
    auto shaderModule = LoadShaderModule("res/shaders/shader.wgsl");

    /// describe vertex buffer layouts
    // define position attribute
    wgpu::VertexAttribute posAttribute{
        .format = wgpu::VertexFormat::Float32x3,
        .offset = 0,
        .shaderLocation = 0,
    };
    // define normal attribute
    wgpu::VertexAttribute normAttribute{
        .format = wgpu::VertexFormat::Float32x3,
        .offset = offsetof(Vertex, norm),
        .shaderLocation = 1,
    };

    std::vector<wgpu::VertexAttribute> vertexAttributes = { posAttribute, normAttribute };
    wgpu::VertexBufferLayout posBufferLayout{
        .stepMode = wgpu::VertexStepMode::Vertex,
        .arrayStride = sizeof(Vertex),
        .attributeCount = vertexAttributes.size(),
        .attributes = vertexAttributes.data(),
    };

    // define instance attributes
    std::vector<wgpu::VertexAttribute> instanceAttributes(5);
    for (size_t i{0}; i < 5; i++) {
        instanceAttributes[i].format = wgpu::VertexFormat::Float32x4;
        instanceAttributes[i].offset = i * sizeof(glm::vec4);
        instanceAttributes[i].shaderLocation = i + 5;
    }
    wgpu::VertexBufferLayout instanceBufferLayout{
        .stepMode = wgpu::VertexStepMode::Instance,
        .arrayStride = sizeof(glm::vec4) * 5, // model matrix + color for each instance
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
    geoRenderPipelineDescriptor.vertex = vertexState;

    /// describe primitive pipeline state
    wgpu::PrimitiveState primitiveState{
        .topology = wgpu::PrimitiveTopology::TriangleList,
        .frontFace = wgpu::FrontFace::CCW,
        .cullMode = wgpu::CullMode::None,
    };
    geoRenderPipelineDescriptor.primitive = primitiveState;

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
    wgpu::ColorTargetState normalTargetState{
        .format = textureFormat,
        .writeMask = wgpu::ColorWriteMask::All,
    };
    std::vector<wgpu::ColorTargetState> targets { colorTargetState, normalTargetState };
    wgpu::FragmentState fragmentState{
        .module = shaderModule,
        .entryPoint = "fs_main",
        .targetCount = 2, // color + normal texture
        .targets = targets.data(),
    };
    geoRenderPipelineDescriptor.fragment = &fragmentState;

    /// describe stencil/depth fragment state
    wgpu::DepthStencilState depthStencilState{
        .format = wgpu::TextureFormat::Depth32Float,
        .depthWriteEnabled = wgpu::OptionalBool::True,
        .depthCompare = wgpu::CompareFunction::Less, // fragment is blended only if depth is less than current
        .stencilReadMask = 0,
        .stencilWriteMask = 0,
    };
    geoRenderPipelineDescriptor.depthStencil = &depthStencilState;

    /// describe pipeline layout
    std::vector<wgpu::BindGroupLayout> bindGroupLayouts = {
        cameraBindGroupLayout,
    };
    wgpu::PipelineLayoutDescriptor pipelineLayoutDesc{
        .label = "Render Pipeline Layout",
        .bindGroupLayoutCount = 1,
        .bindGroupLayouts = bindGroupLayouts.data(),
    };
    geoRenderPipelineDescriptor.layout = device.CreatePipelineLayout(&pipelineLayoutDesc);

    /// create render pipeline
    geoRenderPipeline = device.CreateRenderPipeline(&geoRenderPipelineDescriptor);

    /// create color texture
    wgpu::TextureDescriptor colorTextureDesc{
        .label = "Color Texture",
        .usage = wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::TextureBinding,
        .dimension = wgpu::TextureDimension::e2D,
        .size = {WIDTH, HEIGHT, 1},
        .format = textureFormat,
    };
    colorTexture = device.CreateTexture(&colorTextureDesc);

    /// create view for color texture
    wgpu::TextureViewDescriptor colorTextureViewDesc{
        .label = "Color Texture View",
        .format = colorTexture.GetFormat(),
        .dimension = wgpu::TextureViewDimension::e2D,
        .mipLevelCount = 1,
        .arrayLayerCount = 1,
        .aspect = wgpu::TextureAspect::All,
    };
    colorTextureView = colorTexture.CreateView(&colorTextureViewDesc);

    /// create depth texture
    wgpu::TextureDescriptor depthTextureDesc{
        .label = "Depth Texture",
        .usage = wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::TextureBinding,
        .dimension = wgpu::TextureDimension::e2D,
        .size = {WIDTH, HEIGHT, 1},
        .format = wgpu::TextureFormat::Depth32Float,
    };
    depthTexture = device.CreateTexture(&depthTextureDesc);

    /// create depth texture view
    wgpu::TextureViewDescriptor depthTextureViewDesc{
        .label = "Depth Texture View",
        .format = wgpu::TextureFormat::Depth32Float,
        .dimension = wgpu::TextureViewDimension::e2D,
        .mipLevelCount = 1,
        .arrayLayerCount = 1,
        .aspect = wgpu::TextureAspect::DepthOnly,
        .usage = wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::TextureBinding,
    };
    depthTextureView = depthTexture.CreateView(&depthTextureViewDesc);

    /// create normal texture
    wgpu::TextureDescriptor normalTextureDesc{
        .label = "Normal Texture Descriptor",
        .usage = wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::TextureBinding,
        .dimension = wgpu::TextureDimension::e2D,
        .size = {WIDTH, HEIGHT, 1},
        .format = textureFormat,
    };
    normalTexture = device.CreateTexture(&normalTextureDesc);

    /// create view for normal texture
    wgpu::TextureViewDescriptor normalTextureViewDesc{
        .label = "Normal Texture View",
        .format = normalTexture.GetFormat(),
        .dimension = wgpu::TextureViewDimension::e2D,
        .mipLevelCount = 1,
        .arrayLayerCount = 1,
        .aspect = wgpu::TextureAspect::All,
    };
    normalTextureView = normalTexture.CreateView(&normalTextureViewDesc);
}

void display::Application::CreateSSAOPipeline() {
    // SSAO inputs: 
    // uniforms: proj, inv proj, points to sample on hemisphere, radius of hemisphere, bias
    // populated depth, normals, noise texture, sampler, output texture

    wgpu::ComputePipelineDescriptor ssaoPipelineDescriptor { .label = "SSAO Pipeline" };
    auto ssaoShaderModule = LoadShaderModule("res/shaders/ssao.wgsl");

    ssaoPipelineDescriptor.compute = {
        .module = ssaoShaderModule,
        .entryPoint = "compute_main",
    };

    /// assemble uniforms into one bind group
    float radius{0.133f}, bias{0.185f};
    // generate kernel of points to sample on hemisphere
    std::uniform_real_distribution<float> randFloats(0.0, 1.0);
    std::default_random_engine generator;
    for (int i{0}; i < 64; i++) {
        // create points in hemisphere in +z dir
        glm::vec3 sample(
            randFloats(generator) * 2.0 - 1.0, // -1 to 1
            randFloats(generator) * 2.0 - 1.0,
            randFloats(generator) // 0 to 1
        );
        // each point is on surface of hemisphere
        sample = glm::normalize(sample);
        // accelerating scale function
        float scale = (float)i / 64.0f;
        scale = glm::mix(0.1f, 1.0f, scale * scale);
        sample *= scale;
        ssaoUniforms.kernel[i] = glm::vec4(sample, 0.f);
    }

    wgpu::BufferDescriptor ssaoBufferDesc{
        .label = "SSAO Uniform Buffer",
        .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::Uniform,
        .size = sizeof(SSAOUniforms),
    };
    ssaoUniformBuffer = this->device.CreateBuffer(&ssaoBufferDesc);
    // proj and inv proj, radius, and bias are updated every frame so write kernel for now
    this->queue.WriteBuffer(ssaoUniformBuffer, offsetof(SSAOUniforms, kernel), ssaoUniforms.kernel, 64 * sizeof(glm::vec4));
    
    std::vector<wgpu::BindGroupLayoutEntry> ssaoBindGroupLayoutEntries{
        // ssao uniforms
        wgpu::BindGroupLayoutEntry{
            .binding = 0,
            .visibility = wgpu::ShaderStage::Compute,
            .buffer = {
                .type = wgpu::BufferBindingType::Uniform,
            }
        },
        // depth texture
        wgpu::BindGroupLayoutEntry{
            .binding = 1,
            .visibility = wgpu::ShaderStage::Compute,
            .texture = {
                .sampleType = wgpu::TextureSampleType::Depth,
                .viewDimension = wgpu::TextureViewDimension::e2D,
            }
        },
        // normal texture
        wgpu::BindGroupLayoutEntry{
            .binding = 2,
            .visibility = wgpu::ShaderStage::Compute,
            .texture = {
                .sampleType = wgpu::TextureSampleType::Float,
                .viewDimension = wgpu::TextureViewDimension::e2D,
            },
        },
        // noise texture
        wgpu::BindGroupLayoutEntry{
            .binding = 3,
            .visibility = wgpu::ShaderStage::Compute,
            .texture = {
                .sampleType = wgpu::TextureSampleType::UnfilterableFloat,
                .viewDimension = wgpu::TextureViewDimension::e2D,
            },
        },
        // sampler
        wgpu::BindGroupLayoutEntry{
            .binding = 4,
            .visibility = wgpu::ShaderStage::Compute,
            .sampler = { .type = wgpu::SamplerBindingType::Filtering },
        },
        // output texture
        wgpu::BindGroupLayoutEntry{
            .binding = 5,
            .visibility = wgpu::ShaderStage::Compute,
            .storageTexture = {
                .access = wgpu::StorageTextureAccess::ReadWrite,
                .format = wgpu::TextureFormat::R32Float,
                .viewDimension = wgpu::TextureViewDimension::e2D,
            }
        },
    };
    wgpu::BindGroupLayoutDescriptor ssaoBindGroupLayoutDesc{
        .label = "SSAO Bind Group Layout",
        .entryCount = 6,
        .entries = ssaoBindGroupLayoutEntries.data(),
    };
    wgpu::BindGroupLayout ssaoBindGroupLayout = device.CreateBindGroupLayout(&ssaoBindGroupLayoutDesc);

    /// describe pipeline layout
    std::vector<wgpu::BindGroupLayout> bindGroupLayouts = {
        ssaoBindGroupLayout,
    };
    wgpu::PipelineLayoutDescriptor ssaoPipelineLayoutDesc{
        .label = "SSAO Pipeline Layout",
        .bindGroupLayoutCount = 1,
        .bindGroupLayouts = bindGroupLayouts.data(),
    };
    ssaoPipelineDescriptor.layout = device.CreatePipelineLayout(&ssaoPipelineLayoutDesc);

    ssaoPipeline = device.CreateComputePipeline(&ssaoPipelineDescriptor);

    // depth and normal textures are already created in render pipeline
    // create noise texture (16x16 random values since bytes per row in texture data must be multiple of 256)
    std::vector<glm::vec4> noise;
    for (int i = 0; i < 256; i++) {
        noise.emplace_back(randFloats(generator) * 2.f - 1.f, randFloats(generator) * 2.f - 1.f, 0.f, 0.f);
    }
    wgpu::TextureDescriptor noiseTextureDesc{
        .label = "Noise Texture",
        .usage = wgpu::TextureUsage::CopyDst | wgpu::TextureUsage::TextureBinding,
        .dimension = wgpu::TextureDimension::e2D,
        .size = {16, 16, 1},
        .format = wgpu::TextureFormat::RGBA32Float,
        .mipLevelCount = 1,
        .sampleCount = 1,
    };
    noiseTexture = device.CreateTexture(&noiseTextureDesc);
    // write noise data to the texture
    wgpu::TexelCopyTextureInfo texelCopyTextureInfo{
        .texture = noiseTexture,
        .origin = {0, 0, 0},
        .aspect = wgpu::TextureAspect::All,
    };
    wgpu::TexelCopyBufferLayout texelCopyBufferLayout{
        .bytesPerRow = 16 * sizeof(glm::vec4),
        .rowsPerImage = 16,
    };
    wgpu::Extent3D extent3d{16, 16, 1};
    queue.WriteTexture(
        &texelCopyTextureInfo,
        noise.data(),
        noise.size() * sizeof(glm::vec4),
        &texelCopyBufferLayout,
        &extent3d
    );

    // create texture view for noise
    wgpu::TextureViewDescriptor noiseTextureViewDesc{
        .label = "Noise Texture View",
        .format = noiseTexture.GetFormat(),
        .dimension = wgpu::TextureViewDimension::e2D,
        .baseMipLevel = 0,
        .mipLevelCount = 1,
        .baseArrayLayer = 0,
        .arrayLayerCount = 1,
        .aspect = wgpu::TextureAspect::All,
    };
    wgpu::TextureView noiseTextureView = noiseTexture.CreateView(&noiseTextureViewDesc);

    // sampler for noise and normal textures
    wgpu::SamplerDescriptor samplerDesc{
        .label = "Sampler",
        .addressModeU = wgpu::AddressMode::Repeat,
        .addressModeV = wgpu::AddressMode::Repeat,
        .magFilter = wgpu::FilterMode::Linear,
        .minFilter = wgpu::FilterMode::Linear,
    };
    linearSampler = device.CreateSampler(&samplerDesc);

    // create ambient occlusion output texture
    wgpu::TextureDescriptor ssaoTextureDesc{
        .label = "SSAO Texture",
        .usage = wgpu::TextureUsage::CopyDst | wgpu::TextureUsage::TextureBinding | wgpu::TextureUsage::StorageBinding,
        .dimension = wgpu::TextureDimension::e2D,
        .size = {WIDTH / 2, HEIGHT / 2, 1},
        .format = wgpu::TextureFormat::R32Float,
        .mipLevelCount = 1,
        .sampleCount = 1,
    };
    ssaoTexture = device.CreateTexture(&ssaoTextureDesc);

    // texture view for ssao output
    wgpu::TextureViewDescriptor ssaoTextureViewDesc{
        .label = "SSAO Texture View",
        .format = ssaoTexture.GetFormat(),
        .dimension = wgpu::TextureViewDimension::e2D,
        .baseMipLevel = 0,
        .mipLevelCount = 1,
        .baseArrayLayer = 0,
        .arrayLayerCount = 1,
        .aspect = wgpu::TextureAspect::All,
    };
    ssaoTextureView = ssaoTexture.CreateView(&ssaoTextureViewDesc);

    // describe ssao bind group entries
    std::vector<wgpu::BindGroupEntry> ssaoBindGroupEntries {
        // uniforms
        wgpu::BindGroupEntry{
            .binding = 0,
            .buffer = ssaoUniformBuffer,
            .size = sizeof(SSAOUniforms),
        },
        // depth texture
        wgpu::BindGroupEntry{
            .binding = 1,
            .textureView = depthTextureView,
        },
        // normal texture
        wgpu::BindGroupEntry{
            .binding = 2,
            .textureView = normalTextureView,
        },
        // noise texture
        wgpu::BindGroupEntry{
            .binding = 3,
            .textureView = noiseTextureView,
        },
        // sampler
        wgpu::BindGroupEntry{
            .binding = 4,
            .sampler = linearSampler,
        },
        // ssao texture
        wgpu::BindGroupEntry{
            .binding = 5,
            .textureView = ssaoTextureView,
        }
    };

    wgpu::BindGroupDescriptor ssaoBindGroupDesc{
        .label = "SSAO Bind Group",
        .layout = ssaoBindGroupLayout,
        .entryCount = ssaoBindGroupLayoutEntries.size(),
        .entries = ssaoBindGroupEntries.data(),
    };
    ssaoBindGroup = device.CreateBindGroup(&ssaoBindGroupDesc);


    // ssao blur compute pipeline
    wgpu::ComputePipelineDescriptor ssaoBlurPipelineDescriptor { .label = "SSAO Blur Pipeline" };
    auto ssaoBlurShaderModule = LoadShaderModule("res/shaders/blur_ssao.wgsl");

    ssaoBlurPipelineDescriptor.compute = {
        .module = ssaoBlurShaderModule,
        .entryPoint = "blur_ssao_main",
    };

    std::vector<wgpu::BindGroupLayoutEntry> ssaoBlurBindGroupLayoutEntries{
        // ssao texture in
        wgpu::BindGroupLayoutEntry{
            .binding = 0,
            .visibility = wgpu::ShaderStage::Compute,
            .texture = {
                .sampleType = wgpu::TextureSampleType::UnfilterableFloat,
                .viewDimension = wgpu::TextureViewDimension::e2D,
            }
        },
        // depth texture
        wgpu::BindGroupLayoutEntry{
            .binding = 1,
            .visibility = wgpu::ShaderStage::Compute,
            .texture = {
                .sampleType = wgpu::TextureSampleType::Depth,
                .viewDimension = wgpu::TextureViewDimension::e2D,
            },
        },
        // output ssao blur texture
        wgpu::BindGroupLayoutEntry{
            .binding = 2,
            .visibility = wgpu::ShaderStage::Compute,
            .storageTexture = {
                .access = wgpu::StorageTextureAccess::WriteOnly,
                .format = wgpu::TextureFormat::R32Float,
                .viewDimension = wgpu::TextureViewDimension::e2D,
            }
        },
    };
    wgpu::BindGroupLayoutDescriptor ssaoBlurBindGroupLayoutDesc{
        .label = "SSAO Blur Bind Group Layout",
        .entryCount = 3,
        .entries = ssaoBlurBindGroupLayoutEntries.data(),
    };
    wgpu::BindGroupLayout ssaoBlurBindGroupLayout = device.CreateBindGroupLayout(&ssaoBlurBindGroupLayoutDesc);

    /// describe pipeline layout
    std::vector<wgpu::BindGroupLayout> ssaoBlurBindGroupLayouts = {
        ssaoBlurBindGroupLayout,
    };
    wgpu::PipelineLayoutDescriptor ssaoBlurPipelineLayoutDesc{
        .label = "SSAO Blur Pipeline Layout",
        .bindGroupLayoutCount = 1,
        .bindGroupLayouts = ssaoBlurBindGroupLayouts.data(),
    };
    ssaoBlurPipelineDescriptor.layout = device.CreatePipelineLayout(&ssaoBlurPipelineLayoutDesc);

    ssaoBlurPipeline = device.CreateComputePipeline(&ssaoBlurPipelineDescriptor);

    // create ambient occlusion output texture
    wgpu::TextureDescriptor ssaoBlurTextureDesc{
        .label = "SSAO Blur Texture",
        .usage = wgpu::TextureUsage::CopyDst | wgpu::TextureUsage::TextureBinding | wgpu::TextureUsage::StorageBinding,
        .dimension = wgpu::TextureDimension::e2D,
        .size = {WIDTH / 2, HEIGHT / 2, 1},
        .format = wgpu::TextureFormat::R32Float,
        .mipLevelCount = 1,
        .sampleCount = 1,
    };
    ssaoBlurTexture = device.CreateTexture(&ssaoBlurTextureDesc);

    // texture view for ssao output
    wgpu::TextureViewDescriptor ssaoBlurTextureViewDesc{
        .label = "SSAO Blur Texture View",
        .format = ssaoBlurTexture.GetFormat(),
        .dimension = wgpu::TextureViewDimension::e2D,
        .baseMipLevel = 0,
        .mipLevelCount = 1,
        .baseArrayLayer = 0,
        .arrayLayerCount = 1,
        .aspect = wgpu::TextureAspect::All,
    };
    ssaoBlurTextureView = ssaoBlurTexture.CreateView(&ssaoBlurTextureViewDesc);

    // describe ssao blur bind group entries
    std::vector<wgpu::BindGroupEntry> ssaoBlurBindGroupEntries {
        // ssao texture in
        wgpu::BindGroupEntry{
            .binding = 0,
            .textureView = ssaoTextureView,
        },
        // depth texture
        wgpu::BindGroupEntry{
            .binding = 1,
            .textureView = depthTextureView,
        },
        // noise texture
        wgpu::BindGroupEntry{
            .binding = 2,
            .textureView = ssaoBlurTextureView,
        },
    };

    wgpu::BindGroupDescriptor ssaoBlurBindGroupDesc{
        .label = "SSAO Blur Bind Group",
        .layout = ssaoBlurBindGroupLayout,
        .entryCount = ssaoBlurBindGroupLayoutEntries.size(),
        .entries = ssaoBlurBindGroupEntries.data(),
    };
    ssaoBlurBindGroup = device.CreateBindGroup(&ssaoBlurBindGroupDesc);
}

void display::Application::CreateCompositeRenderPipeline() {
    wgpu::RenderPipelineDescriptor compositeRenderPipelineDescriptor { .label = "Composite Render Pipeline" };
    auto shaderModule = LoadShaderModule("res/shaders/composite.wgsl");

    // vertex state
    wgpu::VertexState vertexState{
        .module = shaderModule,
        .entryPoint = "vs_main",
    };
    compositeRenderPipelineDescriptor.vertex = vertexState;

    /// describe primitive pipeline state
    wgpu::PrimitiveState primitiveState{
        .topology = wgpu::PrimitiveTopology::TriangleList,
        .frontFace = wgpu::FrontFace::CCW,
        .cullMode = wgpu::CullMode::None,
    };
    compositeRenderPipelineDescriptor.primitive = primitiveState;

    // fragment state
    wgpu::BlendState blendState{
        .color =
            wgpu::BlendComponent{
                .operation = wgpu::BlendOperation::Add,
                .srcFactor = wgpu::BlendFactor::One,
                .dstFactor = wgpu::BlendFactor::Zero,
            },
        .alpha = wgpu::BlendComponent{
            .operation = wgpu::BlendOperation::Add,
            .srcFactor = wgpu::BlendFactor::One,
            .dstFactor = wgpu::BlendFactor::Zero,
        }};
    wgpu::ColorTargetState colorTargetState{
        .format = textureFormat,
        .blend = &blendState,
        .writeMask = wgpu::ColorWriteMask::All,
    };
    std::vector<wgpu::ColorTargetState> targets { colorTargetState };
    wgpu::FragmentState fragmentState{
        .module = shaderModule,
        .entryPoint = "fs_main",
        .targetCount = 1,
        .targets = targets.data(),
    };
    compositeRenderPipelineDescriptor.fragment = &fragmentState;

    // bind group layout
    std::vector<wgpu::BindGroupLayoutEntry> compositeBindGroupLayoutEntries{
        // color texture
        wgpu::BindGroupLayoutEntry{
            .binding = 0,
            .visibility = wgpu::ShaderStage::Fragment,
            .texture = {
                .sampleType = wgpu::TextureSampleType::Float,
                .viewDimension = wgpu::TextureViewDimension::e2D,
            },
        },
        // ssao texture
        wgpu::BindGroupLayoutEntry{
            .binding = 1,
            .visibility = wgpu::ShaderStage::Fragment,
            .texture = {
                .sampleType = wgpu::TextureSampleType::UnfilterableFloat,
                .viewDimension = wgpu::TextureViewDimension::e2D,
            },
        },
        // composite sampler
        wgpu::BindGroupLayoutEntry{
            .binding = 2,
            .visibility = wgpu::ShaderStage::Fragment,
            .sampler = { .type = wgpu::SamplerBindingType::Filtering },
        },
    };
    wgpu::BindGroupLayoutDescriptor compositeBindGroupLayoutDesc{
        .label = "Composite Bind Group Layout",
        .entryCount = 3,
        .entries = compositeBindGroupLayoutEntries.data(),
    };

    wgpu::BindGroupLayout compositeBindGroupLayout = device.CreateBindGroupLayout(&compositeBindGroupLayoutDesc);

    /// describe pipeline layout
    std::vector<wgpu::BindGroupLayout> bindGroupLayouts = {
        compositeBindGroupLayout,
    };
    wgpu::PipelineLayoutDescriptor compositePipelineLayoutDesc{
        .label = "Composite Pipeline Layout",
        .bindGroupLayoutCount = 1,
        .bindGroupLayouts = bindGroupLayouts.data(),
    };
    compositeRenderPipelineDescriptor.layout = device.CreatePipelineLayout(&compositePipelineLayoutDesc);

    compositeRenderPipeline = device.CreateRenderPipeline(&compositeRenderPipelineDescriptor);

    // describe composite bind group entries
    std::vector<wgpu::BindGroupEntry> compositeBindGroupEntries {
        // color texture
        wgpu::BindGroupEntry{
            .binding = 0,
            .textureView = colorTextureView,
        },
        // ssao texture
        wgpu::BindGroupEntry{
            .binding = 1,
            .textureView = ssaoBlurTextureView,
        },
        // sampler
        wgpu::BindGroupEntry{
            .binding = 2,
            .sampler = linearSampler,
        },
    };

    wgpu::BindGroupDescriptor compositeBindGroupDesc{
        .label = "Composite Bind Group",
        .layout = compositeBindGroupLayout,
        .entryCount = 3,
        .entries = compositeBindGroupEntries.data(),
    };
    compositeBindGroup = device.CreateBindGroup(&compositeBindGroupDesc);
}

wgpu::TextureView display::Application::GetNextSurfaceTextureView() {
    // get next texture to be presented
    wgpu::SurfaceTexture surfaceTexture;
    surface.GetCurrentTexture(&surfaceTexture);

    // create view for this texture
    wgpu::TextureViewDescriptor textureViewDesc{
        .label = "Color Texture View",
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
    wgpu::InstanceDescriptor instance_desc {
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

    // create instances from solved molecule
    std::string jsonFilePath = "/home/seb/projects/lewis-structure-visualizer/solver/out/" + moleculePath;
    std::string csvFilePath = "/home/seb/projects/lewis-structure-visualizer/data/data.csv";
    molecule::Molecule molecule(jsonFilePath, csvFilePath);
    auto bondedAtoms = molecule.ComputeAtomLocsRots().value();

    CreateInstances(bondedAtoms);
    std::println("Loaded instances...");

    CreateGeometryRenderPipeline();
    std::println("Created geometry render pipeline...");

    CreateSSAOPipeline();
    std::println("Created SSAO pipeline...");

    CreateCompositeRenderPipeline();
    std::println("Created composite render pipeline...");

    // init GUI
    IMGUI_CHECKVERSION();
    ImGui::CreateContext();
    ImGui::StyleColorsDark();

    ImGui_ImplGlfw_InitForOther(window, true);
    ImGui_ImplWGPU_InitInfo initInfo;
    initInfo.Device = device.Get();
    initInfo.NumFramesInFlight = 3;
    initInfo.RenderTargetFormat = static_cast<WGPUTextureFormat>(wgpu::TextureFormat::RGBA16Float);
    ImGui_ImplWGPU_Init(&initInfo);

    return true;
}

void display::Application::RenderPresent() {
    ImGui_ImplWGPU_NewFrame();
    ImGui_ImplGlfw_NewFrame();
    ImGui::NewFrame();

    ImGui::Begin("SSAO Parameters");
    ImGui::SliderFloat("Radius", &ssaoUniforms.radius, 0.001f, 10.0f);
    ImGui::SliderFloat("Bias", &ssaoUniforms.bias, 0.001f, 10.0f);
    ImGui::End();

    wgpu::RenderPassColorAttachment colorAttachment{
        .view = colorTextureView,
        .loadOp = wgpu::LoadOp::Clear,
        .storeOp = wgpu::StoreOp::Store,
    };

    wgpu::RenderPassColorAttachment normalAttachment{
        .view = normalTextureView,
        .loadOp = wgpu::LoadOp::Clear,
        .storeOp = wgpu::StoreOp::Store,
    };

    wgpu::RenderPassDepthStencilAttachment depthStencilAttachment{
        .view = depthTextureView,
        .depthLoadOp = wgpu::LoadOp::Clear,
        .depthStoreOp = wgpu::StoreOp::Store,
        // initial value of depth buffer, signifying the far value in this case
        .depthClearValue = 1.0f,
        .depthReadOnly = false,
    };

    // create surface attachment
    wgpu::RenderPassColorAttachment surfaceAttachment{
        .view = GetNextSurfaceTextureView(),
        .loadOp = wgpu::LoadOp::Clear,
        .storeOp = wgpu::StoreOp::Store,
        .clearValue = wgpu::Color{0.0, 0.0, 0.0, 1.0},
    };

    // attach textures to the geometry render pass: color + normal
    std::vector<wgpu::RenderPassColorAttachment> geoAttachments { colorAttachment, normalAttachment };
    wgpu::RenderPassDescriptor geoRenderPassDesc{
        .colorAttachmentCount = 2,
        .colorAttachments = geoAttachments.data(),
        .depthStencilAttachment = &depthStencilAttachment,
    };

    // attach surface texture to the composite render pass
    std::vector<wgpu::RenderPassColorAttachment> compositeAttachments { surfaceAttachment };
    wgpu::RenderPassDescriptor compositeRenderPassDesc{
        .colorAttachmentCount = 1,
        .colorAttachments = compositeAttachments.data(),
    };

    // process user input
    ProcessInput();
    // update and write camera data to buffer
    camera.Update();
    glm::mat4 viewProjMat = camera.BuildViewProjectionMatrix();
    queue.WriteBuffer(camera.cameraBuffer, 0, &viewProjMat, sizeof(viewProjMat));

    // write camera projection and inverse to ssao uniform
    ssaoUniforms.proj = camera.BuildProjMatrix();
    ssaoUniforms.invProj = glm::inverse(ssaoUniforms.proj);
    queue.WriteBuffer(ssaoUniformBuffer, 0, &ssaoUniforms.proj, sizeof(ssaoUniforms.proj));
    queue.WriteBuffer(ssaoUniformBuffer, offsetof(SSAOUniforms, invProj), &ssaoUniforms.invProj, sizeof(ssaoUniforms.invProj));
    queue.WriteBuffer(ssaoUniformBuffer, offsetof(SSAOUniforms, radius), &ssaoUniforms.radius, sizeof(ssaoUniforms.radius));
    queue.WriteBuffer(ssaoUniformBuffer, offsetof(SSAOUniforms, bias), &ssaoUniforms.bias, sizeof(ssaoUniforms.bias));

    // command encoder for drawing + compute
    wgpu::CommandEncoderDescriptor commandEncoderDesc{ .label = "Command Encoder" };
    wgpu::CommandEncoder commandEncoder = device.CreateCommandEncoder(&commandEncoderDesc);

    {
        // record the render pass
        wgpu::RenderPassEncoder renderPass = commandEncoder.BeginRenderPass(&geoRenderPassDesc);
        renderPass.SetPipeline(geoRenderPipeline);

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
            // draw
            renderPass.DrawIndexed(meshes.at(orbitalType).indexCount, instance.instanceData.size());
        }

        renderPass.End();
    }
    {
        // record ssao pass
        wgpu::ComputePassEncoder ssaoPass = commandEncoder.BeginComputePass();
        ssaoPass.SetPipeline(ssaoPipeline);

        // bind uniforms bind group
        ssaoPass.SetBindGroup(0, ssaoBindGroup);
        
        // shader uses 8x8 workgroups, so must calculate how many groups to fill entire screen
        ssaoPass.DispatchWorkgroups(static_cast<uint32_t>((WIDTH/2 + 7) / 8), static_cast<uint32_t>((HEIGHT/2 + 7) / 8));

        ssaoPass.End();
    }
    {
        // record ssao blur pass
        wgpu::ComputePassEncoder ssaoBlurPass = commandEncoder.BeginComputePass();
        ssaoBlurPass.SetPipeline(ssaoBlurPipeline);

        // bind uniforms bind group
        ssaoBlurPass.SetBindGroup(0, ssaoBlurBindGroup);
        
        // shader uses 8x8 workgroups, so must calculate how many groups to fill entire screen
        ssaoBlurPass.DispatchWorkgroups(static_cast<uint32_t>((WIDTH/2+ 7) / 8), static_cast<uint32_t>((HEIGHT/2 + 7) / 8));

        ssaoBlurPass.End();
    }
    {
        // record composite pass
        wgpu::RenderPassEncoder compositePass = commandEncoder.BeginRenderPass(&compositeRenderPassDesc);
        compositePass.SetPipeline(compositeRenderPipeline);

        compositePass.SetBindGroup(0, compositeBindGroup);
        // draw three vertices for the triangle
        compositePass.Draw(3);
        compositePass.End();
    }
    {
        // imgui pass
        ImGui::Render();
        wgpu::RenderPassColorAttachment uiAttachment{
            .view = GetNextSurfaceTextureView(),
            .loadOp = wgpu::LoadOp::Load,
            .storeOp = wgpu::StoreOp::Store,
        };
        
        wgpu::RenderPassDescriptor uiPassDesc{
            .colorAttachmentCount = 1,
            .colorAttachments = &uiAttachment,
        };

        wgpu::RenderPassEncoder uiPass = commandEncoder.BeginRenderPass(&uiPassDesc);
        ImGui_ImplWGPU_RenderDrawData(ImGui::GetDrawData(), uiPass.Get());
        uiPass.End();
    }

    // finish recording
    wgpu::CommandBufferDescriptor commandBufferDesc { .label = "Command Buffer" };
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

display::Application::~Application() {
    ImGui_ImplWGPU_Shutdown();
    ImGui_ImplGlfw_Shutdown();
    ImGui::DestroyContext();
}
