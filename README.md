[![Build Status](https://travis-ci.org/tsathishkumar/MySController-rs.svg?branch=master)](https://travis-ci.org/tsathishkumar/MySController-rs) [ ![Download](https://api.bintray.com/packages/tsathishkumar/myscontroller-rs/myscontroller-rs/images/download.svg) ](https://bintray.com/tsathishkumar/myscontroller-rs/myscontroller-rs/_latestVersion)
# MySController-rs
Proxy controller for MySensors written in Rust lang. It is to perform OTA firmware updates, and proxy all other requests to the actual controllers like homeassist. Mainly to add OTA support for homeassist controller, but can work with any other controllers.

This server acts as a proxy between Gateway and the Controller. Both might be either connected through a serial port or a TCP connection.

Before running the server, set the correct connection type and connection port for Gateway and Controller in conf.ini file.

## To run the proxy server:
```
cargo run
```

## To install and run as a service in a debian/ubuntu flavour machine
- Add the following to your /etc/apt/sources.list system config file:
    ```bash
    echo "deb http://dl.bintray.com/tsathishkumar/myscontroller-rs vivid main" | sudo tee -a /etc/apt/sources.list
    ```
- Update the package list
    ```bash
    apt-get update
    ```
- Install the package
    ```bash
    apt install myscontroller-rs
    ```
- The configuration of the server can be found at the below location. 
    ```bash
    /etc/myscontroller-rs/conf.ini
    ```
    Example settings:
    ```bash
    encoding=utf-8

    [Gateway]
    type=TCP
    port=10.11.12.13:5003

    [Controller]
    type=TCP
    port=0.0.0.0:5003

    [Server]
    database_url=/var/lib/myscontroller-rs/sqlite.db
    ```
- Set up the right Gateway IP and Controller IP and restart the service.
    ```bash
    systemctl restart myscontroller-rs.service
    ```


Note: If you are using TCP for controller - the port value will be used to create TCP server listening on the specified port. (So it shoud be the address of the machine running MyRController)

## TODO

- [x] Gracefully handle connection at both side, i.e never panic and wait for both connections
- [x] Ability to handle ota requests even when there is no controller connected
- [x] Ability to restart the node using api
- [x] Manage nodes and the firmwares installed, expose api's 
    - GET `/nodes`
    - PUT `/node` `<node>`
    - POST `/reboot_node/<node_id>`
- [x] Get node's firmware type and version from database and use it for ota request from node
- [ ] Handle auto update flag in node 
    - whenever there is new version for a firmware, it should automatically update all nodes which have auto update as `true` to latest version
- [x] Manage firmwares type and version, ability to upload newer versions of firmwares, expose apis 
    - GET `/firmwares` - `[{"firmware_type": 10, "firmware_version": 1, "firmware_name", "Blink"}]`
    - DELETE `/firmwares/{type}/{version}`
    - POST `/firmwares` `{ "firmware_type": 10, "firmware_version": 1, "firmware_name": "Blink", "file": <file>}` - Done
    - PUT `/firmwares` `{ "firmware_type": 10, "firmware_version": 1, "firmware_name": "Blink", "file": <file>}` - Done
- [x] Improve error handling in api's (handling unique constraint in insert, updating unavailable firmwares etc)    
- [ ] Improve logging (parsed message for OTA request etc)
- [ ] Improve error handling across project (remove unwraps)
- [ ] MQTT integration
- [ ] Node name support
- [ ] Child sensors support


## Future goals:

- Parse all the data and expose WoT API's using [webthing-rust](https://github.com/mozilla-iot/webthing-rust)
- MQTT support
- Store the "states" of each nodes - to make it standalone
- Beats/Telegraph support - to store "telemetri" data
