#include <GLFW/glfw3.h>
#include <webgpu/webgpu_cpp.h>

#include "display.hpp"

#ifdef __EMSCRIPTEN__
#include <emscripten/emscripten.h>
#endif

int main() {
    display::Application app;

    if (!app.Initialize(WIDTH, HEIGHT))
        return EXIT_FAILURE;

#ifdef __EMSCRIPTEN__
    emscripten_set_main_loop_arg(
        [](void *arg) {
            auto *pApp = static_cast<display::Application*>(arg);
            pApp->RenderPresent();
        },
        &app, // arg passed to callback
        0,    // fps (0 means that browser decides)
        true  // simulate_infinite_loop (true means that execution will pause here)
    );
#else
    while (app.KeepRunning()) {
        glfwPollEvents();
        app.RenderPresent();
    }
#endif

    return EXIT_SUCCESS;
}
