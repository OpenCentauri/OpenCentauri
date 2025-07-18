# SDCP WebSocket API Documentation - Centauri Carbon FDM Printer

The Centauri Carbon makes use of the "Smart Device Control Protocol"

[See here for official documentation of the protocol](https://github.com/cbd-tech/SDCP-Smart-Device-Control-Protocol-V3.0.0/blob/main/SDCP(Smart%20Device%20Control%20Protocol)_V3.0.0_EN.md)

However, the Centauri has some specific quirks on this specification. See below for a more accurate overview of the API of the Centauri printers.

Written by remmylee on the Elegoo discord.

## Protocol Overview

The Smart Device Control Protocol (SDCP) v3.0.0 is an application layer protocol for interaction between clients and the Centauri Carbon FDM printer motherboard. Communication is conducted using JSON format over WebSocket connections, with MJPEG streams for video.

## Connection Establishment

### Device Discovery
Before establishing a WebSocket connection, devices must be discovered via UDP broadcast:

**UDP Broadcast Discovery:**

- Port: 3000
- Message: `"M99999"`
- Response format:
```json
{
    "Id": "uuid-string",
    "Data": {
        "Name": "Centauri Carbon",
        "MachineName": "Centauri Carbon", 
        "BrandName": "Centauri",
        "MainboardIP": "192.168.1.2",
        "MainboardID": "000000000001d354",
        "ProtocolVersion": "V3.0.0",
        "FirmwareVersion": "V1.0.0"
    }
}
```

**Example**:

```py
import socket

# IP: x.x.x.255
broadcast_address = ('', 3000)

if __name__ == "__main__":
    if broadcast_address[0] == '':
        raise ValueError("Broadcast address cannot be empty. Please specify a valid broadcast address.")

    broadcast_message = "M99999"
    udp_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    udp_socket.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)

    try:
        print(f"Sending broadcast message: {broadcast_message}")
        udp_socket.sendto(broadcast_message.encode(), broadcast_address)

        udp_socket.settimeout(5)
        while True:
            try:
                data, addr = udp_socket.recvfrom(1024)
                print(f"Received response from {addr}: {data.decode()}")
            except socket.timeout:
                print("Listening timed out. No more responses.")
                break
    finally:
        udp_socket.close()
```

### WebSocket Connection
- URL: `ws://{MainboardIP}:3030/websocket`
- Alternative paths: `/ws`, `/`, `/api/websocket`, `/sdcp`
- Protocol: WebSocket over TCP

## Message Format

All WebSocket messages use JSON format with the following structure:

### Request Message Format
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 0,
        "Data": {},
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

### Response Message Format
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 0,
        "Data": {
            "Ack": 0
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655
    },
    "Topic": "sdcp/response/{MainboardID}"
}
```

## Topic Structure

The protocol uses topic-based messaging with the following patterns:

- **Control Request:** `sdcp/request/{MainboardID}`
- **Control Response:** `sdcp/response/{MainboardID}`
- **Status Information:** `sdcp/status/{MainboardID}`
- **Attribute Information:** `sdcp/attributes/{MainboardID}`
- **Error Messages:** `sdcp/error/{MainboardID}`
- **Notification Messages:** `sdcp/notice/{MainboardID}`

## Heartbeat

**Request:** `"ping"`

**Response:** `"pong"`

## Protocol Field Naming Issues

**IMPORTANT**: The SDCP protocol contains several field names with spelling errors that must be used exactly as specified:

### Known Spelling Errors in Field Names
- `CurrenCoord` - Missing 't' in "Current" (should be "CurrentCoord")
- `RelaseFilmState` - Missing 'e' in "Release" (should be "ReleaseFilmState")
- `MaximumCloudSDCPSercicesAllowed` - Missing 'v' in "Services" (should be "ServicesAllowed")

### Implementation Handling
The implementation code handles these inconsistencies by checking multiple possible field locations:

```javascript
// Example from the code - checking for both spellings
const currentCoord = status.CurrenCoord || status.CurrentCoord || "0,0,0";
```

**Critical**: Always use the misspelled field names when sending requests or parsing responses, as these are the actual field names in the protocol specification.

## Status Information

The motherboard reports status information automatically when:

1. Status information changes
2. Upon receiving the status refresh command

### Status Message Format
```json
{
    "Status": {
        "CurrentStatus": [0,1,2,3],
        "PreviousStatus": 0,
        "TempOfNozzle": 210,
        "TempTargetNozzle": 210,
        "TempOfHotbed": 60,
        "TempTargetHotbed": 60,
        "TempOfBox": 25,
        "TempTargetBox": 0,
        "CurrenCoord": "150.5,75.2,10.8",
        "CurrentFanSpeed": {
            "ModelFan": 100,
            "ModeFan": 100,
            "AuxiliaryFan": 50,
            "BoxFan": 25
        },
        "LightStatus": {
            "SecondLight": 1
        },
        "RgbLight": [255, 255, 255],
        "ZOffset": 0.0,
        "PrintSpeed": 100,
        "PrintInfo": {
            "Status": 1,
            "CurrentLayer": 100,
            "TotalLayer": 1000,
            "CurrentTicks": 3600,
            "TotalTicks": 36000,
            "Filename": "model.gcode",
            "ErrorNumber": 0,
            "TaskId": "uuid-string",
            "PrintSpeed": 100
        }
    },
    "MainboardID": "string",
    "TimeStamp": 1687069655,
    "Topic": "sdcp/status/{MainboardID}"
}
```

### Implementation-Specific Field Usage

This implementation supports FDM printing with the following field mappings:

#### Temperature Fields (FDM)
- `TempOfNozzle` / `TempTargetNozzle` - Extruder temperature (typically 180-250°C)
- `TempOfHotbed` / `TempTargetHotbed` - Heated bed temperature (typically 50-100°C)
- `TempOfBox` / `TempTargetBox` - Chamber temperature (for enclosed printers)

#### Position Data
- `CurrenCoord` - Position as comma-separated string "x,y,z" (e.g., "150.5,75.2,10.8")
  - **Note**: Field name is missing the 't' in "Current" - this is a protocol spelling error
- `ZOffset` - Z-axis offset value as number

#### Fan Control Object
```json
{
    "ModelFan": 100,        // Part cooling fan percentage (0-100%)
    "ModeFan": 100,         // Alternative name for ModelFan  
    "AuxiliaryFan": 50,     // Auxiliary fan percentage (hotend cooling)
    "BoxFan": 25            // Chamber/enclosure fan percentage
}
```

#### Lighting Control
```json
{
    "LightStatus": {
        "SecondLight": 1        // 0=Off, 1=On (printer LED lighting)
    },
    "RgbLight": [255, 255, 255]  // RGB values array for accent lighting
}
```

#### Device Status Values

The `DevicesStatus` object contains FDM hardware component status codes:

```json
{
    "ZMotorStatus": 1,             // 0=Disconnected, 1=Connected (Z-axis stepper)
    "YMotorStatus": 1,             // 0=Disconnected, 1=Connected (Y-axis stepper)
    "XMotorStatus": 1,             // 0=Disconnected, 1=Connected (X-axis stepper)
    "ExtruderMotorStatus": 1,      // 0=Disconnected, 1=Connected (Extruder stepper)
    "RelaseFilmState": 0           // 0=Abnormal, 1=Normal (legacy field from SLA)
}
```

**Note**: `RelaseFilmState` is missing the 'e' in "Release" - this is a protocol spelling error that must be used as-is.

#### Network and Storage Status
- `NetworkStatus` - Connection type: `"wlan"` or `"eth"`
- `UsbDiskStatus` - USB drive: `0=Disconnected`, `1=Connected`
- `CameraStatus` - Camera: `0=Disconnected`, `1=Connected`
- `RemainingMemory` - Available storage in bytes
- `MainboardMAC` - MAC address string

#### Video Streaming Limits
- `NumberOfVideoStreamConnected` - Current active video streams
- `MaximumVideoStreamAllowed` - Maximum concurrent video streams
- `NumberOfCloudSDCPServicesConnected` - Active cloud connections
- `MaximumCloudSDCPSercicesAllowed` - Maximum cloud connections
  - **Note**: Field name is missing 'v' in "Services" - this is a protocol spelling error

#### Print Dimensions and Capabilities
- `XYZsize` - Build volume as string "width x depth x height" (e.g., "300x300x400")
- `Capabilities` - Array of supported features: `["FILE_TRANSFER", "PRINT_CONTROL", "VIDEO_STREAM"]`
- `SupportFileType` - Array of supported file formats: `["GCODE"]`

### Machine Status Codes
The `CurrentStatus` and `PreviousStatus` fields indicate overall machine state:

```
0 = "Idle"                  // Machine is idle and ready
1 = "Printing"              // Executing print task
2 = "File Transferring"     // File transfer in progress
3 = "Calibrating"           // Running calibration routines
4 = "Device Testing"        // Device self-check in progress
```

### Print Status Codes
```
SDCP_PRINT_STATUS_IDLE = 0
SDCP_PRINT_STATUS_HOMING = 1
SDCP_PRINT_STATUS_DROPPING = 2
SDCP_PRINT_STATUS_EXPOSURING = 3
SDCP_PRINT_STATUS_LIFTING = 4
SDCP_PRINT_STATUS_PAUSING = 5
SDCP_PRINT_STATUS_PAUSED = 6
SDCP_PRINT_STATUS_STOPPING = 7
SDCP_PRINT_STATUS_STOPPED = 8
SDCP_PRINT_STATUS_COMPLETE = 9
SDCP_PRINT_STATUS_FILE_CHECKING = 10
```

### Print Error Codes
The `PrintInfo.ErrorNumber` field indicates print-related errors:

```
0 = "Normal"                    // No error
1 = "File MD5 Check Failed"     // File integrity verification failed
2 = "File Read Failed"          // Cannot read print file
3 = "Resolution Mismatch"       // File resolution doesn't match printer
4 = "Format Mismatch"           // Unsupported file format
5 = "Machine Model Mismatch"    // File not compatible with printer model
```

### Extended Error Status Reasons (ErrorStatusReason)
This implementation includes detailed error reporting via `ErrorStatusReason` field in task details:

```
SDCP_PRINT_CAUSE_OK = 0                     // Normal
SDCP_PRINT_CAUSE_TEMP_ERROR = 1             // Over-temperature (nozzle/bed)
SDCP_PRINT_CAUSE_FILAMENT_RUNOUT = 3        // Filament runout detected
SDCP_PRINT_CAUSE_FILAMENT_JAM = 6           // Filament jam or clog detected
SDCP_PRINT_CAUSE_LEVEL_FAILED = 7           // Auto-bed leveling failed
SDCP_PRINT_CAUSE_HOME_FAILED_X = 13         // X-axis motor/endstop failure
SDCP_PRINT_CAUSE_HOME_FAILED_Z = 14         // Z-axis motor/endstop failure
SDCP_PRINT_CAUSE_HOME_FAILED_Y = 23         // Y-axis motor/endstop failure
SDCP_PRINT_CAUSE_HOME_FAILED = 17           // General homing failure
SDCP_PRINT_CAUSE_BED_ADHESION_FAILED = 18   // Print detachment from bed
SDCP_PRINT_CAUSE_ERROR = 19                 // General printing exception
SDCP_PRINT_CAUSE_MOVE_ABNORMAL = 20         // Motor movement abnormality
SDCP_PRINT_CAUSED_FILE_ERROR = 24           // G-code file error
SDCP_PRINT_CAUSED_CAMERA_ERROR = 25         // Camera connection error
SDCP_PRINT_CAUSED_NETWORK_ERROR = 26        // Network connection error
SDCP_PRINT_CAUSED_SERVER_CONNECT_FAILED = 27 // Server connection failed
SDCP_PRINT_CAUSED_DISCONNECT_APP = 28       // App disconnected during print
SDCP_PRINT_CAUSE_UDISK_REMOVE = 12          // USB drive removed during print
SDCP_PRINT_CAUSE_NOZZLE_TEMP_SENSOR_OFFLINE = 33 // Nozzle thermistor offline
SDCP_PRINT_CAUSE_BED_TEMP_SENSOR_OFFLINE = 34    // Bed thermistor offline
```

## Attribute Information

Reported automatically when:
1. Attribute information changes
2. Upon receiving the attribute refresh command

### Attribute Message Format
```json
{
    "Attributes": {
        "Name": "Centauri Carbon",
        "MachineName": "Centauri Carbon",
        "BrandName": "Centauri",
        "ProtocolVersion": "V3.0.0",
        "FirmwareVersion": "V1.0.0",
        "XYZsize": "300x300x400",
        "MainboardIP": "192.168.1.1",
        "MainboardID": "000000000001d354",
        "NumberOfVideoStreamConnected": 1,
        "MaximumVideoStreamAllowed": 1,
        "NumberOfCloudSDCPServicesConnected": 0,
        "MaximumCloudSDCPSercicesAllowed": 1,
        "NetworkStatus": "wlan",
        "MainboardMAC": "00:11:22:33:44:55",
        "UsbDiskStatus": 0,
        "Capabilities": [
            "FILE_TRANSFER",
            "PRINT_CONTROL",
            "VIDEO_STREAM"
        ],
        "SupportFileType": ["GCODE"],
        "DevicesStatus": {
            "ZMotorStatus": 1,             // Z-axis stepper motor
            "YMotorStatus": 1,             // Y-axis stepper motor
            "XMotorStatus": 1,             // X-axis stepper motor
            "ExtruderMotorStatus": 1       // Extruder stepper motor
        },
        "CameraStatus": 1,
        "RemainingMemory": 123455,
        "SDCPStatus": 1
    },
    "MainboardID": "string",
    "TimeStamp": 1687069655,
    "Topic": "sdcp/attributes/{MainboardID}"
}
```

## Command Reference

### Information Commands

#### Request Status Refresh (Cmd: 0)
Forces motherboard to report current status.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 0,
        "Data": {},
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

**Response:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 0,
        "Data": {
            "Ack": 0
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655
    },
    "Topic": "sdcp/response/{MainboardID}"
}
```

