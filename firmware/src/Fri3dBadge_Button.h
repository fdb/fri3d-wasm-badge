// Vendored verbatim from Fri3dCamp/badge_2024_arduino
// Source: examples/platformio basic examples/buttons/src/Fri3dBadge_Button.h
// Based on JC_Button by Jack Christensen (GPLv3).

#ifndef FRI3DBADGE_BUTTON_H_INCLUDED
#define FRI3DBADGE_BUTTON_H_INCLUDED

#include <Arduino.h>

enum Fri3DButton_type
{
 FRI3D_BUTTON_TYPE_DIGITAL=1,
 FRI3D_BUTTON_TYPE_ANALOGUE_HIGH,
 FRI3D_BUTTON_TYPE_ANALOGUE_LOW,
 FRI3D_BUTTON_TYPE_TOUCH,
};

class Fri3d_Button
{
    public:
        Fri3d_Button(Fri3DButton_type buttontype, uint8_t pin, uint32_t dbTime=25, uint8_t mode=INPUT_PULLUP, uint8_t invert=true)
        {
            m_buttontype = buttontype;
            m_pin = pin;
            m_dbTime = dbTime;
            m_mode = mode;
            m_invert = invert;
        }

        void begin();
        bool read();
        bool isPressed() const {return m_state;}
        bool isReleased() const {return !m_state;}
        bool wasPressed() const {return m_state && m_changed;}
        bool wasReleased() const {return !m_state && m_changed;}
        bool pressedFor(uint32_t ms) const {return m_state && m_time - m_lastChange >= ms;}
        bool releasedFor(uint32_t ms) const {return !m_state && m_time - m_lastChange >= ms;}
        uint32_t lastChange() const {return m_lastChange;}

    private:
        bool readstate();

        uint8_t m_pin;
        uint32_t m_dbTime;
        uint8_t m_mode;
        bool m_invert;
        bool m_state = false;
        bool m_lastState = false;
        bool m_changed = false;
        uint32_t m_time = 0;
        uint32_t m_lastChange = 0;
        Fri3DButton_type m_buttontype;
};

#endif
