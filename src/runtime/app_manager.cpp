#include "app_manager.h"
#include <iostream>

namespace fri3d {

AppManager::AppManager() = default;

AppManager::~AppManager() = default;

bool AppManager::init(Canvas* canvas, Random* random) {
    m_canvas = canvas;
    m_random = random;

    if (!m_wasmRunner.init()) {
        m_lastError = m_wasmRunner.getLastError();
        return false;
    }

    m_wasmRunner.setCanvas(canvas);
    m_wasmRunner.setRandom(random);

    return true;
}

void AppManager::addApp(const std::string& name, const std::string& path) {
    m_apps.push_back({name, path});
}

void AppManager::clearApps() {
    m_apps.clear();
    m_selectedIndex = 0;
    m_scrollOffset = 0;
}

void AppManager::showLauncher() {
    m_wasmRunner.unloadModule();
    m_inLauncher = true;
}

bool AppManager::launchApp(size_t index) {
    if (index >= m_apps.size()) {
        m_lastError = "Invalid app index";
        return false;
    }

    return launchAppByPath(m_apps[index].path);
}

bool AppManager::launchAppByPath(const std::string& path) {
    if (!m_wasmRunner.loadModule(path)) {
        m_lastError = m_wasmRunner.getLastError();
        return false;
    }

    m_inLauncher = false;
    return true;
}

void AppManager::render() {
    if (m_inLauncher) {
        renderLauncher();
    } else {
        m_wasmRunner.callRender();
    }
}

void AppManager::handleInput(InputKey key, InputType type) {
    if (m_inLauncher) {
        launcherInput(key, type);
    } else {
        // Forward to WASM app (only Press and Release, not ShortPress/LongPress)
        if (type == InputType::Press || type == InputType::Release) {
            m_wasmRunner.callOnInput(static_cast<uint32_t>(key), static_cast<uint32_t>(type));
        }
    }
}

void AppManager::renderLauncher() {
    if (!m_canvas) return;

    m_canvas->clear();
    m_canvas->setColor(Color::Black);
    m_canvas->setFont(Font::Primary);

    // Draw title
    m_canvas->drawStr(2, 10, "Fri3d Apps");
    m_canvas->drawLine(0, 12, 127, 12);

    if (m_apps.empty()) {
        m_canvas->drawStr(2, 30, "No apps found");
        return;
    }

    // Calculate visible range
    size_t startIdx = m_scrollOffset;
    size_t endIdx = std::min(startIdx + VISIBLE_ITEMS, m_apps.size());

    // Draw app list
    for (size_t i = startIdx; i < endIdx; i++) {
        int y = START_Y + static_cast<int>((i - startIdx) + 1) * ITEM_HEIGHT;

        // Draw selection indicator
        if (i == m_selectedIndex) {
            // Filled circle for selected
            m_canvas->drawDisc(CIRCLE_X, y - 3, CIRCLE_RADIUS);
        } else {
            // Empty circle for unselected
            m_canvas->drawCircle(CIRCLE_X, y - 3, CIRCLE_RADIUS);
        }

        // Draw app name
        m_canvas->drawStr(TEXT_X, y, m_apps[i].name.c_str());
    }

    // Draw scroll indicators if needed
    if (m_scrollOffset > 0) {
        // Up arrow at top
        m_canvas->drawStr(120, 20, "^");
    }
    if (endIdx < m_apps.size()) {
        // Down arrow at bottom
        m_canvas->drawStr(120, 60, "v");
    }
}

void AppManager::launcherInput(InputKey key, InputType type) {
    // Only handle key presses, not releases
    if (type != InputType::Press && type != InputType::ShortPress) {
        return;
    }

    if (m_apps.empty()) {
        return;
    }

    switch (key) {
    case InputKey::Up:
        if (m_selectedIndex > 0) {
            m_selectedIndex--;
            // Scroll up if needed
            if (m_selectedIndex < m_scrollOffset) {
                m_scrollOffset = m_selectedIndex;
            }
        }
        break;

    case InputKey::Down:
        if (m_selectedIndex < m_apps.size() - 1) {
            m_selectedIndex++;
            // Scroll down if needed
            if (m_selectedIndex >= m_scrollOffset + VISIBLE_ITEMS) {
                m_scrollOffset = m_selectedIndex - VISIBLE_ITEMS + 1;
            }
        }
        break;

    case InputKey::Ok:
        launchApp(m_selectedIndex);
        break;

    default:
        break;
    }
}

} // namespace fri3d
