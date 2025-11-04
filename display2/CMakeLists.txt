cmake_minimum_required( VERSION 3.20 )
project( display2 VERSION 0.1.0 LANGUAGES CXX C )
set( CMAKE_CXX_STANDARD 20 )
set( CMAKE_EXPORT_COMPILE_COMMANDS ON )

add_library( display2_lib STATIC src/display.cpp )
target_include_directories( display2_lib PUBLIC include )

add_executable( display2
    src/main.cpp
)
target_link_libraries( display2 PRIVATE display2_lib )

set(DAWN_FETCH_DEPENDENCIES ON)
set(DAWN_BUILD_MONOLITHIC_LIBRARY STATIC)
add_subdirectory("third_party/dawn" EXCLUDE_FROM_ALL)

set_target_properties( display2 PROPERTIES
    CXX_STANDARD_REQUIRED ON
    CXX_EXTENSIONS OFF
    COMPILE_WARNING_AS_ERROR ON
)

if (MSVC)
    target_compile_options( display2 PRIVATE /W4 )
else()
    target_compile_options( display2 PRIVATE -Wall -Wextra -pedantic )
endif()

if (XCODE)
    set_target_properties( display2 PROPERTIES
        XCODE_GENERATE_SCHEME ON
        XCODE_SCHEME_ENABLE_GPU_FRAME_CAPTURE_MODE "Metal"
    )
endif()

if(EMSCRIPTEN)
    set_target_properties( display2 PROPERTIES SUFFIX ".html" )
    target_link_libraries( display2_lib PUBLIC emdawnwebgpu_cpp webgpu_glfw )
    target_link_options( display2_lib PUBLIC "-sASYNCIFY=1" "-sUSE_GLFW=3" )
else()
    target_link_libraries( display2_lib PUBLIC webgpu_dawn webgpu_glfw glfw )
endif()
