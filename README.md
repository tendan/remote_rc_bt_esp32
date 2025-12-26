# Remote-Controlled Rover via Bluetooth LE (ESP32 + Android)

---

## 🚀 Project Overview

This project implements a comprehensive system for remotely controlling a mobile robot (rover) using **Bluetooth Low Energy (BLE)**. The system is split into two main components: the embedded firmware running on the ESP32 and a client application for Android, enabling intuitive command and control.

The goal is to create a robust, energy-efficient, and safety-focused remote control solution for small-scale robotic platforms.

---

## 🛠️ System Components & Technologies

The project is built upon a modern, safety-oriented software stack and common hardware components:

### Software Stack

| Component | Technology | Purpose |
| :--- | :--- | :--- |
| **Firmware** | **Rust** on **ESP32** | Provides low-level, safe, and efficient control logic for the rover's motors. |
| **Operating System** | **Embassy** | An asynchronous, zero-cost, and robust runtime/executor for embedded systems. |
| **BLE Stack** | **TrouBLE** | A secure and reliable Bluetooth Low Energy stack implementation for the application. |
| **Testing/Debugging** | **defmt-test** | A minimal, highly efficient testing framework for embedded devices, used for fast development cycles. |
| **Client** | **[Android App](https://github.com/tendan/remote_rc_bt_client)** | The client interface (to be implemented) for sending control commands (speed, direction) via BLE. |

### Hardware

* **ESP32 Microcontroller:** The main processing unit, chosen for its built-in BLE capabilities and Rust support.
* **Bluetooth Low Energy (BLE):** The wireless communication standard used for transmitting low-latency control data between the Android client and the rover.
* **Motor Driver:** Used to translate the ESP32's control signals into the appropriate current for the drive motors.

---

## 💡 Potential Applications

This control system can be easily adapted for various remote control scenarios:

* **Educational Robotics:** As a foundation for learning embedded Rust and BLE communication.
* **Small-Scale Industrial Inspection:** Controlling miniature platforms for visual inspection in hard-to-reach areas.
* **Hobbyist RC Vehicles:** A custom, advanced control system for DIY rovers and cars.

---

## 🎬 Video Demonstration

See the remote control system in action!

[Link to a video demonstration of the remote-controlled robot]

---

## 📜 License

This project is released under the **BSD-3-Clause License**. See the [LICENSE](LICENSE) file for more details.
