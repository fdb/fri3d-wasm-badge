// Vendored verbatim from Fri3dCamp/badge_2024_arduino
// Source: examples/platformio basic examples/buttons/src/Fri3dBadge_Button.cpp

#include "Fri3dBadge_Button.h"

void Fri3d_Button::begin()
{
    pinMode(m_pin, m_mode);
    m_state = readstate();
    m_time = millis();
    m_lastState = m_state;
    m_changed = false;
    m_lastChange = m_time;
}

bool Fri3d_Button::read()
{
    uint32_t ms = millis();
    bool pinVal =  readstate();
    if (ms - m_lastChange < m_dbTime) {
        m_changed = false;
    }
    else {
        m_lastState = m_state;
        m_state = pinVal;
        m_changed = (m_state != m_lastState);
        if (m_changed) m_lastChange = ms;
    }
    m_time = ms;
    return m_state;
}

bool Fri3d_Button::readstate()
{
    bool state=m_lastState;
    int value;

    switch (m_buttontype)
    {
        case FRI3D_BUTTON_TYPE_DIGITAL:
            state = digitalRead(m_pin);
            break;

        case FRI3D_BUTTON_TYPE_ANALOGUE_HIGH:
            value = analogRead(m_pin);
            if (value > 3500) state=true;
            if (value < 3400) state=false;
            break;

        case FRI3D_BUTTON_TYPE_ANALOGUE_LOW:
            value = analogRead(m_pin);
            if (value < 500) state=true;
            if (value > 600) state=false;
            break;

        case FRI3D_BUTTON_TYPE_TOUCH:
            value=0;
            for (int count=0; count<5; ++count)
            {
                if (touchRead(m_pin)>45000) {
                    ++value;
                }
            }
            if (value == 0) state=false;
            if (value == 5) state=true;
            break;
    }

    if (m_invert) state = !state;
    return state;
}
