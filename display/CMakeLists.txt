cmake_minimum_required( VERSION 3.20 )
project( display VERSION 0.1.0 LANGUAGES CXX C )
set( CMAKE_CXX_STANDARD 23 )
set( CMAKE_EXPORT_COMPILE_COMMANDS ON )

add_library( display_lib STATIC src/display.cpp src/molecule.cpp )
target_include_directories( display_lib PUBLIC include )

# quaternions header
add_library( quaternions INTERFACE )
target_include_directories( quaternions INTERFACE
    ${CMAKE_CURRENT_SOURCE_DIR}/third_party/quaternions/include
)
target_link_libraries( display_lib PRIVATE quaternions )

add_executable( display src/main.cpp )
target_link_libraries( display PRIVATE display_lib )

set( DAWN_FETCH_DEPENDENCIES ON )
set( DAWN_BUILD_MONOLITHIC_LIBRARY STATIC )
add_subdirectory( third_party/dawn EXCLUDE_FROM_ALL )

set_target_properties( display PROPERTIES
    CXX_STANDARD_REQUIRED ON
    CXX_EXTENSIONS OFF
    COMPILE_WARNING_AS_ERROR ON
)

if (MSVC)
    target_compile_options( display PRIVATE /W4 )
else()
    target_compile_options( display PRIVATE -Wall -Wextra -pedantic )
endif()

if (XCODE)
    set_target_properties( display PROPERTIES
        XCODE_GENERATE_SCHEME ON
        XCODE_SCHEME_ENABLE_GPU_FRAME_CAPTURE_MODE "Metal"
    )
endif()

if(EMSCRIPTEN)
    set_target_properties( display PROPERTIES SUFFIX ".html" )
    target_link_libraries( display_lib PUBLIC emdawnwebgpu_cpp webgpu_glfw )
    target_link_options( display_lib PUBLIC "-sASYNCIFY=1" "-sUSE_GLFW=3" )
else()
    target_link_libraries( display_lib PUBLIC webgpu_dawn webgpu_glfw glfw )
endif()