#### Request Attributes (Cmd: 1)
Forces motherboard to report current attributes.

**Request/Response:** Same format as status refresh with `"Cmd": 1`

### Print Control Commands

#### Start Print (Cmd: 128)
Initiates printing of a specified file.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 128,
        "Data": {
            "Filename": "model.gcode",
            "StartLayer": 0
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

**Response Ack Codes:**
```
SDCP_PRINT_CTRL_ACK_OK = 0
SDCP_PRINT_CTRL_ACK_BUSY = 1
SDCP_PRINT_CTRL_ACK_NOT_FOUND = 2
SDCP_PRINT_CTRL_ACK_MD5_FAILED = 3
SDCP_PRINT_CTRL_ACK_FILEIO_FAILED = 4
SDCP_PRINT_CTRL_ACK_INVALID_RESOLUTION = 5
SDCP_PRINT_CTRL_ACK_UNKNOWN_FORMAT = 6
SDCP_PRINT_CTRL_ACK_UNKNOWN_MODEL = 7
```

#### Pause Print (Cmd: 129)
Pauses current print job.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 129,
        "Data": {},
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

#### Stop Print (Cmd: 130)
Stops current print job.

**Request:** Same format as pause with `"Cmd": 130`

#### Continue Print (Cmd: 131)
Resumes paused print job.

**Request:** Same format as pause with `"Cmd": 131`

#### Stop Material Feeding (Cmd: 132)
Stops automatic material feeding.

**Request:** Same format as pause with `"Cmd": 132`

#### Skip Preheating (Cmd: 133)
Skips preheating phase.

**Request:** Same format as pause with `"Cmd": 133`

### Configuration Commands

#### Change Printer Name (Cmd: 192)
Updates the printer's display name.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 192,
        "Data": {
            "Name": "Centauri Carbon Lab"
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

#### Set Print Speed
Update the print speed percentage for the current print job. You can find the preset speed values on the [Software Page](/OpenCentauri/software/#speed-profiles).

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 403,
        "Data": {
            "PrintSpeedPct": 50
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

#### Set The Fan Speeds

Update the fan speeds for the different fans on the printer.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 403,
        "Data": {
            "TargetFanSpeed": {
                "ModelFan": 50,
                "AuxiliaryFan": 50,
                "BoxFan": 50,
            }
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

### File Management Commands

#### Retrieve File List (Cmd: 258)
Gets list of files in specified directory.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 258,
        "Data": {
            "Url": "/usb/yourPath"
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

**Path Conventions:**

- `/usb/` - USB storage space
- `/local/` - Onboard storage space
- Default: `/local/` if no path specified

**Response:**

```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 258,
        "Data": {
            "Ack": 0,
            "FileList": [
                {
                    "name": "/usb/xxx",
                    "usedSize": 123456,
                    "totalSize": 123456,
                    "storageType": 0,
                    "type": 0
                }
            ]
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655
    },
    "Topic": "sdcp/response/{MainboardID}"
}
```

**File/Folder Types:**

- `type: 0` - Folder
- `type: 1` - File

**Storage Types:**

- `storageType: 0` - Internal Storage
- `storageType: 1` - External Storage

#### Batch Delete Files (Cmd: 259)
Deletes multiple files and folders.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 259,
        "Data": {
            "FileList": ["/usb/file1", "/usb/file2"],
            "FolderList": ["/usb/folder1", "/usb/folder2"]
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

**Response:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 259,
        "Data": {
            "Ack": 0,
            "ErrData": ["/failed/path"]
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655
    },
    "Topic": "sdcp/response/{MainboardID}"
}
```

### History Commands

#### Retrieve Historical Tasks (Cmd: 320)
Gets list of historical print tasks.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 320,
        "Data": {},
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

**Response:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 320,
        "Data": {
            "Ack": 0,
            "HistoryData": ["task-uuid-1", "task-uuid-2"]
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655
    },
    "Topic": "sdcp/response/{MainboardID}"
}
```

#### Retrieve Task Details (Cmd: 321)
Gets detailed information for specific tasks.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 321,
        "Data": {
            "Id": ["task-uuid-1", "task-uuid-2"]
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

**Response:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 321,
        "Data": {
            "Ack": 0,
            "HistoryDetailList": [
                {
                    "Thumbnail": "http://192.168.1.2/thumb.jpg",
                    "TaskName": "print_job_name",
                    "BeginTime": 1689217424,
                    "EndTime": 1689221024,
                    "TaskStatus": 1,
                    "SliceInformation": {},
                    "AlreadyPrintLayer": 2,
                    "TaskId": "task-uuid",
                    "MD5": "file-hash",
                    "CurrentLayerTalVolume": 0.02,
                    "TimeLapseVideoStatus": 0,
                    "TimeLapseVideoUrl": "http://192.168.1.2/video.mp4",
                    "ErrorStatusReason": 0
                }
            ]
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655
    },
    "Topic": "sdcp/response/{MainboardID}"
}
```

**Task Status:**

- `0` - Other Status
- `1` - Completed
- `2` - Exceptional Status
- `3` - Stopped

**Time-lapse Video Status:**

- `0` - Not shot
- `1` - File exists
- `2` - Deleted
- `3` - Generating
- `4` - Generation failed

### Video Stream Commands

#### Enable/Disable Video Stream (Cmd: 386)
Controls video streaming functionality. Returns an MJPEG stream URL for direct image embedding.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 386,
        "Data": {
            "Enable": 1
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

**Response:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 386,
        "Data": {
            "Ack": 0,
            "VideoUrl": "http://192.168.1.2:3031/video"
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655
    },
    "Topic": "sdcp/response/{MainboardID}"
}
```

**Response Ack Codes:**

- `0` - Success
- `1` - Exceeded maximum streaming limit
- `2` - Camera does not exist
- `3` - Unknown error

**Video Stream Implementation:**
The VideoUrl field contains a direct HTTP MJPEG stream URL that can be embedded directly in an HTML `<img>` element:
```html
<img src="http://192.168.1.2:3031/video" alt="Live Camera Feed">
```

#### Enable/Disable Time-lapse Photography (Cmd: 387)
Controls time-lapse recording functionality for print monitoring.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 387,
        "Data": {
            "Enable": 1
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

### File Transfer Commands

#### Terminate File Transfer (Cmd: 255)
Cancels ongoing file transfer.

**Request:**
```json
{
    "Id": "uuid-string",
    "Data": {
        "Cmd": 255,
        "Data": {
            "Uuid": "transfer-uuid",
            "FileName": "filename.gcode"
        },
        "RequestID": "uuid-string",
        "MainboardID": "string",
        "TimeStamp": 1687069655,
        "From": 0
    },
    "Topic": "sdcp/request/{MainboardID}"
}
```

**Response Ack Codes:**
```
SDCP_FILE_TRANSFER_ACK_SUCCESS = 0
SDCP_FILE_TRANSFER_ACK_NOT_TRANSFER = 1
SDCP_FILE_TRANSFER_ACK_CHECKING = 2
SDCP_FILE_TRANSFER_ACK_NOT_FOUND = 3
```

## Error Messages

Error messages are sent proactively by the motherboard:

```json
{
    "Id": "uuid-string",
    "Data": {
        "Data": {
            "ErrorCode": "error-code"
        },
        "MainboardID": "string",
        "TimeStamp": 1687069655
    },
    "Topic": "sdcp/error/{MainboardID}"
}
```

**Error Codes:**
```
SDCP_ERROR_CODE_MD5_FAILED = 1
SDCP_ERROR_CODE_FORMAT_FAILED = 2
```

## Notification Messages

General notification messages from motherboard:

```json
{
    "Id": "uuid-string",
    "Data": {
        "Data": {
            "Message": "notification-content",
            "Type": 1
        },
        "MainboardID": "string",
        "TimeStamp": 1687069655
    },
    "Topic": "sdcp/notice/{MainboardID}"
}
```

**Notification Types:**
- `1` - History synchronization successful

## Command Source Identification

The `From` field identifies the command source:

```
SDCP_FROM_PC = 0
SDCP_FROM_WEB_PC = 1
SDCP_FROM_WEB = 2
SDCP_FROM_APP = 3
SDCP_FROM_SERVER = 4
```

## HTTP File Transfer Interface

For file uploads, use HTTP POST to:

- **URL:** `http://{MainboardIP}:3030/uploadFile/upload`
- **Method:** POST (multipart/form-data)
- **Form data:**
  - `S-File-MD5: {file-md5-hash}`
  - `Check: 1` (enable verification)
  - `Offset: 0` (file offset)
  - `Uuid: {transfer-uuid}` (unique per transfer)
  - `TotalSize: {file-size}` (total file size)
  - `File: {multipart-file}` (uploaded file contents + name)

**Success Response:**
```json
{
    "code": "000000",
    "messages": null,
    "data": {},
    "success": true
}
```

**Error Response:**
```json
{
    "code": "111111",
    "messages": [
        {
            "field": "common_field",
            "message": -1
        }
    ],
    "data": null,
    "success": false
}
```

**Error Codes:**

- `-1` - Offset error
- `-2` - Offset not match
- `-3` - File open failed
- `-4` - Unknown error

**Python example**

```py
import requests, hashlib

def upload_to_cc(printer_ip : str, filename : str, file : bytes):
    print(f"Uploading '{filename}' ({len(file)} bytes) to printer...")
    url = f"http://{printer_ip}/uploadFile/upload"

    m = hashlib.md5()
    m.update(file)
    file_md5 = m.hexdigest()

    data = {
        "TotalSize": len(file),
        "Uuid": "cc2694a2676b436daec74e53220f87b0",
        "Offset": "0",
        "Check": "1",
        "S-File-MD5": file_md5,
    }

    files = {
        "File": (filename, file, "application/octet-stream")
    }

    response = requests.post(url, data=data, files=files)
    print("Status Code:", response.status_code)
```

## Implementation Notes

1. All timestamps are Unix epoch in seconds
2. UUIDs should be RFC 4122 compliant
3. File transfers are performed in 1MB chunks
4. Video streams use MJPEG over HTTP for direct image embedding
5. WebSocket connections should implement reconnection logic
6. Status updates are pushed automatically, not polled
7. All string fields are UTF-8 encoded
8. Numeric values use standard JSON number format

## Implementation-Specific Behavior

This documentation reflects the actual implementation behavior for the Centauri Carbon FDM printer, which includes:

- **FDM-Specific Temperature Control**: Nozzle, heated bed, and chamber temperature management
- **String-based Coordinates**: Position data in "x,y,z" string format rather than object
- **FDM-Focused Error Reporting**: Comprehensive error status reasons for filament, temperature, and mechanical issues
- **Flexible Field Handling**: Status data can appear at multiple nesting levels
- **Auto-discovery and Connection**: UDP broadcast discovery with automatic WebSocket connection
- **MJPEG Video Streaming**: Direct HTTP MJPEG stream embedding in HTML img elements
- **Real-time Status Updates**: Automatic status polling and push notifications
- **G-code File Support**: Supports standard G-code files for FDM printing

## Video Stream Implementation Details

When video streaming is enabled via Command 386:

1. The printer responds with a VideoUrl containing an MJPEG stream endpoint
2. The client displays the stream by setting the URL as the `src` of an HTML `<img>` element
3. The MJPEG stream provides continuous frame updates automatically
4. No additional video processing or conversion is required
5. Stream type is identified as "http_image" in the implementation

### Client-Side Video Display
```javascript
// Video stream URL received from printer
const videoUrl = "http://192.168.1.2:3031/video";

// Display in HTML img element
const videoPlayer = document.getElementById('video-player');
videoPlayer.innerHTML = `
    <img src="${videoUrl}" 
         alt="Live Camera Feed"
         onload="console.log('Video stream connected')"
         onerror="console.log('Video stream failed')">
`;
```

### Video Stream Events
The implementation emits WebSocket events for video stream status:
- `video_stream_url` - When video URL is received from printer
- `video_stream_stopped` - When video streaming is disabled
- Events include `printer_id`, `video_url`, and `stream_type: "http_image"`

## Field Location Flexibility

Status and attribute data may appear in multiple locations within messages:

- Direct in `last_status` object
- Nested under `Status` property  
- Within `Data` wrapper objects
- As separate `Attributes` objects

**Important**: Due to protocol field naming inconsistencies, clients should check multiple field name variations:
```javascript
// Handle spelling variations
const currentCoord = status.CurrenCoord || status.CurrentCoord || "0,0,0";
const filmState = devices.RelaseFilmState || devices.ReleaseFilmState || 0;
```

Clients should check multiple locations and handle both correct and misspelled field names when parsing responses.
